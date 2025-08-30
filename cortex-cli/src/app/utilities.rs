use anyhow::Result;
use cortex_core::{PanelState, FileMonitorEvent, OperationProgress, SearchProgress};
use std::path::PathBuf;

use super::App;

impl App {
    /// Refresh a panel using the directory cache
    pub fn refresh_panel_with_cache(&mut self, panel: &mut PanelState) -> Result<()> {
        use cortex_core::FileSystem;
        
        // Use the global configuration setting for show_hidden
        let config = self.state.config_manager.get();
        let entries = FileSystem::list_directory(&panel.current_dir, config.general.show_hidden)?;
        
        // Update panel entries
        panel.entries = entries;
        panel.selected_index = panel.selected_index.min(panel.entries.len().saturating_sub(1));
        
        // Clear marked files that no longer exist
        panel.marked_files.retain(|path| {
            panel.entries.iter().any(|entry| &entry.path == path)
        });
        
        Ok(())
    }

    /// Refresh a specific panel by reference (for convenience)
    #[allow(dead_code)] // TODO: Use this method when panel-specific refresh is needed
    pub fn refresh_panel(&mut self, panel: &mut PanelState) -> Result<()> {
        self.refresh_panel_with_cache(panel)
    }

    /// Handle operation progress updates
    pub fn handle_operation_progress(&mut self, progress: OperationProgress) {
        match progress {
            OperationProgress::Started { operation } => {
                self.state.set_status_message(format!("Started: {}", operation));
            }
            OperationProgress::Progress { current, total, message } => {
                let percentage = if total > 0 { (current * 100) / total } else { 0 };
                self.state.set_status_message(format!(
                    "{}: {}% ({}/{})", message, percentage, current, total
                ));
            }
            OperationProgress::Completed { operation } => {
                self.state.set_status_message(format!("Completed: {}", operation));
                self.refresh_needed = true;
            }
            OperationProgress::Failed { operation, error } => {
                self.state.set_status_message(format!("Error in {}: {}", operation, error));
            }
        }
    }

    /// Handle search progress updates
    pub fn handle_search_progress(&mut self, progress: SearchProgress) {
        match progress {
            SearchProgress::Started { total_dirs } => {
                self.state.set_status_message(format!("Search started... {} directories", total_dirs));
            }
            SearchProgress::Searching { current_path, searched, total } => {
                self.state.set_status_message(format!(
                    "Searching... {}: {}/{}", current_path.display(), searched, total
                ));
            }
            SearchProgress::Found { result } => {
                self.state.set_status_message(format!(
                    "Found: {}", result.path.display()
                ));
            }
            SearchProgress::Completed { total_found, elapsed_ms } => {
                self.state.set_status_message(format!(
                    "Search completed. Found {} results in {}ms.", total_found, elapsed_ms
                ));
                // TODO: Display search results in UI
            }
            SearchProgress::Error { path, error } => {
                self.state.set_status_message(format!("Search error at {}: {}", path.display(), error));
            }
        }
    }

    /// Handle file system events
    pub fn handle_file_event(&mut self, event: FileMonitorEvent) {
        match event {
            FileMonitorEvent::Created(path) => {
                self.notification_manager.add_notification(
                    "File Created",
                    format!("Created: {}", path.display()),
                    cortex_tui::NotificationType::Info
                );
                self.refresh_needed = true;
            }
            FileMonitorEvent::Modified(path) => {
                self.notification_manager.add_notification(
                    "File Modified",
                    format!("Modified: {}", path.display()),
                    cortex_tui::NotificationType::Info
                );
            }
            FileMonitorEvent::Deleted(path) => {
                self.notification_manager.add_notification(
                    "File Deleted",
                    format!("Deleted: {}", path.display()),
                    cortex_tui::NotificationType::Warning
                );
                self.refresh_needed = true;
            }
            FileMonitorEvent::Renamed { from, to } => {
                self.notification_manager.add_notification(
                    "File Renamed",
                    format!("Renamed: {} -> {}", from.display(), to.display()),
                    cortex_tui::NotificationType::Info
                );
                self.refresh_needed = true;
            }
        }
    }

    /// Handle command output from background processes
    pub fn handle_command_output(&mut self, output: String) {
        // Display command output in the status line or a dedicated area
        self.state.set_status_message(format!("Command: {}", output.trim()));
    }

