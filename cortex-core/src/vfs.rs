// Virtual File System with modular SSH/SFTP/FTP support

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::time::SystemTime;

#[cfg(feature = "ssh")]
use crate::remote::{SshConnectionManager, SftpProvider, FtpProvider};
#[cfg(feature = "ssh")]
use std::sync::Arc;

/// Virtual File System - abstraction over regular files and archive contents
pub struct VirtualFileSystem {
    providers: Vec<Box<dyn VfsProvider>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsEntry {
    pub name: String,
    pub path: VfsPath,
    pub entry_type: VfsEntryType,
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: String,
    pub compressed_size: Option<u64>, // For archive entries
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VfsPath {
    Local(PathBuf),
    Archive {
        archive_path: PathBuf,
        internal_path: String,
    },
    Sftp {
        host: String,
        port: u16,
        username: String,
        path: String,
    },
    Ftp {
        host: String,
        port: u16,
        username: String,
        path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VfsEntryType {
    File,
    Directory,
    Archive,
    Symlink,
}

/// Connection credentials for remote providers
#[cfg(feature = "ssh")]
pub use crate::remote::ssh_manager::RemoteCredentials;

#[cfg(not(feature = "ssh"))]
#[derive(Debug, Clone)]
pub struct RemoteCredentials {
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub passphrase: Option<String>,
}

/// Trait for VFS providers
pub trait VfsProvider: Send + Sync {
    fn can_handle(&self, path: &VfsPath) -> bool;
    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>>;
    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>>;
    fn write_file(&self, path: &VfsPath, data: Box<dyn Read + Send>) -> Result<()>;
    fn create_directory(&self, path: &VfsPath) -> Result<()>;
    fn delete(&self, path: &VfsPath) -> Result<()>;
    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry>;
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut providers: Vec<Box<dyn VfsProvider>> = vec![
            Box::new(LocalFileSystemProvider),
            Box::new(ArchiveProvider::new()),
        ];
        
        #[cfg(feature = "ssh")]
        {
            let ssh_manager = Arc::new(SshConnectionManager::new());
            let credentials = crate::remote::ssh_manager::RemoteCredentials {
                username: String::new(),
                password: None,
                private_key_path: None,
                passphrase: None,
            };
            providers.push(Box::new(SftpProvider::new(ssh_manager, credentials.clone())));
            providers.push(Box::new(FtpProvider::new()));
        }
        
        Self { providers }
    }

    pub fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.list_entries(path);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }

    pub fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.read_file(path);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }
}

/// Local file system provider
struct LocalFileSystemProvider;

impl VfsProvider for LocalFileSystemProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Local(_))
    }

    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        match path {
            VfsPath::Local(local_path) => {
                let mut entries = Vec::new();

                // Add parent directory entry if not at root
                if local_path.parent().is_some() {
                    entries.push(VfsEntry {
                        name: "..".to_string(),
                        path: VfsPath::Local(local_path.parent().unwrap().to_path_buf()),
                        entry_type: VfsEntryType::Directory,
                        size: 0,
                        modified: SystemTime::now(),
                        permissions: String::new(),
                        compressed_size: None,
                    });
                }

                for entry in std::fs::read_dir(local_path)? {
                    let entry = entry?;
                    let metadata = entry.metadata()?;
                    let name = entry.file_name().to_string_lossy().to_string();

                    entries.push(VfsEntry {
                        name,
                        path: VfsPath::Local(entry.path()),
                        entry_type: if metadata.is_dir() {
                            VfsEntryType::Directory
                        } else {
                            VfsEntryType::File
                        },
                        size: metadata.len(),
                        modified: metadata.modified()?,
                        permissions: String::new(),
                        compressed_size: None,
                    });
                }

                Ok(entries)
            }
            _ => Err(anyhow::anyhow!(
                "LocalFileSystemProvider can only handle local paths"
            )),
        }
    }

    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        match path {
            VfsPath::Local(local_path) => {
                let file = std::fs::File::open(local_path)?;
                Ok(Box::new(file))
            }
            _ => Err(anyhow::anyhow!(
                "LocalFileSystemProvider can only handle local paths"
            )),
        }
    }

    fn write_file(&self, _path: &VfsPath, _data: Box<dyn Read + Send>) -> Result<()> {
        // Implementation would go here
        Ok(())
    }

    fn create_directory(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Local(local_path) => {
                std::fs::create_dir_all(local_path)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "LocalFileSystemProvider can only handle local paths"
            )),
        }
    }

    fn delete(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Local(local_path) => {
                if local_path.is_dir() {
                    std::fs::remove_dir_all(local_path)?;
                } else {
                    std::fs::remove_file(local_path)?;
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "LocalFileSystemProvider can only handle local paths"
            )),
        }
    }

    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        match path {
            VfsPath::Local(local_path) => {
                let metadata = std::fs::metadata(local_path)?;
                Ok(VfsEntry {
                    name: local_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path: path.clone(),
                    entry_type: if metadata.is_dir() {
                        VfsEntryType::Directory
                    } else {
                        VfsEntryType::File
                    },
                    size: metadata.len(),
                    modified: metadata.modified()?,
                    permissions: String::new(),
                    compressed_size: None,
                })
            }
            _ => Err(anyhow::anyhow!(
                "LocalFileSystemProvider can only handle local paths"
            )),
        }
    }
}

/// Archive provider for ZIP, TAR, etc.
pub struct ArchiveProvider {
    // Archive handling would go here
}

impl Default for ArchiveProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl VfsProvider for ArchiveProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Archive { .. })
    }

    fn list_entries(&self, _path: &VfsPath) -> Result<Vec<VfsEntry>> {
        // Archive listing would go here
        Ok(Vec::new())
    }

    fn read_file(&self, _path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        Err(anyhow::anyhow!("Archive reading not implemented"))
    }

    fn write_file(&self, _path: &VfsPath, _data: Box<dyn Read + Send>) -> Result<()> {
        Err(anyhow::anyhow!("Archive writing not implemented"))
    }

    fn create_directory(&self, _path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("Cannot create directories in archives"))
    }

    fn delete(&self, _path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("Cannot delete from archives"))
    }

    fn get_info(&self, _path: &VfsPath) -> Result<VfsEntry> {
        Err(anyhow::anyhow!("Archive info not implemented"))
    }
}

/// Builder for VirtualFileSystem
pub struct VirtualFileSystemBuilder {
    providers: Vec<Box<dyn VfsProvider>>,
}

impl Default for VirtualFileSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualFileSystemBuilder {
    pub fn new() -> Self {
        Self {
            providers: vec![Box::new(LocalFileSystemProvider)],
        }
    }
    
    #[cfg(feature = "ssh")]
    pub fn with_sftp_provider(mut self, provider: SftpProvider) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
    
    #[cfg(feature = "ssh")]
    pub fn with_ftp_provider(mut self, provider: FtpProvider) -> Self {
        self.providers.push(Box::new(provider));
        self
    }

    pub fn with_archive_provider(mut self) -> Self {
        self.providers.push(Box::new(ArchiveProvider::new()));
        self
    }

    pub fn build(self) -> VirtualFileSystem {
        VirtualFileSystem {
            providers: self.providers,
        }
    }
}
