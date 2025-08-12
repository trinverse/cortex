use anyhow::Result;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[cfg(feature = "ssh")]
use ssh2::{Session, Sftp};

use crate::vfs::providers::ssh::connection::SshConnectionManager;
use crate::vfs::traits::VfsProvider;
use crate::vfs::types::{RemoteCredentials, VfsEntry, VfsEntryType, VfsPath};

#[cfg(feature = "ssh")]
pub struct SftpProvider {
    connection_manager: Arc<SshConnectionManager>,
    credentials: RemoteCredentials,
}

#[cfg(feature = "ssh")]
impl SftpProvider {
    pub fn new(credentials: RemoteCredentials) -> Self {
        Self {
            connection_manager: Arc::new(SshConnectionManager::new()),
            credentials,
        }
    }

    fn get_sftp_session(&self, host: &str, port: u16) -> Result<(Arc<Mutex<Session>>, Sftp)> {
        let session = self
            .connection_manager
            .get_or_create_session(host, port, &self.credentials)?;
        let session_guard = session.lock().unwrap();
        let sftp = session_guard.sftp()?;
        std::mem::drop(session_guard); // Release the lock
        Ok((session.clone(), sftp))
    }
}

#[cfg(feature = "ssh")]
impl VfsProvider for SftpProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Sftp { .. })
    }

    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        match path {
            VfsPath::Sftp {
                host,
                port,
                username: _,
                path: remote_path,
            } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                let mut entries = Vec::new();

                // List directory contents
                let dir_entries = sftp.readdir(Path::new(remote_path))?;
                for (file_path, stat) in dir_entries {
                    let name = file_path
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
                        modified: SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_secs(stat.mtime.unwrap_or(0)),
                        permissions: format!("{:o}", stat.perm.unwrap_or(0o644)),
                        compressed_size: None,
                    });
                }

                Ok(entries)
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        match path {
            VfsPath::Sftp {
                host,
                port,
                username: _,
                path: remote_path,
            } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;

                // Read the entire file into memory
                let mut file = sftp.open(Path::new(remote_path))?;
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer)?;

                Ok(Box::new(std::io::Cursor::new(buffer)))
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn write_file(&self, path: &VfsPath, mut content: Box<dyn Read + Send>) -> Result<()> {
        match path {
            VfsPath::Sftp {
                host,
                port,
                username: _,
                path: remote_path,
            } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;

                let mut file = sftp.create(Path::new(remote_path))?;
                std::io::copy(&mut content, &mut file)?;

                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn create_directory(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Sftp {
                host,
                port,
                username: _,
                path: remote_path,
            } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                sftp.mkdir(Path::new(remote_path), 0o755)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn delete(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Sftp {
                host,
                port,
                username: _,
                path: remote_path,
            } => {
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
            VfsPath::Sftp {
                host,
                port,
                username: _,
                path: remote_path,
            } => {
                let (_session, sftp) = self.get_sftp_session(host, *port)?;
                let stat = sftp.stat(Path::new(remote_path))?;

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
                    modified: SystemTime::UNIX_EPOCH
                        + std::time::Duration::from_secs(stat.mtime.unwrap_or(0)),
                    permissions: format!("{:o}", stat.perm.unwrap_or(0o644)),
                    compressed_size: None,
                })
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
}