    /// Navigate to a specific directory in the active panel
    pub fn navigate_to_directory(&mut self, path: PathBuf) -> Result<()> {
        if path.is_dir() {
            {
                let active_panel = self.state.active_panel_mut();
                active_panel.current_dir = path;
                active_panel.selected_index = 0;
            }
            // Refresh after updating the path
            let mut active_panel = self.state.active_panel().clone();
            self.refresh_panel_with_cache(&mut active_panel)?;
            
            // Update the actual panel with the refreshed data
            *self.state.active_panel_mut() = active_panel;
        }
        Ok(())
    }

    /// Execute a file operation (copy, move, delete)
    pub async fn execute_operation(&mut self, operation: cortex_core::FileOperation) -> Result<()> {
        // Use the operation manager to execute the operation
        match operation {
            cortex_core::FileOperation::Copy { sources, destination } => {
                self.operation_manager.copy_files(sources, destination).await?;
            }
            cortex_core::FileOperation::Move { sources, destination } => {
                self.operation_manager.move_files(sources, destination).await?;
            }
            cortex_core::FileOperation::Delete { targets } => {
                self.operation_manager.delete_files(targets).await?;
            }
            cortex_core::FileOperation::CreateDir { path } => {
                std::fs::create_dir_all(path)?;
            }
            cortex_core::FileOperation::DeleteToTrash { targets: _ } => {
                // TODO: Implement trash deletion
                self.state.set_status_message("Trash deletion not yet implemented");
            }
            cortex_core::FileOperation::RestoreFromTrash { targets: _ } => {
                // TODO: Implement trash restoration
                self.state.set_status_message("Trash restoration not yet implemented");
            }
            cortex_core::FileOperation::Rename { old_path: _, new_name: _ } => {
                // TODO: Implement rename operation
                self.state.set_status_message("Rename operation not yet implemented");
            }
            cortex_core::FileOperation::CopyToClipboard { paths: _ } => {
                // TODO: Implement clipboard copy
                self.state.set_status_message("Clipboard copy not yet implemented");
            }
            cortex_core::FileOperation::PasteFromClipboard { destination: _ } => {
                // TODO: Implement clipboard paste
                self.state.set_status_message("Clipboard paste not yet implemented");
            }
        }
        
        // Refresh panels after operation
        self.refresh_needed = true;
        
        Ok(())
    }

    /// Connect to an SFTP server
    #[allow(dead_code)] // TODO: Implement when SFTP functionality is fully integrated
    pub async fn connect_sftp(
        &mut self,
        _credentials: &cortex_core::RemoteCredentials,
        _vfs_path: &cortex_core::VfsPath,
    ) -> Result<()> {
        // TODO: Implement SFTP connection when VFS is properly integrated
        self.state.set_status_message("SFTP connection not yet implemented");
        Ok(())
    }

    /// Get the currently selected file or directory
    #[allow(dead_code)] // TODO: Use this when implementing file operations UI
    pub fn get_selected_entry(&self) -> Option<&cortex_core::FileEntry> {
        let active_panel = self.state.active_panel();
        active_panel.current_entry()
    }

    /// Toggle marking of the currently selected file
    #[allow(dead_code)] // TODO: Use this for file marking functionality
    pub fn toggle_mark_current(&mut self) {
        if let Some(entry) = self.get_selected_entry() {
            let path = entry.path.clone();
            let active_panel = self.state.active_panel_mut();
            
            if active_panel.marked_files.contains(&path) {
                active_panel.marked_files.retain(|p| p != &path);
            } else {
                active_panel.marked_files.push(path);
            }
        }
    }

    /// Mark all files in the current directory
    pub fn mark_all_files(&mut self) {
        let active_panel = self.state.active_panel_mut();
        for entry in &active_panel.entries {
            if entry.name != ".." {
                active_panel.marked_files.push(entry.path.clone());
            }
        }
    }

    /// Unmark all files
    pub fn unmark_all_files(&mut self) {
        let active_panel = self.state.active_panel_mut();
        active_panel.marked_files.clear();
    }

    /// Get status message for display
    #[allow(dead_code)] // TODO: Use this when implementing status bar UI
    pub fn get_status_message(&self) -> Option<&str> {
        self.state.status_message.as_deref()
    }

    /// Set a status message
    #[allow(dead_code)] // TODO: Remove this allow when status setting is used more widely
    pub fn set_status_message(&mut self, message: &str) {
        self.state.set_status_message(message);
    }
}
