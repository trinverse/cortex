use crate::ClipboardOperations;
use anyhow::{Context, Result};
use std::path::Path;
use x11_clipboard::Clipboard as X11Clipboard;

pub struct LinuxClipboard {
    clipboard: Option<X11Clipboard>,
}

impl LinuxClipboard {
    pub fn new() -> Self {
        let clipboard = X11Clipboard::new().ok();
        if clipboard.is_none() {
            log::warn!("Failed to initialize X11 clipboard - clipboard operations will fail");
        }
        Self { clipboard }
    }
}

impl Default for LinuxClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardOperations for LinuxClipboard {
    fn copy_text(&self, text: &str) -> Result<()> {
        let clipboard = self
            .clipboard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("X11 clipboard not available"))?;

        clipboard
            .store(
                clipboard.setter.atoms.clipboard,
                clipboard.setter.atoms.utf8_string,
                text.as_bytes(),
            )
            .context("Failed to copy text to clipboard")?;

        Ok(())
    }

    fn paste_text(&self) -> Result<String> {
        let clipboard = self
            .clipboard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("X11 clipboard not available"))?;

        let result = clipboard
            .load(
                clipboard.getter.atoms.clipboard,
                clipboard.getter.atoms.utf8_string,
                clipboard.getter.atoms.property,
                std::time::Duration::from_secs(3),
            )
            .context("Failed to paste text from clipboard")?;

        String::from_utf8(result).context("Invalid UTF-8 in clipboard")
    }

    fn copy_files(&self, paths: &[&Path]) -> Result<()> {
        // Create a file list in URI format for clipboard
        let uri_list = paths
            .iter()
            .map(|p| format!("file://{}", p.display()))
            .collect::<Vec<_>>()
            .join("\n");

        let clipboard = self
            .clipboard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("X11 clipboard not available"))?;

        // Store as URI list
        clipboard
            .store(
                clipboard.setter.atoms.clipboard,
                clipboard.setter.atoms.targets,
                uri_list.as_bytes(),
            )
            .context("Failed to copy files to clipboard")?;

        Ok(())
    }

    fn paste_files(&self) -> Result<Vec<String>> {
        let clipboard = self
            .clipboard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("X11 clipboard not available"))?;

        let result = clipboard
            .load(
                clipboard.getter.atoms.clipboard,
                clipboard.getter.atoms.targets,
                clipboard.getter.atoms.property,
                std::time::Duration::from_secs(3),
            )
            .context("Failed to paste files from clipboard")?;

        let uri_list = String::from_utf8(result).context("Invalid UTF-8 in clipboard")?;

        // Parse file URIs
        let files: Vec<String> = uri_list
            .lines()
            .filter_map(|line| line.strip_prefix("file://").map(|s| s.to_string()))
            .collect();

        Ok(files)
    }

    fn has_content(&self) -> bool {
        self.clipboard.is_some()
    }
}
