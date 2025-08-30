use anyhow::Result;
use cortex_core::AppState;
use cortex_tui::{
    ConfigDialog, Dialog, EditorDialog, EventHandler,
    MouseHandler, MouseRegionManager, NotificationManager, ContextMenu
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

use crate::operations::OperationManager;

// Sub-modules
mod initialization;
mod event_handling;
mod utilities;

// The modules are used internally by the App implementation

/// Main application structure containing all state and UI components
pub struct App {
    // Core state and UI
    pub state: AppState,
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub events: EventHandler,
    
    // Dialog management
    pub dialog: Option<Dialog>,
    #[allow(dead_code)] // TODO: Implement editor dialog functionality
    pub pending_editor: Option<EditorDialog>,
    #[allow(dead_code)] // TODO: Implement config dialog functionality
    pub pending_config_dialog: Option<ConfigDialog>,

    // Operation management
    pub operation_manager: OperationManager,
    pub operation_rx: Option<mpsc::UnboundedReceiver<cortex_core::OperationProgress>>,
    pub search_rx: Option<mpsc::UnboundedReceiver<cortex_core::SearchProgress>>,
    pub refresh_needed: bool,
    
    // File system monitoring
    pub file_change_rx: Option<mpsc::UnboundedReceiver<()>>,
    pub command_output_rx: Option<mpsc::Receiver<String>>,
    pub file_event_rx: Option<mpsc::UnboundedReceiver<cortex_core::FileMonitorEvent>>,
    
    // UI components
    pub notification_manager: NotificationManager,
    #[allow(dead_code)] // TODO: Implement mouse handling functionality
    pub mouse_handler: MouseHandler,
    pub context_menu: Option<ContextMenu>,
    _mouse_regions: MouseRegionManager,
    
    // State tracking
    pub suggestions_dismissed: bool,
    
    // AI integration
    pub ai_response_rx: Option<mpsc::UnboundedReceiver<(String, bool)>>,
    #[allow(dead_code)] // TODO: Implement AI response sending functionality
    pub ai_response_tx: mpsc::UnboundedSender<(String, bool)>,
    
    // Configuration
    pub config_reload_rx: std::sync::mpsc::Receiver<()>,
}

impl App {
    /// Main application run loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // Draw the UI
            self.draw_ui()?;
            
            // Process various message channels
            self.process_operation_progress();
            self.process_ai_responses();
            self.process_search_progress();
            self.process_file_changes();
            self.process_file_events();
            self.process_command_output().await;
            self.process_config_reload();

            // Handle input events
            if !self.handle_input_events().await? {
                break; // Exit requested
            }

            // Apply refresh if needed
            if self.refresh_needed {
                self.apply_refresh()?;
                self.refresh_needed = false;
            }
        }

        Ok(())
    }

    /// Draw the user interface
    fn draw_ui(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            cortex_tui::UI::draw(frame, &self.state);
            
            if let Some(ref mut dialog) = self.dialog {
                let theme = self.state.theme_manager.get_current_theme();
                cortex_tui::dialogs::render_dialog(frame, dialog, theme);
            }
            
            // Render notifications on top
            self.notification_manager.render(frame);
        })?;
        
        Ok(())
    }

    /// Process operation progress updates
    fn process_operation_progress(&mut self) {
        if let Some(rx) = &mut self.operation_rx {
            if let Ok(progress) = rx.try_recv() {
                self.handle_operation_progress(progress);
            }
        }
    }

    /// Process AI response messages
    fn process_ai_responses(&mut self) {
        if let Some(rx) = &mut self.ai_response_rx {
            if let Ok((response, _is_error)) = rx.try_recv() {
                if let Some(Dialog::AIChat(dialog)) = &mut self.dialog {
                    dialog.add_assistant_message(response);
                }
            }
        }
    }

    /// Process search progress updates
    fn process_search_progress(&mut self) {
        let search_progresses: Vec<_> = if let Some(rx) = &mut self.search_rx {
            let mut progresses = Vec::new();
            while let Ok(progress) = rx.try_recv() {
                progresses.push(progress);
            }
            progresses
        } else {
            Vec::new()
        };

        for progress in search_progresses {
            self.handle_search_progress(progress);
        }
    }

    /// Process file system change notifications
    fn process_file_changes(&mut self) {
        if let Some(rx) = &mut self.file_change_rx {
            if rx.try_recv().is_ok() {
                self.refresh_needed = true;
            }
        }
    }

    /// Process file system events for notifications
    fn process_file_events(&mut self) {
        let mut events_to_process = Vec::new();
        if let Some(rx) = &mut self.file_event_rx {
            while let Ok(event) = rx.try_recv() {
                events_to_process.push(event);
            }
        }

        for event in events_to_process {
            self.handle_file_event(event);
        }
    }

    /// Process command output from background tasks
    async fn process_command_output(&mut self) {
        let mut command_outputs = Vec::new();
        let mut channel_closed = false;

        if let Some(rx) = &mut self.command_output_rx {
            loop {
                match rx.try_recv() {
                    Ok(output) => command_outputs.push(output),
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        channel_closed = true;
                        break;
                    }
                }
            }
        }

        // Process collected outputs
        for output in command_outputs {
            self.handle_command_output(output);
        }

        // Clean up closed channel
        if channel_closed {
            self.command_output_rx = None;
        }
    }

    /// Process configuration reload notifications
    fn process_config_reload(&mut self) {
        if self.config_reload_rx.try_recv().is_ok() {
            if let Err(e) = self.reload_configuration() {
                self.state.set_status_message(format!("Failed to reload config: {}", e));
            } else {
                self.state.set_status_message("Configuration reloaded");
            }
        }
    }

    /// Apply pending refresh operations
    fn apply_refresh(&mut self) -> Result<()> {
        // Clone the panels to avoid borrow checker issues
        let mut left_panel = self.state.left_panel.clone();
        let mut right_panel = self.state.right_panel.clone();
        
        self.refresh_panel_with_cache(&mut left_panel)?;
        self.refresh_panel_with_cache(&mut right_panel)?;
        
        // Update the original panels
        self.state.left_panel = left_panel;
        self.state.right_panel = right_panel;
        
        Ok(())
    }
}