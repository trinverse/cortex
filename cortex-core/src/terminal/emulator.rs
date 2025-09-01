use anyhow::Result;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;
use vt100::Parser;

#[derive(Debug, Clone)]
pub struct TerminalSize {
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Output(Vec<u8>),
    Resize(TerminalSize),
    Exit(Option<i32>),
    Error(String),
}

pub struct TerminalEmulator {
    pty_system: Box<dyn PtySystem>,
    master: Option<Box<dyn MasterPty + Send>>,
    child: Option<Box<dyn Child + Send + Sync>>,
    parser: Arc<Mutex<Parser>>,
    size: TerminalSize,
    event_tx: mpsc::UnboundedSender<TerminalEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<TerminalEvent>>,
    reader_thread: Option<thread::JoinHandle<()>>,
}

impl TerminalEmulator {
    pub fn new(rows: u16, cols: u16) -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        let parser = Parser::new(rows, cols, 1000);
        
        Ok(Self {
            pty_system,
            master: None,
            child: None,
            parser: Arc::new(Mutex::new(parser)),
            size: TerminalSize { rows, cols },
            event_tx,
            event_rx: Some(event_rx),
            reader_thread: None,
        })
    }
    
    pub fn spawn_shell(&mut self, shell_cmd: Option<String>) -> Result<()> {
        let pty_size = PtySize {
            rows: self.size.rows,
            cols: self.size.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        let pair = self.pty_system.openpty(pty_size)?;
        
        let mut cmd = if let Some(shell) = shell_cmd {
            CommandBuilder::new(shell)
        } else if cfg!(windows) {
            CommandBuilder::new("cmd.exe")
        } else {
            // Try to get user's preferred shell
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            CommandBuilder::new(shell)
        };
        
        // Set environment variables
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        
        let child = pair.slave.spawn_command(cmd)?;
        
        self.master = Some(pair.master);
        self.child = Some(child);
        
        // Start reader thread
        self.start_reader_thread()?;
        
        Ok(())
    }
    
    fn start_reader_thread(&mut self) -> Result<()> {
        let mut reader = self.master.as_mut()
            .ok_or_else(|| anyhow::anyhow!("No master PTY"))?
            .try_clone_reader()?;
        
        let event_tx = self.event_tx.clone();
        let parser = self.parser.clone();
        
        let handle = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF
                        let _ = event_tx.send(TerminalEvent::Exit(None));
                        break;
                    }
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        
                        // Update parser
                        if let Ok(mut p) = parser.lock() {
                            p.process(&data);
                        }
                        
                        // Send output event
                        let _ = event_tx.send(TerminalEvent::Output(data));
                    }
                    Err(e) => {
                        let _ = event_tx.send(TerminalEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
        });
        
        self.reader_thread = Some(handle);
        Ok(())
    }
    
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut master) = self.master {
            let mut writer = master.take_writer()?;
            writer.write_all(data)?;
        }
        Ok(())
    }
    
    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<()> {
        self.size = TerminalSize { rows, cols };
        
        let pty_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        if let Some(ref mut master) = self.master {
            master.resize(pty_size)?;
        }
        
        // Create a new parser with the new size (vt100 0.16+ doesn't have set_size)
        let new_parser = Parser::new(rows, cols, 1000);
        if let Ok(mut parser) = self.parser.lock() {
            *parser = new_parser;
        }
        
        self.event_tx.send(TerminalEvent::Resize(self.size.clone()))?;
        
        Ok(())
    }
    
    pub fn get_screen_content(&self) -> Result<Vec<String>> {
        let parser = self.parser.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let screen = parser.screen();
        
        let mut lines = Vec::new();
        for row in 0..screen.size().0 {
            let mut line = String::new();
            for col in 0..screen.size().1 {
                let cell = &screen.cell(row, col).unwrap();
                line.push_str(&cell.contents());
            }
            lines.push(line.trim_end().to_string());
        }
        
        Ok(lines)
    }
    
    pub fn get_cursor_position(&self) -> Result<(u16, u16)> {
        let parser = self.parser.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let screen = parser.screen();
        Ok((screen.cursor_position().0, screen.cursor_position().1))
    }
    
    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<TerminalEvent>> {
        self.event_rx.take()
    }
    
    pub fn is_running(&self) -> bool {
        // For now, just check if child exists
        // A more robust check would require refactoring the child storage
        self.child.is_some()
    }
    
    pub fn kill(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
        }
        Ok(())
    }
}

impl Drop for TerminalEmulator {
    fn drop(&mut self) {
        let _ = self.kill();
        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }
    }
}