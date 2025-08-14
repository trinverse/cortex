use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowMode {
    /// Run in existing terminal (traditional TUI mode)
    Terminal,
    /// Create a new native window
    Windowed,
    /// Create a fullscreen window
    Fullscreen,
}

pub trait WindowBackend: Send + Sync {
    fn create_window(&mut self, title: &str, width: u32, height: u32) -> Result<()>;
    fn run_event_loop(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn request_redraw(&self);
}

/// Detects the best window mode based on environment
pub fn detect_window_mode() -> WindowMode {
    // Check if we're in a SSH session
    if std::env::var("SSH_CLIENT").is_ok() || std::env::var("SSH_TTY").is_ok() {
        return WindowMode::Terminal;
    }
    
    // Check if we're in a container
    if std::path::Path::new("/.dockerenv").exists() {
        return WindowMode::Terminal;
    }
    
    // Check if display is available (X11/Wayland on Linux)
    #[cfg(target_os = "linux")]
    {
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            return WindowMode::Terminal;
        }
    }
    
    // Check if we're in a terminal emulator that doesn't support GUI
    if let Ok(term) = std::env::var("TERM") {
        if term == "linux" || term == "dumb" {
            return WindowMode::Terminal;
        }
    }
    
    // Default to windowed mode if GUI is available
    WindowMode::Windowed
}

/// Check if the system supports creating windows
pub fn can_create_window() -> bool {
    match detect_window_mode() {
        WindowMode::Terminal => false,
        _ => true,
    }
}