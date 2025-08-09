use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::io::Read;
use anyhow::Result;
use serde::{Deserialize, Serialize};
// #[cfg(feature = "ssh")]
// use ssh2::{Session, Sftp};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

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
    Archive { archive_path: PathBuf, internal_path: String },
    Sftp { host: String, port: u16, username: String, path: String },
    Ftp { host: String, port: u16, username: String, path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VfsEntryType {
    File,
    Directory,
    Archive,
    Symlink,
}

/// Connection credentials for remote providers
#[derive(Debug, Clone)]
pub struct RemoteCredentials {
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub passphrase: Option<String>,
}

/// Connection manager for SSH/SFTP sessions
pub struct SshConnectionManager {
    sessions: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<Session>>>>>,
}

impl SshConnectionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    pub fn get_or_create_session(&self, host: &str, port: u16, credentials: &RemoteCredentials) -> Result<Arc<Mutex<Session>>> {
        let session_key = format!("{}:{}", host, port);
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(session) = sessions.get(&session_key) {
            return Ok(session.clone());
        }
        
        // Create new SSH connection
        let tcp = TcpStream::connect(format!("{}:{}", host, port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        
        // Authenticate
        if let Some(ref private_key_path) = credentials.private_key_path {
            session.userauth_pubkey_file(
                &credentials.username,
                None,
                private_key_path,
                credentials.passphrase.as_deref(),
            )?;
        } else if let Some(ref password) = credentials.password {
            session.userauth_password(&credentials.username, password)?;
        } else {
            return Err(anyhow::anyhow!("No authentication method provided"));
        }
        
        if !session.authenticated() {
            return Err(anyhow::anyhow!("SSH authentication failed"));
        }
        
        let session_arc = Arc::new(Mutex::new(session));
        sessions.insert(session_key, session_arc.clone());
        
        Ok(session_arc)
    }
}

pub trait VfsProvider: Send + Sync {
    fn can_handle(&self, path: &VfsPath) -> bool;
    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>>;
    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>>;
    fn write_file(&self, path: &VfsPath, content: Box<dyn Read + Send>) -> Result<()>;
    fn create_directory(&self, path: &VfsPath) -> Result<()>;
    fn delete(&self, path: &VfsPath) -> Result<()>;
    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry>;
}

/// Local file system provider
pub struct LocalFsProvider;

impl VfsProvider for LocalFsProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Local(_))
    }
    
    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        match path {
            VfsPath::Local(local_path) => {
                use crate::fs::FileSystem;
                let entries = FileSystem::list_directory(local_path, false)?;
                
                Ok(entries.into_iter().map(|entry| {
                    let entry_type = match entry.file_type {
                        crate::fs::FileType::File => {
                            if is_supported_archive(&entry.path) {
                                VfsEntryType::Archive
                            } else {
                                VfsEntryType::File
                            }
                        },
                        crate::fs::FileType::Directory => VfsEntryType::Directory,
                        crate::fs::FileType::Symlink => VfsEntryType::Symlink,
                        _ => VfsEntryType::File,
                    };
                    
                    VfsEntry {
                        name: entry.name,
                        path: VfsPath::Local(entry.path),
                        entry_type,
                        size: entry.size,
                        modified: entry.modified
                            .map(|dt| dt.timestamp() as u64)
                            .map(|ts| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(ts))
                            .unwrap_or(SystemTime::UNIX_EPOCH),
                        permissions: entry.permissions,
                        compressed_size: None,
                    }
                }).collect())
            }
            _ => Err(anyhow::anyhow!("LocalFsProvider cannot handle non-local paths")),
        }
    }
    
    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        match path {
            VfsPath::Local(local_path) => {
                let file = std::fs::File::open(local_path)?;
                Ok(Box::new(file))
            }
            _ => Err(anyhow::anyhow!("LocalFsProvider cannot handle non-local paths")),
        }
    }
    
    fn write_file(&self, path: &VfsPath, mut content: Box<dyn Read + Send>) -> Result<()> {
        match path {
            VfsPath::Local(local_path) => {
                let mut file = std::fs::File::create(local_path)?;
                std::io::copy(&mut content, &mut file)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("LocalFsProvider cannot handle non-local paths")),
        }
    }
    
    fn create_directory(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Local(local_path) => {
                std::fs::create_dir_all(local_path)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("LocalFsProvider cannot handle non-local paths")),
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
            _ => Err(anyhow::anyhow!("LocalFsProvider cannot handle non-local paths")),
        }
    }
    
    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        match path {
            VfsPath::Local(local_path) => {
                let metadata = std::fs::metadata(local_path)?;
                let entry_type = if metadata.is_dir() {
                    VfsEntryType::Directory
                } else if is_supported_archive(local_path) {
                    VfsEntryType::Archive
                } else {
                    VfsEntryType::File
                };
                
                Ok(VfsEntry {
                    name: local_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    path: path.clone(),
                    entry_type,
                    size: metadata.len(),
                    modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    permissions: format_permissions(&metadata),
                    compressed_size: None,
                })
            }
            _ => Err(anyhow::anyhow!("LocalFsProvider cannot handle non-local paths")),
        }
    }
}

