use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsEntry {
    pub name: String,
    pub path: VfsPath,
    pub entry_type: VfsEntryType,
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: String,
    pub compressed_size: Option<u64>,
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

#[derive(Debug, Clone)]
pub struct RemoteCredentials {
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub passphrase: Option<String>,
}