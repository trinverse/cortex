use anyhow::Result;
use ssh2::{Session, Sftp};
use std::collections::HashMap;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RemoteCredentials {
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub passphrase: Option<String>,
}

struct SessionInfo {
    session: Arc<Mutex<Session>>,
    last_used: Instant,
    #[allow(dead_code)]
    credentials: RemoteCredentials,
}

pub struct SshConnectionManager {
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    connection_timeout: Duration,
    idle_timeout: Duration,
    max_retries: u32,
}

impl Default for SshConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SshConnectionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_retries: 3,
        }
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

    pub fn get_or_create_session(
        &self,
        host: &str,
        port: u16,
        credentials: &RemoteCredentials,
    ) -> Result<Arc<Mutex<Session>>> {
        let session_key = format!("{}:{}@{}", credentials.username, port, host);
        
        self.cleanup_idle_sessions()?;
        
        {
            let sessions = self.sessions.read().unwrap();
            if let Some(session_info) = sessions.get(&session_key) {
                if self.is_session_valid(&session_info.session)? {
                    let mut sessions = self.sessions.write().unwrap();
                    if let Some(info) = sessions.get_mut(&session_key) {
                        info.last_used = Instant::now();
                    }
                    return Ok(session_info.session.clone());
                }
            }
        }
        
        let mut last_error = None;
        for attempt in 0..self.max_retries {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(500 * (1 << attempt)));
            }
            
            match self.create_new_session(host, port, credentials) {
                Ok(session) => {
                    let session_arc = Arc::new(Mutex::new(session));
                    let mut sessions = self.sessions.write().unwrap();
                    sessions.insert(
                        session_key,
                        SessionInfo {
                            session: session_arc.clone(),
                            last_used: Instant::now(),
                            credentials: credentials.clone(),
                        },
                    );
                    return Ok(session_arc);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to create SSH session")))
    }

    fn create_new_session(
        &self,
        host: &str,
        port: u16,
        credentials: &RemoteCredentials,
    ) -> Result<Session> {
        let tcp = TcpStream::connect_timeout(
            &format!("{}:{}", host, port).parse()?,
            self.connection_timeout,
        )?;
        tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
        
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        
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
            session.userauth_agent(&credentials.username)?;
        }
        
        if !session.authenticated() {
            return Err(anyhow::anyhow!("SSH authentication failed"));
        }
        
        session.set_keepalive(true, 30);
        
        Ok(session)
    }

    fn is_session_valid(&self, session: &Arc<Mutex<Session>>) -> Result<bool> {
        let session = session.lock().unwrap();
        Ok(session.authenticated())
    }

    fn cleanup_idle_sessions(&self) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        let now = Instant::now();
        
        sessions.retain(|_, info| {
            if now.duration_since(info.last_used) > self.idle_timeout {
                if let Ok(session) = info.session.lock() {
                    let _ = session.disconnect(None, "idle timeout", None);
                }
                false
            } else {
                true
            }
        });
        
        Ok(())
    }

    pub fn create_sftp(&self, session: &Arc<Mutex<Session>>) -> Result<Sftp> {
        let session = session.lock().unwrap();
        session.sftp().map_err(|e| anyhow::anyhow!("Failed to create SFTP channel: {}", e))
    }

    pub fn disconnect(&self, host: &str, port: u16, username: &str) -> Result<()> {
        let session_key = format!("{}:{}@{}", username, port, host);
        let mut sessions = self.sessions.write().unwrap();
        
        if let Some(info) = sessions.remove(&session_key) {
            if let Ok(session) = info.session.lock() {
                session.disconnect(None, "manual disconnect", None)?;
            }
        }
        
        Ok(())
    }

    pub fn disconnect_all(&self) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        
        for (_, info) in sessions.drain() {
            if let Ok(session) = info.session.lock() {
                let _ = session.disconnect(None, "shutdown", None);
            }
        }
        
        Ok(())
    }
}