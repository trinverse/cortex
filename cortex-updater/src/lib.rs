use anyhow::{Result, Context};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Digest};
use std::io;

pub mod updater;
pub mod downloader;
pub mod installer;
pub mod rollback;

pub use updater::{UpdateChecker, UpdateChannel, UpdateInfo};
pub use downloader::Downloader;
pub use installer::Installer;
pub use rollback::RollbackManager;

/// Configuration for the auto-update system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Enable automatic update checks
    pub enabled: bool,
    
    /// Update channel (stable, beta, nightly)
    pub channel: UpdateChannel,
    
    /// Check frequency in hours
    pub check_interval_hours: u32,
    
    /// Auto-download updates
    pub auto_download: bool,
    
    /// Auto-install updates
    pub auto_install: bool,
    
    /// Update server URL
    pub update_url: String,
    
    /// Public key for signature verification
    pub public_key: Option<String>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channel: UpdateChannel::Stable,
            check_interval_hours: 24,
            auto_download: false,
            auto_install: false,
            update_url: "https://updates.cortex-fm.io".to_string(),
            public_key: None,
        }
    }
}

/// Main auto-updater struct
pub struct AutoUpdater {
    config: UpdateConfig,
    current_version: Version,
    update_dir: PathBuf,
    rollback_manager: RollbackManager,
}

impl AutoUpdater {
    /// Create a new auto-updater instance
    pub fn new(config: UpdateConfig, current_version: Version) -> Result<Self> {
        let update_dir = Self::get_update_directory()?;
        fs::create_dir_all(&update_dir)?;
        
        let rollback_manager = RollbackManager::new(&update_dir)?;
        
        Ok(Self {
            config,
            current_version,
            update_dir,
            rollback_manager,
        })
    }
    
    /// Check for available updates
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        let checker = UpdateChecker::new(
            self.config.update_url.clone(),
            self.config.channel.clone(),
        );
        
        let update_info = checker.check(&self.current_version).await?;
        
        if update_info.version > self.current_version {
            Ok(Some(update_info))
        } else {
            Ok(None)
        }
    }
    
    /// Download an update
    pub async fn download_update(
        &self,
        update_info: &UpdateInfo,
        progress_callback: impl Fn(u64, u64) + Send + 'static,
    ) -> Result<PathBuf> {
        let downloader = Downloader::new(self.update_dir.clone());
        
        let download_path = downloader.download(
            &update_info.download_url,
            &update_info.sha256,
            progress_callback,
        ).await?;
        
        // Verify signature if public key is configured
        if let Some(ref public_key) = self.config.public_key {
            self.verify_signature(&download_path, &update_info.signature, public_key)?;
        }
        
        Ok(download_path)
    }
    
    /// Install an update
    pub async fn install_update(
        &mut self,
        update_path: &Path,
        update_info: &UpdateInfo,
    ) -> Result<()> {
        // Create backup for rollback
        self.rollback_manager.create_backup(&self.current_version)?;
        
        // Install the update
        let installer = Installer::new();
        match installer.install(update_path, &update_info.version).await {
            Ok(_) => {
                self.current_version = update_info.version.clone();
                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                log::error!("Installation failed, rolling back: {}", e);
                self.rollback_manager.rollback(&self.current_version)?;
                Err(e)
            }
        }
    }
    
    /// Get the system update directory
    fn get_update_directory() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .context("Failed to get data directory")?;
        Ok(data_dir.join("cortex").join("updates"))
    }
    
    /// Verify update signature
    fn verify_signature(&self, file_path: &Path, signature: &str, _public_key: &str) -> Result<()> {
        // TODO: Implement actual signature verification
        // For now, just compare SHA256
        let mut file = fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        io::copy(&mut file, &mut hasher)?;
        let result = hasher.finalize();
        let file_hash = hex::encode(result);
        
        if file_hash != signature {
            anyhow::bail!("Signature verification failed");
        }
        
        Ok(())
    }
    
    /// Clean up old update files
    pub fn cleanup_old_updates(&self) -> Result<()> {
        let entries = fs::read_dir(&self.update_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip directories (backups)
            if path.is_dir() {
                continue;
            }
            
            // Remove old update files
            if let Some(extension) = path.extension() {
                if extension == "tar" || extension == "gz" || extension == "zip" {
                    let metadata = fs::metadata(&path)?;
                    let age = metadata.modified()?.elapsed().unwrap_or_default();
                    
                    // Remove files older than 7 days
                    if age.as_secs() > 7 * 24 * 60 * 60 {
                        fs::remove_file(&path)?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Update status for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available(UpdateInfo),
    Downloading { progress: u64, total: u64 },
    Installing,
    Failed(String),
    Success,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_update_config_default() {
        let config = UpdateConfig::default();
        assert!(config.enabled);
        assert_eq!(config.check_interval_hours, 24);
        assert!(!config.auto_download);
        assert!(!config.auto_install);
    }
    
    #[tokio::test]
    async fn test_auto_updater_creation() {
        let config = UpdateConfig::default();
        let version = Version::parse("0.1.0").unwrap();
        let updater = AutoUpdater::new(config, version);
        assert!(updater.is_ok());
    }
}