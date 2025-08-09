#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

use std::path::Path;
use anyhow::Result;

pub fn get_trash_info(path: &Path) -> Result<crate::TrashItem> {
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();
    let deletion_date = chrono::Local::now();
    
    Ok(crate::TrashItem {
        original_path: path.to_string_lossy().to_string(),
        trash_path: String::new(), // Will be filled by platform implementation
        deletion_date,
        size,
    })
}