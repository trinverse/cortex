use anyhow::Result;
use std::io::Read;

use crate::vfs::traits::VfsProvider;
use crate::vfs::types::{RemoteCredentials, VfsEntry, VfsPath};

#[cfg(feature = "ssh")]
pub struct FtpProvider {
    #[allow(dead_code)]
    credentials: RemoteCredentials,
}

#[cfg(feature = "ssh")]
impl FtpProvider {
    pub fn new(credentials: RemoteCredentials) -> Self {
        Self { credentials }
    }
}

#[cfg(feature = "ssh")]
impl VfsProvider for FtpProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Ftp { .. })
    }

    fn list_entries(&self, _path: &VfsPath) -> Result<Vec<VfsEntry>> {
        // FTP implementation would use suppaftp crate
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }

    fn read_file(&self, _path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }

    fn write_file(&self, _path: &VfsPath, _data: Box<dyn Read + Send>) -> Result<()> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }

    fn create_directory(&self, _path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }

    fn delete(&self, _path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }

    fn get_info(&self, _path: &VfsPath) -> Result<VfsEntry> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }
}