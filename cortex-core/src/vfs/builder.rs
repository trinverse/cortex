use super::providers::{ArchiveProvider, LocalFileSystemProvider};
use super::traits::VfsProvider;
use super::VirtualFileSystem;

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