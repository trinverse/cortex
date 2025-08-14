use crate::terminal::{ShellConfig, TerminalEmulator, TerminalEvent, TerminalSize};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub id: String,
    pub title: String,
    pub working_dir: PathBuf,
    pub rows: u16,
    pub cols: u16,
}

pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalEmulator>>>,
    session_info: Arc<Mutex<HashMap<String, TerminalSession>>>,
    next_id: Arc<Mutex<usize>>,
    default_shell: ShellConfig,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_info: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            default_shell: ShellConfig::default(),
        }
    }
    
    pub fn with_default_shell(mut self, shell: ShellConfig) -> Self {
        self.default_shell = shell;
        self
    }
    
    pub fn create_session(
        &self,
        title: Option<String>,
        working_dir: Option<PathBuf>,
        size: Option<TerminalSize>,
    ) -> Result<(String, mpsc::UnboundedReceiver<TerminalEvent>)> {
        let mut id_guard = self.next_id.lock().unwrap();
        let session_id = format!("term_{}", *id_guard);
        *id_guard += 1;
        drop(id_guard);
        
        let title = title.unwrap_or_else(|| format!("Terminal {}", session_id));
        let working_dir = working_dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let size = size.unwrap_or(TerminalSize { rows: 24, cols: 80 });
        
        // Create terminal emulator
        let mut emulator = TerminalEmulator::new(size.rows, size.cols)?;
        
        // Configure shell with working directory
        let shell_config = self.default_shell.clone().with_working_dir(working_dir.clone());
        
        // Spawn shell
        emulator.spawn_shell(Some(shell_config.build_command()))?;
        
        // Get event receiver
        let event_rx = emulator.take_event_receiver()
            .ok_or_else(|| anyhow::anyhow!("Failed to get event receiver"))?;
        
        // Store session info
        let session = TerminalSession {
            id: session_id.clone(),
            title,
            working_dir,
            rows: size.rows,
            cols: size.cols,
        };
        
        self.session_info.lock().unwrap().insert(session_id.clone(), session);
        self.sessions.lock().unwrap().insert(session_id.clone(), emulator);
        
        Ok((session_id, event_rx))
    }
    
    pub fn write_to_session(&self, session_id: &str, data: &[u8]) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(emulator) = sessions.get_mut(session_id) {
            emulator.write(data)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    pub fn resize_session(&self, session_id: &str, rows: u16, cols: u16) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(emulator) = sessions.get_mut(session_id) {
            emulator.resize(rows, cols)?;
            
            // Update session info
            if let Some(info) = self.session_info.lock().unwrap().get_mut(session_id) {
                info.rows = rows;
                info.cols = cols;
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    pub fn get_screen_content(&self, session_id: &str) -> Result<Vec<String>> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(emulator) = sessions.get(session_id) {
            emulator.get_screen_content()
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    pub fn get_cursor_position(&self, session_id: &str) -> Result<(u16, u16)> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(emulator) = sessions.get(session_id) {
            emulator.get_cursor_position()
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    pub fn list_sessions(&self) -> Vec<TerminalSession> {
        self.session_info.lock().unwrap()
            .values()
            .cloned()
            .collect()
    }
    
    pub fn get_session_info(&self, session_id: &str) -> Option<TerminalSession> {
        self.session_info.lock().unwrap().get(session_id).cloned()
    }
    
    pub fn close_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(mut emulator) = sessions.remove(session_id) {
            emulator.kill()?;
            self.session_info.lock().unwrap().remove(session_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
    
    pub fn close_all_sessions(&self) -> Result<()> {
        let session_ids: Vec<String> = self.session_info.lock().unwrap()
            .keys()
            .cloned()
            .collect();
        
        for session_id in session_ids {
            let _ = self.close_session(&session_id);
        }
        
        Ok(())
    }
    
    pub fn is_session_running(&self, session_id: &str) -> bool {
        self.sessions.lock().unwrap()
            .get(session_id)
            .map(|e| e.is_running())
            .unwrap_or(false)
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        let _ = self.close_all_sessions();
    }
}