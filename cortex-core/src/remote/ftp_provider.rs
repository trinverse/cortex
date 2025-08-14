use crate::vfs::{VfsEntry, VfsEntryType, VfsPath, VfsProvider};
use anyhow::Result;
use std::io::Read;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use suppaftp::AsyncFtpStream;
use tokio::runtime::Runtime;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FtpCredentials {
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

struct FtpConnectionInfo {
    stream: Arc<Mutex<AsyncFtpStream>>,
    last_used: Instant,
    #[allow(dead_code)]
    credentials: FtpCredentials,
}

pub struct FtpProvider {
    connections: Arc<RwLock<HashMap<String, FtpConnectionInfo>>>,
    runtime: Arc<Runtime>,
    connection_timeout: Duration,
    idle_timeout: Duration,
    max_retries: u32,
}

impl Default for FtpProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FtpProvider {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            runtime: Arc::new(runtime),
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_retries: 3,
        }
    }

    pub fn with_credentials(self, _credentials: FtpCredentials) -> Self {
        self
    }

    pub fn with_timeouts(mut self, connection_timeout: Duration, idle_timeout: Duration) -> Self {
        self.connection_timeout = connection_timeout;
        self.idle_timeout = idle_timeout;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    fn get_or_create_connection(
        &self,
        host: &str,
        port: u16,
        credentials: &FtpCredentials,
    ) -> Result<Arc<Mutex<AsyncFtpStream>>> {
        let connection_key = format!("{}:{}@{}", credentials.username, port, host);
        
        self.cleanup_idle_connections()?;
        
        {
            let connections = self.connections.read().unwrap();
            if let Some(conn_info) = connections.get(&connection_key) {
                return Ok(conn_info.stream.clone());
            }
        }
        
        let mut last_error = None;
        for attempt in 0..self.max_retries {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(500 * (1 << attempt)));
            }
            
            match self.create_new_connection(host, port, credentials) {
                Ok(stream) => {
                    let stream_arc = Arc::new(Mutex::new(stream));
                    let mut connections = self.connections.write().unwrap();
                    connections.insert(
                        connection_key,
                        FtpConnectionInfo {
                            stream: stream_arc.clone(),
                            last_used: Instant::now(),
                            credentials: credentials.clone(),
                        },
                    );
                    return Ok(stream_arc);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to create FTP connection")))
    }

    fn create_new_connection(
        &self,
        host: &str,
        port: u16,
        credentials: &FtpCredentials,
    ) -> Result<AsyncFtpStream> {
        use std::net::{SocketAddr, ToSocketAddrs};
        
        let addr = format!("{}:{}", host, port);
        let socket_addr: SocketAddr = addr.to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve address"))?;
        
        self.runtime.block_on(async {
            let mut stream = AsyncFtpStream::connect_timeout(
                socket_addr,
                self.connection_timeout,
            ).await?;
            
            // TLS support temporarily disabled due to library compatibility issues
            // TODO: Re-enable when suppaftp TLS API stabilizes
            if credentials.use_tls {
                return Err(anyhow::anyhow!("TLS support is temporarily disabled"));
            }
            
            stream.login(&credentials.username, &credentials.password).await?;
            stream.transfer_type(suppaftp::types::FileType::Binary).await?;
            
            Ok::<AsyncFtpStream, anyhow::Error>(stream)
        })
    }

    fn cleanup_idle_connections(&self) -> Result<()> {
        let mut connections = self.connections.write().unwrap();
        let now = Instant::now();
        
        connections.retain(|_, info| {
            if now.duration_since(info.last_used) > self.idle_timeout {
                if let Ok(mut stream) = info.stream.lock() {
                    let _ = self.runtime.block_on(async {
                        stream.quit().await
                    });
                }
                false
            } else {
                true
            }
        });
        
        Ok(())
    }

    fn parse_list_entry(&self, line: &str, base_path: &VfsPath) -> Option<VfsEntry> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            return None;
        }
        
        let permissions = parts[0];
        let size: u64 = parts[4].parse().unwrap_or(0);
        let name = parts[8..].join(" ");
        
        let entry_type = if permissions.starts_with('d') {
            VfsEntryType::Directory
        } else if permissions.starts_with('l') {
            VfsEntryType::Symlink
        } else {
            VfsEntryType::File
        };
        
        let path = match base_path {
            VfsPath::Ftp { host, port, username, path: base } => {
                let full_path = if base.ends_with('/') {
                    format!("{}{}", base, name)
                } else {
                    format!("{}/{}", base, name)
                };
                VfsPath::Ftp {
                    host: host.clone(),
                    port: *port,
                    username: username.clone(),
                    path: full_path,
                }
            }
            _ => return None,
        };
        
        Some(VfsEntry {
            name,
            path,
            entry_type,
            size,
            modified: SystemTime::now(),
            permissions: permissions[1..10].to_string(),
            compressed_size: None,
        })
    }
}

