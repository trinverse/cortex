use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use semver::Version;
use serde::{Serialize, Deserialize};

/// Backup information for rollback
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    version: Version,
    timestamp: u64,
    path: PathBuf,
}

/// Rollback manager for handling update failures
pub struct RollbackManager {
    backup_dir: PathBuf,
    max_backups: usize,
}

impl RollbackManager {
    pub fn new(update_dir: &Path) -> Result<Self> {
        let backup_dir = update_dir.join("backups");
        fs::create_dir_all(&backup_dir)?;
        
        Ok(Self {
            backup_dir,
            max_backups: 3,
        })
    }
    
    /// Create a backup of the current installation
    pub fn create_backup(&self, version: &Version) -> Result<PathBuf> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        let backup_name = format!("cortex-{}-{}", version, timestamp);
        let backup_path = self.backup_dir.join(&backup_name);
        
        // Get current installation directory
        let install_dir = Self::get_install_directory();
        
        if !install_dir.exists() {
            anyhow::bail!("Installation directory not found");
        }
        
        // Create backup
        self.copy_directory(&install_dir, &backup_path)?;
        
        // Save backup info
        let info = BackupInfo {
            version: version.clone(),
            timestamp,
            path: backup_path.clone(),
        };
        
        let info_path = self.backup_dir.join(format!("{}.json", backup_name));
        let info_json = serde_json::to_string_pretty(&info)?;
        fs::write(info_path, info_json)?;
        
        // Clean up old backups
        self.cleanup_old_backups()?;
        
        Ok(backup_path)
    }
    
    /// Rollback to a previous version
    pub fn rollback(&self, target_version: &Version) -> Result<()> {
        // Find backup for target version
        let backup_info = self.find_backup(target_version)?;
        
        if !backup_info.path.exists() {
            anyhow::bail!("Backup not found: {:?}", backup_info.path);
        }
        
        // Get installation directory
        let install_dir = Self::get_install_directory();
        
        // Remove current installation
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)?;
        }
        
        // Restore backup
        self.copy_directory(&backup_info.path, &install_dir)?;
        
        log::info!("Rolled back to version {}", target_version);
        
        Ok(())
    }
    
    /// Find a backup for a specific version
    fn find_backup(&self, version: &Version) -> Result<BackupInfo> {
        let entries = fs::read_dir(&self.backup_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("json")) {
                let content = fs::read_to_string(&path)?;
                if let Ok(info) = serde_json::from_str::<BackupInfo>(&content) {
                    if info.version == *version {
                        return Ok(info);
                    }
                }
            }
        }
        
        anyhow::bail!("No backup found for version {}", version)
    }
    
    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();
        let entries = fs::read_dir(&self.backup_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("json")) {
                let content = fs::read_to_string(&path)?;
                if let Ok(info) = serde_json::from_str::<BackupInfo>(&content) {
                    backups.push(info);
                }
            }
        }
        
        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(backups)
    }
    
    /// Clean up old backups, keeping only the most recent ones
    fn cleanup_old_backups(&self) -> Result<()> {
        let mut backups = self.list_backups()?;
        
        if backups.len() <= self.max_backups {
            return Ok(());
        }
        
        // Remove oldest backups
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        for backup in backups.iter().skip(self.max_backups) {
            // Remove backup directory
            if backup.path.exists() {
                fs::remove_dir_all(&backup.path)?;
            }
            
            // Remove info file
            let info_file = self.backup_dir.join(format!(
                "cortex-{}-{}.json",
                backup.version, backup.timestamp
            ));
            if info_file.exists() {
                fs::remove_file(info_file)?;
            }
        }
        
        Ok(())
    }
    
    /// Copy a directory recursively
    fn copy_directory(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                self.copy_directory(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Get installation directory
    fn get_install_directory() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            exe_path.parent().unwrap().to_path_buf()
        } else {
            #[cfg(unix)]
            {
                PathBuf::from("/usr/local/bin")
            }
            #[cfg(windows)]
            {
                PathBuf::from("C:\\Program Files\\Cortex")
            }
        }
    }
    
    /// Verify backup integrity
    pub fn verify_backup(&self, version: &Version) -> Result<bool> {
        let backup_info = self.find_backup(version)?;
        
        if !backup_info.path.exists() {
            return Ok(false);
        }
        
        // Check for main executable
        let executable = if cfg!(target_os = "windows") {
            backup_info.path.join("cortex.exe")
        } else {
            backup_info.path.join("cortex")
        };
        
        Ok(executable.exists())
    }
}