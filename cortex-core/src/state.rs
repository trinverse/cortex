use crate::fs::FileEntry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub view_offset: usize,
    pub show_hidden: bool,
    pub sort_mode: SortMode,
    pub marked_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SortMode {
    Name,
    Size,
    Modified,
    Extension,
}

impl PanelState {
    pub fn new(path: PathBuf) -> Self {
        Self {
            current_dir: path,
            entries: Vec::new(),
            selected_index: 0,
            view_offset: 0,
            show_hidden: false,
            sort_mode: SortMode::Name,
            marked_files: Vec::new(),
        }
    }

    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn current_entry_mut(&mut self) -> Option<&mut FileEntry> {
        self.entries.get_mut(self.selected_index)
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_selection_page_up(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
    }

    pub fn move_selection_page_down(&mut self, page_size: usize) {
        let max_index = self.entries.len().saturating_sub(1);
        self.selected_index = (self.selected_index + page_size).min(max_index);
    }

    pub fn move_selection_home(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_selection_end(&mut self) {
        self.selected_index = self.entries.len().saturating_sub(1);
    }

    pub fn toggle_mark_current(&mut self) {
        if let Some(entry) = self.current_entry() {
            let path = entry.path.clone();
            if let Some(pos) = self.marked_files.iter().position(|p| p == &path) {
                self.marked_files.remove(pos);
            } else {
                self.marked_files.push(path);
            }
        }
    }

    pub fn is_marked(&self, path: &PathBuf) -> bool {
        self.marked_files.contains(path)
    }

    pub fn clear_marks(&mut self) {
        self.marked_files.clear();
    }

    pub fn update_view_offset(&mut self, window_height: usize) {
        let padding = 3;
        
        if self.selected_index < self.view_offset + padding {
            self.view_offset = self.selected_index.saturating_sub(padding);
        } else if self.selected_index >= self.view_offset + window_height - padding {
            self.view_offset = self.selected_index + padding - window_height + 1;
        }
    }

    pub fn sort_entries(&mut self) {
        use crate::fs::FileType;
        
        self.entries.sort_by(|a, b| {
            if a.name == ".." {
                return std::cmp::Ordering::Less;
            }
            if b.name == ".." {
                return std::cmp::Ordering::Greater;
            }

            match (&a.file_type, &b.file_type) {
                (FileType::Directory, FileType::File) => std::cmp::Ordering::Less,
                (FileType::File, FileType::Directory) => std::cmp::Ordering::Greater,
                _ => match self.sort_mode {
                    SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortMode::Size => b.size.cmp(&a.size),
                    SortMode::Modified => b.modified.cmp(&a.modified),
                    SortMode::Extension => {
                        let ext_a = a.extension.as_deref().unwrap_or("");
                        let ext_b = b.extension.as_deref().unwrap_or("");
                        ext_a.cmp(ext_b).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    }
                },
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub left_panel: PanelState,
    pub right_panel: PanelState,
    pub active_panel: ActivePanel,
    pub command_line: String,
    pub status_message: Option<String>,
    pub show_help: bool,
    pub pending_operation: Option<FileOperation>,
}

#[derive(Debug, Clone)]
pub enum FileOperation {
    Copy { sources: Vec<PathBuf>, destination: PathBuf },
    Move { sources: Vec<PathBuf>, destination: PathBuf },
    Delete { targets: Vec<PathBuf> },
    CreateDir { path: PathBuf },
    Rename { old_path: PathBuf, new_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Left,
    Right,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        
        Ok(Self {
            left_panel: PanelState::new(current_dir.clone()),
            right_panel: PanelState::new(current_dir),
            active_panel: ActivePanel::Left,
            command_line: String::new(),
            status_message: None,
            show_help: false,
            pending_operation: None,
        })
    }

    pub fn active_panel(&self) -> &PanelState {
        match self.active_panel {
            ActivePanel::Left => &self.left_panel,
            ActivePanel::Right => &self.right_panel,
        }
    }

    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        match self.active_panel {
            ActivePanel::Left => &mut self.left_panel,
            ActivePanel::Right => &mut self.right_panel,
        }
    }

    pub fn inactive_panel(&self) -> &PanelState {
        match self.active_panel {
            ActivePanel::Left => &self.right_panel,
            ActivePanel::Right => &self.left_panel,
        }
    }

    pub fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Left => ActivePanel::Right,
            ActivePanel::Right => ActivePanel::Left,
        };
    }

    pub fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }
}