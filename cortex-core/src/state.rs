use crate::cache::{CacheConfig, CacheRefresher, DirectoryCache};
use crate::config::ConfigManager;
use crate::file_monitor::{ChangeNotification, EventCallback, FileMonitorManager};
use crate::fs::FileEntry;
use crate::git::GitInfo;
use crate::vfs::{RemoteCredentials, VfsEntry, VfsPath, VirtualFileSystem};
use anyhow::Result;
use cortex_plugins::{PluginContext, PluginManager};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    pub current_dir: PathBuf,
    pub current_vfs_path: Option<VfsPath>,
    pub entries: Vec<FileEntry>,
    pub vfs_entries: Vec<VfsEntry>,
    pub filtered_entries: Vec<FileEntry>,
    pub filtered_vfs_entries: Vec<VfsEntry>,
    pub selected_index: usize,
    pub view_offset: usize,
    pub show_hidden: bool,
    pub sort_mode: SortMode,
    pub marked_files: Vec<PathBuf>,
    pub filter: Option<String>,
    #[serde(skip)]
    pub git_info: Option<GitInfo>,
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
        let git_info = crate::git::get_git_info(&path);
        Self {
            current_dir: path,
            current_vfs_path: None,
            entries: Vec::new(),
            vfs_entries: Vec::new(),
            filtered_entries: Vec::new(),
            filtered_vfs_entries: Vec::new(),
            selected_index: 0,
            view_offset: 0,
            show_hidden: false,
            sort_mode: SortMode::Name,
            marked_files: Vec::new(),
            filter: None,
            git_info,
        }
    }

    pub fn current_entry(&self) -> Option<&FileEntry> {
        let entries = if self.filter.is_some() {
            &self.filtered_entries
        } else {
            &self.entries
        };
        entries.get(self.selected_index)
    }

    pub fn current_vfs_entry(&self) -> Option<&VfsEntry> {
        let entries = if self.filter.is_some() {
            &self.filtered_vfs_entries
        } else {
            &self.vfs_entries
        };
        entries.get(self.selected_index)
    }

    pub fn is_using_vfs(&self) -> bool {
        self.current_vfs_path.is_some()
    }

    pub fn current_entry_mut(&mut self) -> Option<&mut FileEntry> {
        if self.filter.is_some() {
            self.filtered_entries.get_mut(self.selected_index)
        } else {
            self.entries.get_mut(self.selected_index)
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        let len = if self.filter.is_some() {
            self.filtered_entries.len()
        } else {
            self.entries.len()
        };
        if self.selected_index < len.saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn move_selection_page_up(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
    }

    pub fn move_selection_page_down(&mut self, page_size: usize) {
        let len = if self.filter.is_some() {
            self.filtered_entries.len()
        } else {
            self.entries.len()
        };
        let max_index = len.saturating_sub(1);
        self.selected_index = (self.selected_index + page_size).min(max_index);
    }

    pub fn move_selection_home(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_selection_end(&mut self) {
        let len = if self.filter.is_some() {
            self.filtered_entries.len()
        } else {
            self.entries.len()
        };
        self.selected_index = len.saturating_sub(1);
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

    pub fn apply_filter(&mut self, filter: &str) {
        if filter.is_empty() {
            self.filter = None;
            self.filtered_entries.clear();
            self.filtered_vfs_entries.clear();
        } else {
            self.filter = Some(filter.to_string());
            self.filtered_entries = self
                .entries
                .iter()
                .filter(|entry| entry.name.to_lowercase().contains(&filter.to_lowercase()))
                .cloned()
                .collect();

            self.filtered_vfs_entries = self
                .vfs_entries
                .iter()
                .filter(|entry| entry.name.to_lowercase().contains(&filter.to_lowercase()))
                .cloned()
                .collect();
        }

        // Reset selection if needed
        let len = if self.filter.is_some() {
            if self.is_using_vfs() {
                self.filtered_vfs_entries.len()
            } else {
                self.filtered_entries.len()
            }
        } else if self.is_using_vfs() {
            self.vfs_entries.len()
        } else {
            self.entries.len()
        };

        if self.selected_index >= len && len > 0 {
            self.selected_index = len - 1;
        } else if len == 0 {
            self.selected_index = 0;
        }

        self.view_offset = 0;
    }

    pub fn clear_filter(&mut self) {
        self.filter = None;
        self.filtered_entries.clear();
        self.filtered_vfs_entries.clear();
        self.selected_index = 0;
        self.view_offset = 0;
    }

    pub fn get_visible_entries(&self) -> &Vec<FileEntry> {
        if self.filter.is_some() {
            &self.filtered_entries
        } else {
            &self.entries
        }
    }

    pub fn get_visible_vfs_entries(&self) -> &Vec<VfsEntry> {
        if self.filter.is_some() {
            &self.filtered_vfs_entries
        } else {
            &self.vfs_entries
        }
    }

    pub fn sort_entries(&mut self) {
        use crate::fs::FileType;

        // Sort main entries
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
                        ext_a
                            .cmp(ext_b)
                            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    }
                },
            }
        });

        // Also sort filtered entries if filter is active
        if self.filter.is_some() {
            self.filtered_entries.sort_by(|a, b| {
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
                            ext_a
                                .cmp(ext_b)
                                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                        }
                    },
                }
            });
        }
    }
}

