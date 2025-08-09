pub mod clipboard;
pub mod trash;
pub mod platform;

use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
        }
    }
}

pub trait TrashOperations: Send + Sync {
    fn move_to_trash(&self, path: &Path) -> Result<()>;
    fn restore_from_trash(&self, path: &Path) -> Result<()>;
    fn empty_trash(&self) -> Result<()>;
    fn list_trash_contents(&self) -> Result<Vec<TrashItem>>;
}

pub trait ClipboardOperations: Send + Sync {
    fn copy_text(&self, text: &str) -> Result<()>;
    fn paste_text(&self) -> Result<String>;
    fn copy_files(&self, paths: &[&Path]) -> Result<()>;
    fn paste_files(&self) -> Result<Vec<String>>;
    fn has_content(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct TrashItem {
    pub original_path: String,
    pub trash_path: String,
    pub deletion_date: chrono::DateTime<chrono::Local>,
    pub size: u64,
}

pub fn get_trash_handler() -> Box<dyn TrashOperations> {
    #[cfg(target_os = "windows")]
    return Box::new(trash::windows::WindowsTrash::new());
    
    #[cfg(target_os = "macos")]
    return Box::new(trash::macos::MacOSTrash::new());
    
    #[cfg(target_os = "linux")]
    return Box::new(trash::linux::LinuxTrash::new());
}

pub fn get_clipboard_handler() -> Box<dyn ClipboardOperations> {
    #[cfg(target_os = "windows")]
    return Box::new(clipboard::windows::WindowsClipboard::new());
    
    #[cfg(target_os = "macos")]
    return Box::new(clipboard::macos::MacOSClipboard::new());
    
    #[cfg(target_os = "linux")]
    return Box::new(clipboard::linux::LinuxClipboard::new());
}