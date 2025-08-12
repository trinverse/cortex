use anyhow::Result;
use std::io::Read;

use crate::vfs::traits::VfsProvider;
use crate::vfs::types::{VfsEntry, VfsPath};

pub struct ArchiveProvider;

impl Default for ArchiveProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveProvider {
    pub fn new() -> Self {
        Self
    }
}

impl VfsProvider for ArchiveProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Archive { .. })
    }

    fn list_entries(&self, _path: &VfsPath) -> Result<Vec<VfsEntry>> {
        // Archive listing implementation would go here
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