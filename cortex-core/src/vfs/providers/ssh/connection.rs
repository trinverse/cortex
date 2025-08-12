use anyhow::Result;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[cfg(feature = "ssh")]
use ssh2::Session;

use crate::vfs::types::RemoteCredentials;

#[cfg(feature = "ssh")]
pub struct SshConnectionManager {
    sessions: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<Session>>>>>,
}

#[cfg(feature = "ssh")]
impl SshConnectionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn get_or_create_session(
        &self,
        host: &str,
        port: u16,
        credentials: &RemoteCredentials,
    ) -> Result<Arc<Mutex<Session>>> {
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