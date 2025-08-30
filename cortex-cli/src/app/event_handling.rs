use anyhow::Result;
use cortex_core::FileOperation;
use cortex_tui::{Event, Dialog};
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use crossterm::{
    execute,
    style::{ResetColor},
    terminal::{disable_raw_mode, Clear, ClearType, LeaveAlternateScreen},
    event::DisableMouseCapture,
};

use super::App;

impl App {
    /// Main input event handling loop
    pub async fn handle_input_events(&mut self) -> Result<bool> {
        if let Ok(event) = self.events.next().await {
            match event {
                Event::Key(key_event) => {
                    if self.context_menu.is_some() {
                        self.handle_context_menu_input(key_event).await?;
                    } else if matches!(self.dialog, Some(Dialog::Suggestions(_))) {
                        // Suggestions dialog is special - it doesn't block normal input
                        if !self.handle_input(key_event).await? {
                            return Ok(false);
                        }
                    } else if self.dialog.is_some() {
                        // Other dialogs block input
                        if !self.handle_dialog_input(key_event).await? {
                            return Ok(false);
                        }
                    } else {
                        // Handle all input - typing goes to command line by default
                        if !self.handle_input(key_event).await? {
                            return Ok(false);
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    self.handle_mouse_event(mouse_event).await?;
                }
                Event::Resize(_, _) => {
                    self.terminal.autoresize()?;
                }
                Event::Tick => {
                    // Update theme manager for random rotation
                    self.state.theme_manager.update();
                }
            }
        }
        Ok(true)
    }

    /// Handle input when no dialog is active
    async fn handle_input(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle suggestions dialog specially
        if matches!(self.dialog, Some(Dialog::Suggestions(_))) {
            match key.code {
                KeyCode::Up => {
                    if let Some(Dialog::Suggestions(dialog)) = &mut self.dialog {
                        dialog.move_up();
                    }
                    return Ok(true);
                }
                KeyCode::Down => {
                    if let Some(Dialog::Suggestions(dialog)) = &mut self.dialog {
                        dialog.move_down();
                    }
                    return Ok(true);
                }
                KeyCode::Enter => {
                    if let Some(Dialog::Suggestions(dialog)) = &self.dialog {
                        if let Some(suggestion) = dialog.get_selected_suggestion() {
                            self.state.command_line = format!("cd {}", suggestion);
                            self.state.command_cursor = self.state.command_line.len();
                        }
                    }
                    self.dialog = None;
                    self.suggestions_dismissed = false;
                    return Ok(true);
                }
                KeyCode::Esc => {
                    self.dialog = None;
                    self.suggestions_dismissed = true;
                    return Ok(true);
                }
                _ => {}
            }
        }

        // Global hotkeys
        match (key.code, key.modifiers) {
            // Exit application
            (KeyCode::F(10), _) => {
                return self.handle_exit().await;
            }
            
            // Navigation keys - work on panels when command line is empty
            (KeyCode::Up, modifiers) if modifiers.is_empty() && self.state.command_line.is_empty() => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_up();
                if let Ok(size) = self.terminal.size() {
                    panel.update_view_offset(size.height as usize - 5);
                }
            }
            (KeyCode::Down, modifiers) if modifiers.is_empty() && self.state.command_line.is_empty() => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_down();
                if let Ok(size) = self.terminal.size() {
                    panel.update_view_offset(size.height as usize - 5);
                }
            }
            (KeyCode::Left, modifiers) if modifiers.is_empty() && self.state.command_line.is_empty() => {
                let current_dir = self.state.active_panel().current_dir.clone();
                if let Some(parent) = current_dir.parent() {
                    let _ = self.navigate_to_directory(parent.to_path_buf());
                }
            }
            (KeyCode::Right, modifiers) if modifiers.is_empty() && self.state.command_line.is_empty() => {
                let current_entry = self.state.active_panel().current_entry().cloned();
                if let Some(entry) = current_entry {
                    if entry.file_type == cortex_core::FileType::Directory && entry.name != ".." {
                        let _ = self.navigate_to_directory(entry.path);
                    }
                }
            }

            // Panel switching
            (KeyCode::Tab, _) if self.state.command_line.is_empty() => {
                // Toggle active panel
                match self.state.active_panel {
                    cortex_core::ActivePanel::Left => {
                        self.state.active_panel = cortex_core::ActivePanel::Right;
                    }
                    cortex_core::ActivePanel::Right => {
                        self.state.active_panel = cortex_core::ActivePanel::Left;
                    }
                }
            }

            // File operations
            (KeyCode::F(5), _) => {
                self.handle_copy_operation().await?;
            }
            (KeyCode::F(6), _) => {
                self.handle_move_operation().await?;
            }
            (KeyCode::F(7), _) => {
                self.handle_create_directory_operation().await?;
            }
            (KeyCode::F(8), _) => {
                self.handle_delete_operation().await?;
            }
            (KeyCode::F(9), _) => {
                // F9 - Configuration dialog
                let config = self.state.config_manager.get();
                self.dialog = Some(Dialog::Config(cortex_tui::ConfigDialog::new(
                    config,
                    &self.state.theme_manager,
                )));
            }

            // Special keys with Ctrl
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                // Toggle hidden files
                {
                    let panel = self.state.active_panel_mut();
                    panel.show_hidden = !panel.show_hidden;
                }
                // Refresh after toggling
                let mut active_panel = self.state.active_panel().clone();
                self.refresh_panel_with_cache(&mut active_panel)?;
                *self.state.active_panel_mut() = active_panel;
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.mark_all_files();
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if !self.state.command_line.is_empty() {
                    self.state.command_line.clear();
                    self.state.command_cursor = 0;
                } else {
                    self.unmark_all_files();
                }
            }

            // Command line handling
            (KeyCode::Char(c), _) => {
                self.state.command_line.insert(self.state.command_cursor, c);
                self.state.command_cursor += 1;
            }
            (KeyCode::Backspace, _) => {
                if self.state.command_cursor > 0 && !self.state.command_line.is_empty() {
                    self.state.command_line.remove(self.state.command_cursor - 1);
                    self.state.command_cursor -= 1;
                }
            }
            (KeyCode::Enter, _) => {
                if !self.state.command_line.is_empty() {
                    return self.handle_command_execution().await;
                }
            }
            (KeyCode::Esc, _) => {
                self.state.command_line.clear();
                self.state.command_cursor = 0;
                self.suggestions_dismissed = true;
                if matches!(self.dialog, Some(Dialog::Suggestions(_))) {
                    self.dialog = None;
                }
            }

            _ => {}
        }

