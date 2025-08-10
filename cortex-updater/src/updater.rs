use anyhow::Result;
use reqwest;
use semver::Version;
use serde::{Deserialize, Serialize};

/// Update channel for release tracks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: Version,
    pub channel: UpdateChannel,
    pub download_url: String,
    pub sha256: String,
    pub signature: String,
    pub release_notes: String,
    pub release_date: String,
    pub size: u64,
    pub minimum_version: Option<Version>,
    pub critical: bool,
}

/// Update manifest from server
#[derive(Debug, Deserialize)]
struct UpdateManifest {
    releases: Vec<ReleaseInfo>,
}

#[derive(Debug, Deserialize)]
struct ReleaseInfo {
    version: String,
    channel: UpdateChannel,
    platforms: Vec<PlatformInfo>,
    release_notes: String,
    release_date: String,
    minimum_version: Option<String>,
    critical: bool,
}

#[derive(Debug, Deserialize)]
struct PlatformInfo {
    os: String,
    arch: String,
    download_url: String,
    sha256: String,
    signature: String,
    size: u64,
}

/// Update checker that queries the update server
pub struct UpdateChecker {
    update_url: String,
    channel: UpdateChannel,
    client: reqwest::Client,
}

impl UpdateChecker {
    pub fn new(update_url: String, channel: UpdateChannel) -> Self {
        Self {
            update_url,
            channel,
            client: reqwest::Client::new(),
        }
    }

    /// Check for available updates
    pub async fn check(&self, current_version: &Version) -> Result<UpdateInfo> {
        let manifest_url = format!("{}/manifest.json", self.update_url);

        // Fetch update manifest
        let response = self
            .client
            .get(&manifest_url)
            .header("User-Agent", format!("Cortex/{}", current_version))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch update manifest: {}", response.status());
        }

        let manifest: UpdateManifest = response.json().await?;

        // Find the latest release for our channel
        let platform = Self::get_platform();
        let arch = Self::get_arch();

        for release in manifest.releases {
            // Skip if wrong channel
            if release.channel != self.channel {
                continue;
            }

            let version = Version::parse(&release.version)?;

            // Skip if not newer
            if version <= *current_version {
                continue;
            }

            // Check minimum version requirement
            if let Some(min_ver_str) = &release.minimum_version {
                let min_version = Version::parse(min_ver_str)?;
                if *current_version < min_version {
                    log::warn!(
                        "Update {} requires minimum version {}, current is {}",
                        version,
                        min_version,
                        current_version
                    );
                    continue;
                }
            }

            // Find platform-specific download
            for platform_info in release.platforms {
                if platform_info.os == platform && platform_info.arch == arch {
                    return Ok(UpdateInfo {
                        version,
                        channel: release.channel,
                        download_url: platform_info.download_url,
                        sha256: platform_info.sha256,
                        signature: platform_info.signature,
                        release_notes: release.release_notes,
                        release_date: release.release_date,
                        size: platform_info.size,
                        minimum_version: release
                            .minimum_version
                            .map(|v| Version::parse(&v).unwrap()),
                        critical: release.critical,
                    });
                }
            }
        }

        // No update available
        anyhow::bail!("No update available for current platform");
    }

    /// Get current platform identifier
    fn get_platform() -> String {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Get current architecture
    fn get_arch() -> String {
        if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "aarch64".to_string()
        } else if cfg!(target_arch = "x86") {
            "x86".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

/// Mock update server for testing
pub struct MockUpdateServer {
    releases: Vec<UpdateInfo>,
}

impl MockUpdateServer {
    pub fn new() -> Self {
        Self {
            releases: vec![
                UpdateInfo {
                    version: Version::parse("0.2.0").unwrap(),
                    channel: UpdateChannel::Stable,
                    download_url: "https://example.com/cortex-0.2.0.tar.gz".to_string(),
                    sha256: "abc123def456".to_string(),
                    signature: "signature".to_string(),
                    release_notes: "Bug fixes and improvements".to_string(),
                    release_date: "2025-01-15".to_string(),
                    size: 10_000_000,
                    minimum_version: None,
                    critical: false,
                },
                UpdateInfo {
                    version: Version::parse("0.3.0-beta.1").unwrap(),
                    channel: UpdateChannel::Beta,
                    download_url: "https://example.com/cortex-0.3.0-beta.1.tar.gz".to_string(),
                    sha256: "def789ghi012".to_string(),
                    signature: "signature".to_string(),
                    release_notes: "New features in beta".to_string(),
                    release_date: "2025-01-20".to_string(),
                    size: 11_000_000,
                    minimum_version: Some(Version::parse("0.2.0").unwrap()),
                    critical: false,
                },
            ],
        }
    }

    pub fn get_latest(&self, channel: &UpdateChannel, current: &Version) -> Option<UpdateInfo> {
        self.releases
            .iter()
            .filter(|r| r.channel == *channel && r.version > *current)
            .max_by_key(|r| &r.version)
            .cloned()
    }
}