/// ZIP archive provider
pub struct ZipArchiveProvider;

impl VfsProvider for ZipArchiveProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Archive { archive_path, .. } if is_zip_archive(archive_path))
    }
    
    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        match path {
            VfsPath::Archive { archive_path, internal_path } => {
                use zip::ZipArchive;
                
                let file = std::fs::File::open(archive_path)?;
                let mut archive = ZipArchive::new(file)?;
                let mut entries = Vec::new();
                
                let prefix = if internal_path.is_empty() {
                    String::new()
                } else {
                    format!("{}/", internal_path.trim_matches('/'))
                };
                
                // Collect all entries that are direct children of the current path
                for i in 0..archive.len() {
                    let file = archive.by_index(i)?;
                    let file_path = file.name();
                    
                    if file_path.starts_with(&prefix) {
                        let relative_path = &file_path[prefix.len()..];
                        
                        // Skip if this is the current directory itself
                        if relative_path.is_empty() {
                            continue;
                        }
                        
                        // Check if this is a direct child (no additional slashes)
                        let parts: Vec<&str> = relative_path.trim_end_matches('/').split('/').collect();
                        if parts.len() == 1 {
                            let name = parts[0].to_string();
                            let is_dir = file_path.ends_with('/') || file.is_dir();
                            
                            entries.push(VfsEntry {
                                name: name.clone(),
                                path: VfsPath::Archive {
                                    archive_path: archive_path.clone(),
                                    internal_path: if prefix.is_empty() {
                                        name
                                    } else {
                                        format!("{}/{}", internal_path, name)
                                    },
                                },
                                entry_type: if is_dir {
                                    VfsEntryType::Directory
                                } else {
                                    VfsEntryType::File
                                },
                                size: file.size(),
                                modified: SystemTime::UNIX_EPOCH,
                                permissions: if is_dir { "drwxr-xr-x" } else { "-rw-r--r--" }.to_string(),
                                compressed_size: Some(file.compressed_size()),
                            });
                        }
                    }
                }
                
                // Sort entries: directories first, then files
                entries.sort_by(|a, b| {
                    match (&a.entry_type, &b.entry_type) {
                        (VfsEntryType::Directory, VfsEntryType::File) => std::cmp::Ordering::Less,
                        (VfsEntryType::File, VfsEntryType::Directory) => std::cmp::Ordering::Greater,
                        _ => a.name.cmp(&b.name),
                    }
                });
                
                Ok(entries)
            }
            _ => Err(anyhow::anyhow!("ZipArchiveProvider can only handle archive paths")),
        }
    }
    
    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        match path {
            VfsPath::Archive { archive_path, internal_path } => {
                use zip::ZipArchive;
                
                let file = std::fs::File::open(archive_path)?;
                let mut archive = ZipArchive::new(file)?;
                let mut zip_file = archive.by_name(internal_path)?;
                
                // Read the entire file into memory
                let mut buffer = Vec::new();
                zip_file.read_to_end(&mut buffer)?;
                
                Ok(Box::new(std::io::Cursor::new(buffer)))
            }
            _ => Err(anyhow::anyhow!("ZipArchiveProvider can only handle archive paths")),
        }
    }
    
    fn write_file(&self, _path: &VfsPath, _content: Box<dyn Read + Send>) -> Result<()> {
        Err(anyhow::anyhow!("Writing to ZIP archives is not yet supported"))
    }
    
    fn create_directory(&self, _path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("Creating directories in ZIP archives is not yet supported"))
    }
    
    fn delete(&self, _path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("Deleting from ZIP archives is not yet supported"))
    }
    
    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        match path {
            VfsPath::Archive { archive_path, internal_path } => {
                use zip::ZipArchive;
                
                let file = std::fs::File::open(archive_path)?;
                let mut archive = ZipArchive::new(file)?;
                let zip_file = archive.by_name(internal_path)?;
                
                Ok(VfsEntry {
                    name: Path::new(internal_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    path: path.clone(),
                    entry_type: if zip_file.is_dir() {
                        VfsEntryType::Directory
                    } else {
                        VfsEntryType::File
                    },
                    size: zip_file.size(),
                    modified: SystemTime::UNIX_EPOCH,
                    permissions: if zip_file.is_dir() { "drwxr-xr-x" } else { "-rw-r--r--" }.to_string(),
                    compressed_size: Some(zip_file.compressed_size()),
                })
            }
            _ => Err(anyhow::anyhow!("ZipArchiveProvider can only handle archive paths")),
        }
    }
}

