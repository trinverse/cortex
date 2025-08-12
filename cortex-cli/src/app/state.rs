// Application state management
use cortex_core::AppState;
use cortex_tui::{
    ContextMenu, Dialog, EditorDialog, EventHandler, MouseHandler, 
    MouseRegionManager, NotificationManager
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

use crate::operations::OperationManager;

/// Main application structure
pub struct App {
    pub state: AppState,
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub events: EventHandler,
    pub dialog: Option<Dialog>,
    pub pending_editor: Option<EditorDialog>,
    pub _operation_manager: OperationManager,
    pub operation_rx: Option<mpsc::UnboundedReceiver<cortex_core::OperationProgress>>,
    pub search_rx: Option<mpsc::UnboundedReceiver<cortex_core::SearchProgress>>,
    pub refresh_needed: bool,
    pub _file_change_rx: Option<mpsc::UnboundedReceiver<()>>,
    pub command_output_rx: Option<mpsc::Receiver<String>>,
    pub file_event_rx: Option<mpsc::UnboundedReceiver<cortex_core::FileMonitorEvent>>,
    pub notification_manager: NotificationManager,
    pub _mouse_handler: MouseHandler,
    pub context_menu: Option<ContextMenu>,
    pub _mouse_regions: MouseRegionManager,
    pub suggestions_dismissed: bool,
}

impl App {
    /// Get selected files from the active panel
    pub fn get_selected_files(&self) -> Vec<std::path::PathBuf> {
        let active_panel = self.state.active_panel();
        
        if !active_panel.marked_files.is_empty() {
            active_panel.marked_files.clone()
        } else if let Some(entry) = active_panel.current_entry() {
            vec![entry.path.clone()]
        } else {
            Vec::new()
        }
    }

    /// Check if refresh is needed
    pub fn _needs_refresh(&self) -> bool {
        self.refresh_needed
    }

    /// Mark that refresh is needed
    pub fn mark_refresh_needed(&mut self) {
        self.refresh_needed = true;
    }

    /// Clear refresh flag
    pub fn _clear_refresh_flag(&mut self) {
        self.refresh_needed = false;
    }
}