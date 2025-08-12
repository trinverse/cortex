pub mod builder;
pub mod providers;
pub mod traits;
pub mod types;

use anyhow::Result;
use std::io::Read;

use self::providers::{ArchiveProvider, LocalFileSystemProvider};
use self::traits::VfsProvider;

// Re-export commonly used types
pub use self::builder::VirtualFileSystemBuilder;
pub use self::traits::VfsProvider as VfsProviderTrait;
pub use self::types::{RemoteCredentials, VfsEntry, VfsEntryType, VfsPath};

#[cfg(feature = "ssh")]
pub use self::providers::{FtpProvider, SftpProvider, SshConnectionManager};

/// Virtual File System - abstraction over regular files and archive contents
pub struct VirtualFileSystem {
    pub(crate) providers: Vec<Box<dyn VfsProvider>>,
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Box::new(LocalFileSystemProvider),
                Box::new(ArchiveProvider::new()),
            ],
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

    pub fn list_entries(&self, path: &types::VfsPath) -> Result<Vec<types::VfsEntry>> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.list_entries(path);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }

    pub fn read_file(&self, path: &types::VfsPath) -> Result<Box<dyn Read + Send>> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.read_file(path);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }

    pub fn write_file(&self, path: &types::VfsPath, data: Box<dyn Read + Send>) -> Result<()> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.write_file(path, data);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }

    pub fn create_directory(&self, path: &types::VfsPath) -> Result<()> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.create_directory(path);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }

    pub fn delete(&self, path: &types::VfsPath) -> Result<()> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.delete(path);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }

    pub fn get_info(&self, path: &types::VfsPath) -> Result<types::VfsEntry> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.get_info(path);
            }
        }
        Err(anyhow::anyhow!("No provider found for path"))
    }
}