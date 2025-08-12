use anyhow::Result;
use std::io::Read;

use super::types::{VfsEntry, VfsPath};

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