        Ok(true)
    }

    /// Handle dialog input when a dialog is active
    async fn handle_dialog_input(&mut self, key: KeyEvent) -> Result<bool> {
        // Global exit key
        if key.code == KeyCode::F(10) {
            return self.handle_exit().await;
        }

        match &mut self.dialog {
            Some(Dialog::Confirm(dialog)) => {
                match key.code {
                    KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                        dialog.toggle_selection();
                    }
                    KeyCode::Enter => {
                        if dialog.selected {
                            if let Some(operation) = self.state.pending_operation.take() {
                                self.execute_operation(operation).await?;
                            }
                        }
                        self.dialog = None;
                        self.state.pending_operation = None;
                    }
                    KeyCode::Esc => {
                        self.dialog = None;
                        self.state.pending_operation = None;
                    }
                    _ => {}
                }
            }
            Some(Dialog::Input(dialog)) => {
                match key.code {
                    KeyCode::Char(c) => {
                        dialog.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        dialog.delete_char();
                    }
                    KeyCode::Left => {
                        dialog.move_cursor_left();
                    }
                    KeyCode::Right => {
                        dialog.move_cursor_right();
                    }
                    KeyCode::Enter => {
                        // Handle dialog submission
                        if let Some(cortex_core::FileOperation::CreateDir { path }) = &self.state.pending_operation {
                            let new_operation = cortex_core::FileOperation::CreateDir {
                                path: path.join(&dialog.value),
                            };
                            self.execute_operation(new_operation).await?;
                        }
                        self.dialog = None;
                    }
                    KeyCode::Esc => {
                        self.dialog = None;
                        self.state.pending_operation = None;
                    }
                    _ => {}
                }
            }
            Some(Dialog::Error(_)) => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    self.dialog = None;
                }
            }
            Some(Dialog::Help(dialog)) => {
                match key.code {
                    KeyCode::Up => dialog.scroll_up(),
                    KeyCode::Down => dialog.scroll_down(),
                    KeyCode::Esc => {
                        self.dialog = None;
                    }
                    _ => {}
                }
            }
            Some(Dialog::Config(dialog)) => {
                if dialog.editing {
                    // Handle editing mode
                    match key.code {
                        KeyCode::Char(c) => {
                            dialog.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            dialog.delete_char();
                        }
                        KeyCode::Left => {
                            dialog.move_cursor_left();
                        }
                        KeyCode::Right => {
                            dialog.move_cursor_right();
                        }
                        KeyCode::Enter => {
                            dialog.confirm_edit();
                        }
                        KeyCode::Esc => {
                            dialog.cancel_edit();
                        }
                        _ => {}
                    }
                } else {
                    // Handle navigation mode
                    match key.code {
                        KeyCode::Up => {
                            dialog.move_selection_up();
                        }
                        KeyCode::Down => {
                            dialog.move_selection_down();
                        }
                        KeyCode::Left => {
                            dialog.prev_tab();
                        }
                        KeyCode::Right => {
                            dialog.next_tab();
                        }
                        KeyCode::Enter => {
                            dialog.start_edit();
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            // Save configuration
                            let config = dialog.config.clone();
                            if let Err(e) = self.state.config_manager.update(|c| *c = config) {
                                self.dialog = Some(Dialog::Error(cortex_tui::ErrorDialog::new(format!(
                                    "Failed to save configuration: {}",
                                    e
                                ))));
                            } else {
                                self.state.set_status_message("Configuration saved successfully");
                                // Apply the new configuration
                                let _ = self.apply_configuration();
                                self.dialog = None;
                            }
                        }
                        KeyCode::Esc => {
                            self.dialog = None;
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                // Handle other dialog types with basic Esc to close
                if key.code == KeyCode::Esc {
                    self.dialog = None;
                }
            }
        }

        Ok(true)
    }

    /// Handle application exit
    async fn handle_exit(&mut self) -> Result<bool> {
        // Stop file monitoring if active
        if let Some(ref monitor) = self.state.file_monitor {
            if let Err(e) = monitor.stop().await {
                log::warn!("Failed to stop file monitor: {}", e);
            }
        }

        // Reset terminal colors before exit
        let _ = execute!(
            self.terminal.backend_mut(),
            ResetColor,
            Clear(ClearType::All)
        );

        // Cleanup terminal
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );

        Ok(false) // Signal to exit
    }

    /// Handle command execution
    async fn handle_command_execution(&mut self) -> Result<bool> {
        let command = self.state.command_line.clone();
        self.state.command_history.push(command.clone());
        self.state.command_line.clear();
        self.state.command_cursor = 0;

        // Handle special commands
        match command.trim() {
            "exit" | "quit" | "q" => {
                return Ok(false);
            }
            "restart" => {
                let exe = std::env::current_exe()?;
                let args: Vec<String> = std::env::args().skip(1).collect();
                std::process::Command::new(exe).args(&args).spawn()?;
                return Ok(false);
            }
            cmd if cmd.starts_with("cd ") => {
                let path = cmd[3..].trim();
                let new_path = if path.starts_with('/') {
                    std::path::PathBuf::from(path)
                } else {
                    self.state.active_panel().current_dir.join(path)
                };
                let _ = self.navigate_to_directory(new_path);
            }
            _ => {
                // Execute as system command
                self.execute_system_command(&command).await?;
            }
        }

        Ok(true)
    }

    /// Handle file operation requests
    async fn handle_copy_operation(&mut self) -> Result<()> {
        if let Some(operation) = self.prepare_copy_operation() {
            let config = self.state.config_manager.get();
            if config.general.confirm_operations {
                self.state.pending_operation = Some(operation.clone());
                self.dialog = Some(Dialog::Confirm(
                    cortex_tui::ConfirmDialog::new("Copy Files", "Confirm copy operation?")
                ));
            } else {
                // Execute immediately without confirmation
                self.execute_operation(operation).await?;
            }
        }
        Ok(())
    }

    async fn handle_move_operation(&mut self) -> Result<()> {
        if let Some(operation) = self.prepare_move_operation() {
            let config = self.state.config_manager.get();
            if config.general.confirm_operations {
                self.state.pending_operation = Some(operation.clone());
                self.dialog = Some(Dialog::Confirm(
                    cortex_tui::ConfirmDialog::new("Move Files", "Confirm move operation?")
                ));
            } else {
                // Execute immediately without confirmation
                self.execute_operation(operation).await?;
            }
        }
        Ok(())
    }

    async fn handle_create_directory_operation(&mut self) -> Result<()> {
        self.dialog = Some(Dialog::Input(
            cortex_tui::InputDialog::new("Create Directory", "Enter directory name:")
        ));
        self.state.pending_operation = Some(cortex_core::FileOperation::CreateDir {
            path: self.state.active_panel().current_dir.clone(),
        });
        Ok(())
    }

    async fn handle_delete_operation(&mut self) -> Result<()> {
        if let Some(operation) = self.prepare_delete_operation() {
            let config = self.state.config_manager.get();
            if config.general.confirm_delete {
                self.state.pending_operation = Some(operation.clone());
                self.dialog = Some(Dialog::Confirm(
                    cortex_tui::ConfirmDialog::new("Delete Files", "Confirm delete operation? This cannot be undone!")
                ));
            } else {
                // Execute immediately without confirmation
                self.execute_operation(operation).await?;
            }
        }
        Ok(())
    }

    /// Prepare operations based on current selection
    fn prepare_copy_operation(&self) -> Option<FileOperation> {
        let active_panel = self.state.active_panel();
        let inactive_panel = self.state.inactive_panel();
        
        let sources = if !active_panel.marked_files.is_empty() {
            active_panel.marked_files.to_vec()
        } else if let Some(entry) = active_panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::Copy {
            sources,
            destination: inactive_panel.current_dir.clone(),
        })
    }

    fn prepare_move_operation(&self) -> Option<FileOperation> {
        let active_panel = self.state.active_panel();
        let inactive_panel = self.state.inactive_panel();
        
        let sources = if !active_panel.marked_files.is_empty() {
            active_panel.marked_files.to_vec()
        } else if let Some(entry) = active_panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::Move {
            sources,
            destination: inactive_panel.current_dir.clone(),
        })
    }

    fn prepare_delete_operation(&self) -> Option<FileOperation> {
        let active_panel = self.state.active_panel();
        
        let targets = if !active_panel.marked_files.is_empty() {
            active_panel.marked_files.to_vec()
        } else if let Some(entry) = active_panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::Delete { targets })
    }

    /// Execute system command
    async fn execute_system_command(&mut self, command: &str) -> Result<()> {
        use tokio::process::Command;
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.state.active_panel().current_dir)
            .output()
            .await?;

        let result = String::from_utf8_lossy(&output.stdout);
        if !result.trim().is_empty() {
            self.state.set_status_message(format!("Command output: {}", result.trim()));
        } else if !output.stderr.is_empty() {
            let error = String::from_utf8_lossy(&output.stderr);
            self.state.set_status_message(format!("Command error: {}", error.trim()));
        }

        Ok(())
    }

    /// Handle mouse events
    async fn handle_mouse_event(&mut self, _mouse_event: crossterm::event::MouseEvent) -> Result<()> {
        // TODO: Implement mouse event handling
        Ok(())
    }

    /// Handle context menu input
    async fn handle_context_menu_input(&mut self, _key_event: KeyEvent) -> Result<()> {
        // TODO: Implement context menu handling
        Ok(())
    }
}