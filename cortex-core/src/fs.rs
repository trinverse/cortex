use anyhow::Result;
use chrono::{DateTime, Utc};
use humansize::{format_size, BINARY};
use serde::{Deserialize, Serialize};
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum FileType {
    #[default]
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub file_type: FileType,
    pub size: u64,
    pub size_display: String,
    pub modified: Option<DateTime<Utc>>,
    pub permissions: String,
    pub is_hidden: bool,
    pub extension: Option<String>,
    pub is_selected: bool,
}

impl FileEntry {
    pub fn from_path(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_file() {
            FileType::File
        } else if metadata.is_symlink() {
            FileType::Symlink
        } else {
            FileType::Other
        };

        let size = metadata.len();
        let size_display = if file_type == FileType::Directory {
            "<DIR>".to_string()
        } else {
            format_size(size, BINARY)
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| DateTime::<Utc>::from_timestamp(d.as_secs() as i64, 0))
            .flatten();

        let permissions = format_permissions(&metadata);
        let is_hidden = name.starts_with('.');

        let extension = if file_type == FileType::File {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_lowercase())
        } else {
            None
        };

        Ok(FileEntry {
            name,
            path: path.to_path_buf(),
            file_type,
            size,
            size_display,
            modified,
            permissions,
            is_hidden,
            extension,
            is_selected: false,
        })
    }

    pub fn parent() -> Self {
        FileEntry {
            name: "..".to_string(),
            path: PathBuf::from(".."),
            file_type: FileType::Directory,
            size: 0,
            size_display: "<DIR>".to_string(),
            modified: None,
            permissions: String::new(),
            is_hidden: false,
            extension: None,
            is_selected: false,
        }
    }
}

#[cfg(unix)]
fn format_permissions(metadata: &Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    format!(
        "{}{}{}",
        format_permission_group((mode >> 6) & 0o7),
        format_permission_group((mode >> 3) & 0o7),
        format_permission_group(mode & 0o7)
    )
}

#[cfg(not(unix))]
fn format_permissions(_metadata: &Metadata) -> String {
    "---".to_string()
}

#[cfg(unix)]
fn format_permission_group(perms: u32) -> String {
    format!(
        "{}{}{}",
        if perms & 0o4 != 0 { "r" } else { "-" },
        if perms & 0o2 != 0 { "w" } else { "-" },
        if perms & 0o1 != 0 { "x" } else { "-" }
    )
}

pub struct FileSystem;

impl FileSystem {
    pub fn list_directory(path: &Path, show_hidden: bool) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        if path.parent().is_some() {
            entries.push(FileEntry::parent());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_entry = FileEntry::from_path(&entry.path())?;

            if !show_hidden && file_entry.is_hidden {
                continue;
            }

            entries.push(file_entry);
        }

        entries.sort_by(|a, b| match (&a.file_type, &b.file_type) {
            (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
            (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(entries)
    }

    pub fn get_directory_info(path: &Path) -> Result<(usize, u64)> {
        let mut count = 0;
        let mut total_size = 0;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            count += 1;
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }

        Ok((count, total_size))
    }

    pub fn create_directory(path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn delete_entry(path: &Path) -> Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn copy_entry(src: &Path, dst: &Path) -> Result<()> {
        if src.is_dir() {
            Self::copy_dir_recursive(src, dst)?;
        } else {
            fs::copy(src, dst)?;
        }
        Ok(())
    }

    fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    pub fn move_entry(src: &Path, dst: &Path) -> Result<()> {
        fs::rename(src, dst)?;
        Ok(())
    }
}
