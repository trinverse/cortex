use anyhow::Result;
use std::io::Read;
use std::time::SystemTime;

use crate::vfs::traits::VfsProvider;
use crate::vfs::types::{VfsEntry, VfsEntryType, VfsPath};

pub struct LocalFileSystemProvider;

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

    fn write_file(&self, path: &VfsPath, mut data: Box<dyn Read + Send>) -> Result<()> {
        match path {
            VfsPath::Local(local_path) => {
                let mut file = std::fs::File::create(local_path)?;
                std::io::copy(&mut data, &mut file)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "LocalFileSystemProvider can only handle local paths"
            )),
        }
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