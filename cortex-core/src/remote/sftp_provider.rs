use crate::vfs::{VfsEntry, VfsEntryType, VfsPath, VfsProvider};
use crate::remote::ssh_manager::{RemoteCredentials, SshConnectionManager};
use anyhow::Result;
use ssh2::{FileStat, OpenFlags, OpenType};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

pub struct SftpProvider {
    connection_manager: Arc<SshConnectionManager>,
    credentials: RemoteCredentials,
}

impl SftpProvider {
    pub fn new(connection_manager: Arc<SshConnectionManager>, credentials: RemoteCredentials) -> Self {
        Self {
            connection_manager,
            credentials,
        }
    }

    fn convert_file_stat(&self, stat: FileStat, name: String, path: VfsPath) -> VfsEntry {
        let entry_type = if stat.is_dir() {
            VfsEntryType::Directory
        } else if stat.is_file() {
            VfsEntryType::File
        } else {
            VfsEntryType::Symlink
        };

        let modified = UNIX_EPOCH + std::time::Duration::from_secs(stat.mtime.unwrap_or(0));
        
        let permissions = format!(
            "{}{}{}",
            Self::format_permissions((stat.perm.unwrap_or(0) >> 6) & 0o7),
            Self::format_permissions((stat.perm.unwrap_or(0) >> 3) & 0o7),
            Self::format_permissions(stat.perm.unwrap_or(0) & 0o7)
        );

        VfsEntry {
            name,
            path,
            entry_type,
            size: stat.size.unwrap_or(0),
            modified,
            permissions,
            compressed_size: None,
        }
    }

    fn format_permissions(mode: u32) -> String {
        format!(
            "{}{}{}",
            if mode & 0o4 != 0 { 'r' } else { '-' },
            if mode & 0o2 != 0 { 'w' } else { '-' },
            if mode & 0o1 != 0 { 'x' } else { '-' }
        )
    }
}

impl VfsProvider for SftpProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Sftp { .. })
    }

    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let session = self.connection_manager.get_or_create_session(
                    host,
                    *port,
                    &self.credentials,
                )?;
                
                let sftp = self.connection_manager.create_sftp(&session)?;
                let entries = sftp.readdir(Path::new(remote_path))?;
                
                Ok(entries
                    .into_iter()
                    .map(|(path_buf, stat)| {
                        let name = path_buf
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        
                        let full_path = if remote_path.ends_with('/') {
                            format!("{}{}", remote_path, name)
                        } else {
                            format!("{}/{}", remote_path, name)
                        };
                        
                        self.convert_file_stat(
                            stat,
                            name,
                            VfsPath::Sftp {
                                host: host.clone(),
                                port: *port,
                                username: self.credentials.username.clone(),
                                path: full_path,
                            },
                        )
                    })
                    .collect())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let session = self.connection_manager.get_or_create_session(
                    host,
                    *port,
                    &self.credentials,
                )?;
                
                let sftp = self.connection_manager.create_sftp(&session)?;
                let file = sftp.open_mode(
                    Path::new(remote_path),
                    OpenFlags::READ,
                    0,
                    OpenType::File,
                )?;
                
                struct SftpFileReader {
                    file: ssh2::File,
                }
                
                impl Read for SftpFileReader {
                    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                        self.file.read(buf)
                    }
                }
                
                unsafe impl Send for SftpFileReader {}
                
                Ok(Box::new(SftpFileReader { file }))
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn write_file(&self, path: &VfsPath, mut data: Box<dyn Read + Send>) -> Result<()> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let session = self.connection_manager.get_or_create_session(
                    host,
                    *port,
                    &self.credentials,
                )?;
                
                let sftp = self.connection_manager.create_sftp(&session)?;
                let mut file = sftp.open_mode(
                    Path::new(remote_path),
                    OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::TRUNCATE,
                    0o644,
                    OpenType::File,
                )?;
                
                let mut buffer = Vec::new();
                data.read_to_end(&mut buffer)?;
                file.write_all(&buffer)?;
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn create_directory(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let session = self.connection_manager.get_or_create_session(
                    host,
                    *port,
                    &self.credentials,
                )?;
                
                let sftp = self.connection_manager.create_sftp(&session)?;
                sftp.mkdir(Path::new(remote_path), 0o755)?;
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn delete(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let session = self.connection_manager.get_or_create_session(
                    host,
                    *port,
                    &self.credentials,
                )?;
                
                let sftp = self.connection_manager.create_sftp(&session)?;
                
                let stat = sftp.stat(Path::new(remote_path))?;
                if stat.is_dir() {
                    sftp.rmdir(Path::new(remote_path))?;
                } else {
                    sftp.unlink(Path::new(remote_path))?;
                }
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }

    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        match path {
            VfsPath::Sftp { host, port, username: _, path: remote_path } => {
                let session = self.connection_manager.get_or_create_session(
                    host,
                    *port,
                    &self.credentials,
                )?;
                
                let sftp = self.connection_manager.create_sftp(&session)?;
                let stat = sftp.stat(Path::new(remote_path))?;
                
                let name = Path::new(remote_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| remote_path.clone());
                
                Ok(self.convert_file_stat(stat, name, path.clone()))
            }
            _ => Err(anyhow::anyhow!("SftpProvider can only handle SFTP paths")),
        }
    }
}