impl VfsProvider for FtpProvider {
    fn can_handle(&self, path: &VfsPath) -> bool {
        matches!(path, VfsPath::Ftp { .. })
    }

    fn list_entries(&self, path: &VfsPath) -> Result<Vec<VfsEntry>> {
        match path {
            VfsPath::Ftp { host, port, username, path: remote_path } => {
                let credentials = FtpCredentials {
                    username: username.clone(),
                    password: String::new(),
                    use_tls: false,
                };
                
                let stream = self.get_or_create_connection(host, *port, &credentials)?;
                
                let list_result = self.runtime.block_on(async {
                    let mut stream = stream.lock().unwrap();
                    stream.list(Some(remote_path)).await
                })?;
                
                let entries: Vec<VfsEntry> = list_result
                    .into_iter()
                    .filter_map(|line| self.parse_list_entry(&line, path))
                    .collect();
                
                Ok(entries)
            }
            _ => Err(anyhow::anyhow!("FtpProvider can only handle FTP paths")),
        }
    }

    fn read_file(&self, path: &VfsPath) -> Result<Box<dyn Read + Send>> {
        match path {
            VfsPath::Ftp { host, port, username, path: remote_path } => {
                let credentials = FtpCredentials {
                    username: username.clone(),
                    password: String::new(),
                    use_tls: false,
                };
                
                let stream = self.get_or_create_connection(host, *port, &credentials)?;
                
                let data = self.runtime.block_on(async {
                    let mut stream = stream.lock().unwrap();
                    let mut cursor = stream.retr_as_stream(remote_path).await?;
                    let mut buffer = Vec::new();
                    futures_util::AsyncReadExt::read_to_end(&mut cursor, &mut buffer).await?;
                    Ok::<Vec<u8>, anyhow::Error>(buffer)
                })?;
                
                Ok(Box::new(std::io::Cursor::new(data)))
            }
            _ => Err(anyhow::anyhow!("FtpProvider can only handle FTP paths")),
        }
    }

    fn write_file(&self, path: &VfsPath, mut data: Box<dyn Read + Send>) -> Result<()> {
        match path {
            VfsPath::Ftp { host, port, username, path: remote_path } => {
                let credentials = FtpCredentials {
                    username: username.clone(),
                    password: String::new(),
                    use_tls: false,
                };
                
                let stream = self.get_or_create_connection(host, *port, &credentials)?;
                
                let mut buffer = Vec::new();
                data.read_to_end(&mut buffer)?;
                
                self.runtime.block_on(async {
                    let mut stream = stream.lock().unwrap();
                    let mut cursor = futures_util::io::Cursor::new(buffer);
                    stream.put_file(remote_path, &mut cursor).await
                })?;
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("FtpProvider can only handle FTP paths")),
        }
    }

    fn create_directory(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Ftp { host, port, username, path: remote_path } => {
                let credentials = FtpCredentials {
                    username: username.clone(),
                    password: String::new(),
                    use_tls: false,
                };
                
                let stream = self.get_or_create_connection(host, *port, &credentials)?;
                
                self.runtime.block_on(async {
                    let mut stream = stream.lock().unwrap();
                    stream.mkdir(remote_path).await
                })?;
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("FtpProvider can only handle FTP paths")),
        }
    }

    fn delete(&self, path: &VfsPath) -> Result<()> {
        match path {
            VfsPath::Ftp { host, port, username, path: remote_path } => {
                let credentials = FtpCredentials {
                    username: username.clone(),
                    password: String::new(),
                    use_tls: false,
                };
                
                let stream = self.get_or_create_connection(host, *port, &credentials)?;
                
                let result = self.runtime.block_on(async {
                    let mut stream = stream.lock().unwrap();
                    
                    match stream.rm(remote_path).await {
                        Ok(_) => Ok(()),
                        Err(_) => stream.rmdir(remote_path).await,
                    }
                });
                
                result?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("FtpProvider can only handle FTP paths")),
        }
    }

    fn get_info(&self, path: &VfsPath) -> Result<VfsEntry> {
        match path {
            VfsPath::Ftp { host, port, username, path: remote_path } => {
                let credentials = FtpCredentials {
                    username: username.clone(),
                    password: String::new(),
                    use_tls: false,
                };
                
                let stream = self.get_or_create_connection(host, *port, &credentials)?;
                
                let size = self.runtime.block_on(async {
                    let mut stream = stream.lock().unwrap();
                    stream.size(remote_path).await
                }).ok();
                
                let name = std::path::Path::new(remote_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| remote_path.clone());
                
                Ok(VfsEntry {
                    name,
                    path: path.clone(),
                    entry_type: VfsEntryType::File,
                    size: size.unwrap_or(0) as u64,
                    modified: SystemTime::now(),
                    permissions: String::new(),
                    compressed_size: None,
                })
            }
            _ => Err(anyhow::anyhow!("FtpProvider can only handle FTP paths")),
        }
    }
}