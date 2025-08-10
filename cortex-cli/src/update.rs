use anyhow::Result;
use cortex_updater::{AutoUpdater, UpdateChannel, UpdateConfig, UpdateInfo, UpdateStatus};
use semver::Version;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Update manager for the application
pub struct UpdateManager {
    updater: Arc<RwLock<AutoUpdater>>,
    status: Arc<RwLock<UpdateStatus>>,
    current_version: Version,
}

impl UpdateManager {
    pub fn new() -> Result<Self> {
        // Get current version from Cargo.toml
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        // Load update configuration
        let config = Self::load_config()?;

        let updater = AutoUpdater::new(config, current_version.clone())?;

        Ok(Self {
            updater: Arc::new(RwLock::new(updater)),
            status: Arc::new(RwLock::new(UpdateStatus::Idle)),
            current_version,
        })
    }

    /// Load update configuration from settings
    fn load_config() -> Result<UpdateConfig> {
        // Try to load from config file
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("cortex").join("update.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(config_path)?;
                let config: UpdateConfig = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        // Use default configuration
        Ok(UpdateConfig {
            enabled: true,
            channel: UpdateChannel::Stable,
            check_interval_hours: 24,
            auto_download: false,
            auto_install: false,
            update_url: "https://github.com/cortex-fm/cortex/releases".to_string(),
            public_key: None,
        })
    }

    /// Check for updates
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        // Update status
        {
            let mut status = self.status.write().await;
            *status = UpdateStatus::Checking;
        }

        let updater = self.updater.read().await;
        match updater.check_for_updates().await {
            Ok(update_info) => {
                if let Some(ref info) = update_info {
                    let mut status = self.status.write().await;
                    *status = UpdateStatus::Available(info.clone());
                }
                Ok(update_info)
            }
            Err(e) => {
                let mut status = self.status.write().await;
                *status = UpdateStatus::Failed(e.to_string());
                Err(e)
            }
        }
    }

    /// Download update
    pub async fn download_update(&self, update_info: UpdateInfo) -> Result<()> {
        let updater = self.updater.read().await;
        let status = self.status.clone();

        let download_path = updater
            .download_update(&update_info, move |downloaded, total| {
                let status = status.clone();
                tokio::spawn(async move {
                    let mut s = status.write().await;
                    *s = UpdateStatus::Downloading {
                        progress: downloaded,
                        total,
                    };
                });
            })
            .await?;

        log::info!("Update downloaded to: {:?}", download_path);

        // Update status
        {
            let mut status = self.status.write().await;
            *status = UpdateStatus::Success;
        }

        Ok(())
    }

    /// Install update
    pub async fn install_update(&self, update_info: UpdateInfo) -> Result<()> {
        // Update status
        {
            let mut status = self.status.write().await;
            *status = UpdateStatus::Installing;
        }

        // Download first if needed
        let updater = self.updater.read().await;
        let update_path = updater
            .download_update(
                &update_info,
                |_, _| {}, // No progress callback for install
            )
            .await?;

        // Install the update
        let mut updater = self.updater.write().await;
        match updater.install_update(&update_path, &update_info).await {
            Ok(_) => {
                let mut status = self.status.write().await;
                *status = UpdateStatus::Success;
                Ok(())
            }
            Err(e) => {
                let mut status = self.status.write().await;
                *status = UpdateStatus::Failed(e.to_string());
                Err(e)
            }
        }
    }

    /// Get current update status
    pub async fn get_status(&self) -> UpdateStatus {
        self.status.read().await.clone()
    }

    /// Get current version
    pub fn get_current_version(&self) -> &Version {
        &self.current_version
    }

    /// Enable/disable auto-updates
    pub async fn set_auto_update(&self, enabled: bool) -> Result<()> {
        // Save to config
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?;
        let config_path = config_dir.join("cortex").join("update.toml");

        // Create directory if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Load current config or create new
        let mut config = Self::load_config().unwrap_or_default();
        config.enabled = enabled;

        // Save config
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(config_path, content)?;

        Ok(())
    }

    /// Clean up old update files
    pub async fn cleanup(&self) -> Result<()> {
        let updater = self.updater.read().await;
        updater.cleanup_old_updates()
    }
}

/// Check for updates in background
pub async fn background_update_check(manager: Arc<UpdateManager>) {
    loop {
        // Wait for check interval (24 hours by default)
        tokio::time::sleep(tokio::time::Duration::from_secs(24 * 60 * 60)).await;

        // Check for updates
        if let Err(e) = manager.check_for_updates().await {
            log::error!("Background update check failed: {}", e);
        }
    }
}