/// SFTP remote file system provider
pub struct SftpProvider {
    connection_manager: Arc<SshConnectionManager>,
    credentials: RemoteCredentials,
}

impl SftpProvider {
    pub fn new(credentials: RemoteCredentials) -> Self {
        Self {
            connection_manager: Arc::new(SshConnectionManager::new()),
            credentials,
        }
    }
    
    fn get_sftp_session(&self, host: &str, port: u16) -> Result<(Arc<Mutex<Session>>, Sftp)> {
        let session = self.connection_manager.get_or_create_session(host, port, &self.credentials)?;
        let session_guard = session.lock().unwrap();
        let sftp = session_guard.sftp()?;
        std::mem::drop(session_guard); // Release the lock
        Ok((session.clone(), sftp))
    }
}

impl VfsProvider for SftpProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Sftp { .. })
    }
    
    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                let mut entries = Vec::new();
                
                // Add parent directory entry if not at root
                if remote_path != "/" {
                    entries.push(VfsEntry {
                        name: "..".to_string(),
                        path: VfsPath::Sftp {
                            host: host.clone(),
                            port: *port,
                            username: self.credentials.username.clone(),
                            path: if let Some(parent) = Path::new(remote_path).parent() {
                                parent.to_string_lossy().to_string()
                            } else {
                                "/".to_string()
                            },
                        },
                        entry_type: VfsEntryType::Directory,
                        size: 0,
                        modified: SystemTime::UNIX_EPOCH,
                        permissions: "drwxr-xr-x".to_string(),
                        compressed_size: None,
                    });
                }
                
                // List directory contents
                let dir_entries = sftp.readdir(&Path::new(remote_path))?;
                for (file_path, stat) in dir_entries {
                    let name = file_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    let entry_type = if stat.is_dir() {
                        VfsEntryType::Directory
                    } else if stat.is_file() {
                        VfsEntryType::File
                    } else {
                        VfsEntryType::Symlink
                    };
                    
                    let new_path = if remote_path.ends_with('/') {
                        format!("{}{}", remote_path, name)
                    } else {
                        format!("{}/{}", remote_path, name)
                    };
                    
                    entries.push(VfsEntry {
                        name,
                        path: VfsPath::Sftp {
                            host: host.clone(),
                            port: *port,
                            username: self.credentials.username.clone(),
                            path: new_path,
                        },
                        entry_type,
                        size: stat.size.unwrap_or(0),
                        modified: SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(stat.mtime.unwrap_or(0)),
                        permissions: format!("{:o}", stat.perm.unwrap_or(0o644)),
                        compressed_size: None,
                    });
                }
                
                // Sort entries: directories first, then files
                entries.sort_by(|a, b| {
                    match (&a.entry_type, &b.entry_type) {
                        (VfsEntryType::Directory, VfsEntryType::File) => std::cmp::Ordering::Less,
                        (VfsEntryType::File, VfsEntryType::Directory) => std::cmp::Ordering::Greater,
                        _ => a.name.cmp(&b.name),
                    }
                });
                
                Ok(entries)
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
    
    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                
                // Read the entire file into memory
                let mut file = sftp.open(&Path::new(remote_path))?;
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer)?;
                
                Ok(Box::new(std::io::Cursor::new(buffer)))
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
    
    fn write_file(&self, path: &VfsPath, mut content: Box<dyn Read + Send>) -> Result<()> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                
                let mut file = sftp.create(&Path::new(remote_path))?;
                std::io::copy(&mut content, &mut file)?;
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
    
    fn create_directory(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                sftp.mkdir(&Path::new(remote_path), 0o755)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
    
    fn delete(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                let remote_path_obj = Path::new(remote_path);
                
                // Try to get file stats to determine if it's a file or directory
                match sftp.stat(remote_path_obj) {
                    Ok(stat) => {
                        if stat.is_dir() {
                            sftp.rmdir(remote_path_obj)?;
                        } else {
                            sftp.unlink(remote_path_obj)?;
                        }
                    }
                    Err(_) => {
                        // If stat fails, try both operations
                        if sftp.unlink(remote_path_obj).is_err() {
                            sftp.rmdir(remote_path_obj)?;
                        }
                    }
                }
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
    
    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                let stat = sftp.stat(&Path::new(remote_path))?;
                
                let name = Path::new(remote_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let entry_type = if stat.is_dir() {
                    VfsEntryType::Directory
                } else if stat.is_file() {
                    VfsEntryType::File
                } else {
                    VfsEntryType::Symlink
                };
                
                Ok(VfsEntry {
                    name,
                    path: path.clone(),
                    entry_type,
                    size: stat.size.unwrap_or(0),
                    modified: SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(stat.mtime.unwrap_or(0)),
                    permissions: format!("{:o}", stat.perm.unwrap_or(0o644)),
                    compressed_size: None,
                })
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
}

/// FTP remote file system provider (placeholder - full implementation pending)
pub struct FtpProvider {
    credentials: RemoteCredentials,
}

impl FtpProvider {
    pub fn new(credentials: RemoteCredentials) -> Self {
        Self { credentials }
    }
}

impl VfsProvider for FtpProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Ftp { .. })
    }
    
    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        // Placeholder implementation - FTP support pending full async refactor
        match path {
            VfsPath::Ftp { .. } => {
                Err(anyhow::anyhow!("FTP support is being implemented"))
            }
            _ => Err(anyhow::anyhow!("FtpProvider can only handle FTP paths")),
        }
    }
    
    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }
    
    fn write_file(&self, path: &VfsPath, _data: Box<dyn Read + Send>) -> Result<()> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }
    
    fn create_directory(&self, path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }
    
    fn delete(&self, path: &VfsPath) -> Result<()> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }
    
    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        Err(anyhow::anyhow!("FTP support is being implemented"))
    }
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Box::new(LocalFsProvider),
                Box::new(ZipArchiveProvider),
            ],
        }
    }
    
    pub fn with_sftp_provider(mut self, provider: SftpProvider) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
    
    pub fn with_ftp_provider(mut self, provider: FtpProvider) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
    
    pub fn add_provider(&mut self, provider: Box<dyn VfsProvider>) {
        self.providers.push(provider);
    }
    
    pub fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.list_entries(path);
            }
        }
        Err(anyhow::anyhow!("No provider can handle path: {:?}", path))
    }
    
    pub fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.read_file(path);
            }
        }
        Err(anyhow::anyhow!("No provider can handle path: {:?}", path))
    }
    
    pub fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        for provider in &self.providers {
            if provider.can_handle(path) {
                return provider.get_info(path);
            }
        }
        Err(anyhow::anyhow!("No provider can handle path: {:?}", path))
    }
    
    pub fn can_navigate_into(&self, entry: &VfsEntry) -> bool {
        matches!(entry.entry_type, VfsEntryType::Directory | VfsEntryType::Archive)
    }
    
    pub fn navigate_into(&self, entry: &VfsEntry) -> Option<VfsPath> {
        match &entry.entry_type {
            VfsEntryType::Directory => Some(entry.path.clone()),
            VfsEntryType::Archive => match &entry.path {
                VfsPath::Local(archive_path) => Some(VfsPath::Archive {
                    archive_path: archive_path.clone(),
                    internal_path: String::new(),
                }),
                _ => None,
            },
            _ => None,
        }
    }
}

// Helper functions
pub fn is_supported_archive(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), "zip" | "tar" | "tar.gz" | "tgz" | "tar.bz2" | "tbz2" | "7z" | "rar")
    } else {
        false
    }
}

fn is_zip_archive(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), "zip")
    } else {
        false
    }
}

#[cfg(unix)]
fn format_permissions(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    format!("{:o}", mode & 0o777)
}

#[cfg(not(unix))]
fn format_permissions(metadata: &std::fs::Metadata) -> String {
    if metadata.permissions().readonly() {
        "r--r--r--".to_string()
    } else {
        "rw-rw-rw-".to_string()
    }
}