pub struct AppState {
    pub left_panel: PanelState,
    pub right_panel: PanelState,
    pub active_panel: ActivePanel,
    pub command_line: String,
    pub command_cursor: usize,
    pub command_history: Vec<String>,
    pub command_history_index: Option<usize>,
    pub status_message: Option<String>,
    pub show_help: bool,
    pub pending_operation: Option<FileOperation>,
    pub remote_connections: HashMap<String, RemoteCredentials>,
    pub plugin_manager: PluginManager,
    pub config_manager: ConfigManager,
    pub file_monitor: Option<Arc<FileMonitorManager>>,
    pub auto_reload_enabled: bool,
    pub directory_cache: Arc<DirectoryCache>,
    pub cache_refresher: Option<Arc<CacheRefresher>>,
    pub theme_manager: crate::ThemeManager,
    // Command execution state
    pub command_output: VecDeque<String>,
    pub command_output_visible: bool,
    pub command_running: bool,
    pub command_output_height: u16,
}

#[derive(Debug, Clone)]
pub enum FileOperation {
    Copy {
        sources: Vec<PathBuf>,
        destination: PathBuf,
    },
    Move {
        sources: Vec<PathBuf>,
        destination: PathBuf,
    },
    Delete {
        targets: Vec<PathBuf>,
    },
    DeleteToTrash {
        targets: Vec<PathBuf>,
    },
    RestoreFromTrash {
        targets: Vec<PathBuf>,
    },
    CreateDir {
        path: PathBuf,
    },
    Rename {
        old_path: PathBuf,
        new_name: String,
    },
    CopyToClipboard {
        paths: Vec<PathBuf>,
    },
    PasteFromClipboard {
        destination: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Left,
    Right,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let config_manager = ConfigManager::new()?;
        let auto_reload_enabled = config_manager.get().general.auto_reload;

        // Initialize directory cache with configuration
        let cache_config = CacheConfig {
            max_entries: 1000,
            ttl: std::time::Duration::from_secs(300),
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            enable_background_refresh: true,
            frequent_access_threshold: 5,
        };
        let directory_cache = Arc::new(DirectoryCache::with_config(cache_config));

        Ok(Self {
            left_panel: PanelState::new(current_dir.clone()),
            right_panel: PanelState::new(current_dir),
            active_panel: ActivePanel::Left,
            command_line: String::new(),
            command_cursor: 0,
            command_history: Vec::new(),
            command_history_index: None,
            status_message: None,
            show_help: false,
            pending_operation: None,
            remote_connections: HashMap::new(),
            plugin_manager: PluginManager::new(),
            config_manager,
            file_monitor: None,
            auto_reload_enabled,
            directory_cache,
            cache_refresher: None,
            theme_manager: crate::ThemeManager::new(crate::ThemeMode::Dark),
            command_output: VecDeque::new(),
            command_output_visible: false,
            command_running: false,
            command_output_height: 10, // Default height for command output area
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

    pub fn add_command_output(&mut self, line: String) {
        const MAX_OUTPUT_LINES: usize = 1000;
        self.command_output.push_back(line);
        if self.command_output.len() > MAX_OUTPUT_LINES {
            self.command_output.pop_front();
        }
    }

    pub fn clear_command_output(&mut self) {
        self.command_output.clear();
    }

    pub fn toggle_command_output(&mut self) {
        self.command_output_visible = !self.command_output_visible;
    }

    pub fn set_command_running(&mut self, running: bool) {
        self.command_running = running;
        if running {
            self.command_output_visible = true;
        }
    }

    /// Navigate into archive or VFS path
    pub fn navigate_into_vfs(&mut self, vfs_path: VfsPath) -> Result<()> {
        let vfs = VirtualFileSystem::new();

        // Add appropriate provider based on VFS path type
        match &vfs_path {
            VfsPath::Sftp { .. } => {
                // SSH/FTP support temporarily disabled - requires OpenSSL
                return Err(anyhow::anyhow!(
                    "SSH/SFTP connections are not available in this build"
                ));
            }
            VfsPath::Ftp { .. } => {
                // SSH/FTP support temporarily disabled - requires OpenSSL
                return Err(anyhow::anyhow!(
                    "FTP connections are not available in this build"
                ));
            }
            _ => {}
        }

        let vfs_entries = vfs.list_entries(&vfs_path)?;

        let active_panel = self.active_panel_mut();
        active_panel.current_vfs_path = Some(vfs_path.clone());
        active_panel.vfs_entries = vfs_entries;
        active_panel.filtered_vfs_entries.clear();
        active_panel.selected_index = 0;
        active_panel.view_offset = 0;

        Ok(())
    }

    /// Navigate back to regular file system from VFS
    pub fn navigate_back_from_vfs(&mut self) -> Result<()> {
        let active_panel = self.active_panel_mut();

        if let Some(vfs_path) = &active_panel.current_vfs_path {
            match vfs_path {
                VfsPath::Archive { archive_path, .. } => {
                    // Navigate back to the directory containing the archive
                    if let Some(parent) = archive_path.parent() {
                        active_panel.current_dir = parent.to_path_buf();
                    }
                    active_panel.current_vfs_path = None;
                    active_panel.vfs_entries.clear();
                    active_panel.filtered_vfs_entries.clear();
                }
                _ => {
                    // For other VFS types, go back to regular filesystem
                    active_panel.current_vfs_path = None;
                    active_panel.vfs_entries.clear();
                    active_panel.filtered_vfs_entries.clear();
                }
            }
        }

        Ok(())
    }

    /// Store connection credentials for reuse
    pub fn store_connection_credentials(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        credentials: RemoteCredentials,
    ) {
        let connection_key = format!("{}:{}@{}", username, port, host);
        self.remote_connections.insert(connection_key, credentials);
    }

    /// Check if we can navigate into the current selection
    pub fn can_navigate_into_current(&self) -> bool {
        let _vfs = VirtualFileSystem::new();
        let active_panel = self.active_panel();

        if active_panel.is_using_vfs() {
            // In VFS mode, check VFS entry
            if let Some(vfs_entry) = active_panel.current_vfs_entry() {
                // Check if it's a directory or archive
                matches!(vfs_entry.entry_type, crate::vfs::VfsEntryType::Directory)
            } else {
                false
            }
        } else {
            // In regular mode, check if it's an archive or directory
            if let Some(entry) = active_panel.current_entry() {
                entry.file_type == crate::fs::FileType::Directory ||
                // Simple archive detection by extension
                entry.path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| matches!(ext.to_lowercase().as_str(), "zip" | "tar" | "gz" | "7z" | "rar"))
                    .unwrap_or(false)
            } else {
                false
            }
        }
    }

    /// Create plugin context from current state
    pub fn create_plugin_context(&self) -> PluginContext {
        let active_panel = self.active_panel();
        let inactive_panel = self.inactive_panel();

        let current_file = if active_panel.is_using_vfs() {
            active_panel
                .current_vfs_entry()
                .map(|entry| PathBuf::from(&entry.name))
        } else {
            active_panel.current_entry().map(|entry| entry.path.clone())
        };

        let selected_files = if active_panel.is_using_vfs() {
            // For VFS, use marked entries (simplified for now)
            Vec::new()
        } else if active_panel.marked_files.is_empty() {
            // If no files marked, use current file
            current_file.clone().into_iter().collect()
        } else {
            active_panel.marked_files.clone()
        };

        PluginContext {
            current_file,
            current_directory: active_panel.current_dir.clone(),
            selected_files,
            active_panel: match self.active_panel {
                ActivePanel::Left => "left".to_string(),
                ActivePanel::Right => "right".to_string(),
            },
            other_panel_directory: inactive_panel.current_dir.clone(),
        }
    }

    /// Initialize the file monitor
    pub async fn init_file_monitor(&mut self) -> Result<()> {
        if self.auto_reload_enabled {
            let monitor_manager = Arc::new(FileMonitorManager::new().await?);
            monitor_manager.start().await?;

            // Set up callback for panel refresh
            let callback: EventCallback = Arc::new(move |notification: ChangeNotification| {
                log::debug!(
                    "File change detected: {} - {:?}",
                    notification.path.display(),
                    notification.event
                );
                // The actual panel refresh will be handled by the UI layer
            });

            monitor_manager.register_change_callback(callback).await;

            // Watch current directories
            monitor_manager
                .watch_directory(&self.left_panel.current_dir, false)
                .await?;
            monitor_manager
                .watch_directory(&self.right_panel.current_dir, false)
                .await?;

            self.file_monitor = Some(monitor_manager);
        }
        Ok(())
    }

    /// Update file monitoring when navigating to a new directory
    pub async fn update_file_monitoring(
        &mut self,
        panel: ActivePanel,
        new_path: &Path,
    ) -> Result<()> {
        if let Some(ref monitor) = self.file_monitor {
            let old_path = match panel {
                ActivePanel::Left => &self.left_panel.current_dir,
                ActivePanel::Right => &self.right_panel.current_dir,
            };

            // Unwatch old directory
            monitor.unwatch_directory(old_path).await?;

            // Watch new directory
            monitor.watch_directory(new_path, false).await?;
        }
        Ok(())
    }

    /// Check if file monitoring is active
    pub fn is_file_monitoring_active(&self) -> bool {
        self.file_monitor.is_some() && self.auto_reload_enabled
    }

    /// Toggle auto-reload functionality
    pub async fn toggle_auto_reload(&mut self) -> Result<()> {
        self.auto_reload_enabled = !self.auto_reload_enabled;

        if self.auto_reload_enabled && self.file_monitor.is_none() {
            self.init_file_monitor().await?;
        } else if !self.auto_reload_enabled && self.file_monitor.is_some() {
            if let Some(monitor) = self.file_monitor.take() {
                monitor.stop().await?;
            }
        }

        // Update config
        let auto_reload = self.auto_reload_enabled;
        self.config_manager.update(|config| {
            config.general.auto_reload = auto_reload;
        })?;

        Ok(())
    }

    /// Get watched directories for debugging/status
    pub async fn get_watched_directories(&self) -> Vec<PathBuf> {
        if let Some(ref monitor) = self.file_monitor {
            monitor.get_watched_directories().await
        } else {
            Vec::new()
        }
    }
}
