use anyhow::Result;
use cortex_core::{
    shortcuts::Action,
    state::{FileOperation, SortMode, SortOrder, ViewMode},
    ActivePanel,
};
use cortex_tui::{Dialog, Event};
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::{
    event::DisableMouseCapture,
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, Clear, ClearType, LeaveAlternateScreen},
};
use std::io;

use super::App;

impl App {
    /// Main input event handling loop
    pub async fn handle_input_events(&mut self) -> Result<bool> {
        if let Ok(event) = self.events.next().await {
            match event {
                Event::Key(key_event) => {
                    if self.context_menu.is_some() {
                        self.handle_context_menu_input(key_event).await?;
                    } else if self.dialog.is_some() {
                        if !self.handle_dialog_input(key_event).await? {
                            return Ok(false);
                        }
                    } else {
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
                    self.state.theme_manager.update();
                }
            }
        }
        Ok(true)
    }

    /// Handle input when no dialog is active
    async fn handle_input(&mut self, key: KeyEvent) -> Result<bool> {
        if let Some(action) = self.shortcut_manager.get_action(key.code, key.modifiers) {
            if !self.handle_action(action).await? {
                return Ok(false);
            }
        } else {
            match key.code {
                KeyCode::Char(c) => {
                    self.state.command_line.insert(self.state.command_cursor, c);
                    self.state.command_cursor += 1;
                }
                KeyCode::Backspace => {
                    if self.state.command_cursor > 0 && !self.state.command_line.is_empty() {
                        self.state.command_line.remove(self.state.command_cursor - 1);
                        self.state.command_cursor -= 1;
                    }
                }
                KeyCode::Esc => {
                    self.state.command_line.clear();
                    self.state.command_cursor = 0;
                }
                _ => {}
            }
        }
        Ok(true)
    }

    async fn handle_action(&mut self, action: Action) -> Result<bool> {
        match action {
            // Navigation
            Action::NavigateUp => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_up();
                if let Ok(size) = self.terminal.size() {
                    panel.update_view_offset(size.height as usize - 5);
                }
            }
            Action::NavigateDown => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_down();
                if let Ok(size) = self.terminal.size() {
                    panel.update_view_offset(size.height as usize - 5);
                }
            }
            Action::NavigateToParent => {
                let current_dir = self.state.active_panel().current_dir.clone();
                if let Some(parent) = current_dir.parent() {
                    let _ = self.navigate_to_directory(parent.to_path_buf());
                }
            }
            Action::NavigateInto => {
                let current_entry = self.state.active_panel().current_entry().cloned();
                if let Some(entry) = current_entry {
                    if entry.file_type == cortex_core::fs::FileType::Directory {
                        if entry.name == ".." {
                            let current_dir = self.state.active_panel().current_dir.clone();
                            if let Some(parent) = current_dir.parent() {
                                let _ = self.navigate_to_directory(parent.to_path_buf());
                            }
                        } else {
                            let _ = self.navigate_to_directory(entry.path);
                        }
                    }
                }
            }
            Action::SwitchPanel => {
                self.state.active_panel = match self.state.active_panel {
                    ActivePanel::Left => ActivePanel::Right,
                    ActivePanel::Right => ActivePanel::Left,
                }
            }

            // File Operations
            Action::Copy => self.handle_copy_operation().await?,
            Action::CopyAs => self.handle_copy_as_operation().await?,
            Action::Move => self.handle_move_operation().await?,
            Action::Delete => self.handle_delete_operation().await?,
            Action::CreateDirectory => self.handle_create_directory_operation().await?,
            Action::Rename => self.handle_rename_operation().await?,
            Action::NewFile => self.handle_new_file_operation().await?,
            Action::ViewFile => self.handle_view_file_operation().await?,
            Action::EditFile => self.handle_edit_file_operation().await?,

            // Sorting
            Action::SortByName => {
                let panel = self.state.active_panel_mut();
                panel.sort_mode = SortMode::Name;
                panel.sort_entries();
                self.refresh_needed = true;
            }
            Action::SortByExtension => {
                let panel = self.state.active_panel_mut();
                panel.sort_mode = SortMode::Extension;
                panel.sort_entries();
                self.refresh_needed = true;
            }
            Action::SortByDate => {
                let panel = self.state.active_panel_mut();
                panel.sort_mode = SortMode::Modified;
                panel.sort_entries();
                self.refresh_needed = true;
            }
            Action::SortBySize => {
                let panel = self.state.active_panel_mut();
                panel.sort_mode = SortMode::Size;
                panel.sort_entries();
                self.refresh_needed = true;
            }
            Action::ReverseSort => {
                let panel = self.state.active_panel_mut();
                panel.sort_order = match panel.sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
                panel.sort_entries();
                self.refresh_needed = true;
            }

            // View
            Action::QuickFilter => self.handle_quick_filter_operation().await?,
            Action::FindInFiles => {
                self.state.set_status_message("Find in files not yet implemented");
            }
            Action::GoToLine => {
                self.state.set_status_message("Go to line not yet implemented");
            }
            Action::ToggleTreeView => {
                self.state.set_status_message("Tree view not yet implemented");
            }
            Action::BriefView => {
                self.state.active_panel_mut().view_mode = ViewMode::Brief;
                self.refresh_needed = true;
            }
            Action::FullView => {
                self.state.active_panel_mut().view_mode = ViewMode::Full;
                self.refresh_needed = true;
            }
            Action::WideView => {
                self.state.active_panel_mut().view_mode = ViewMode::Wide;
                self.refresh_needed = true;
            }

            // Panel Management
            Action::FocusLeftPanel => {
                self.state.active_panel = ActivePanel::Left;
            }
            Action::FocusRightPanel => {
                self.state.active_panel = ActivePanel::Right;
            }
            Action::HidePanels => {
                self.state.panels_hidden = !self.state.panels_hidden;
            }
            Action::PanelMenu => {
                self.state.set_status_message("Panel menu not yet implemented");
            }
            Action::TreePanel => {
                self.state.set_status_message("Tree panel not yet implemented");
            }
            Action::ChangeDriveLeft => {
                self.state.set_status_message("Change drive not yet implemented");
            }
            Action::ChangeDriveRight => {
                self.state.set_status_message("Change drive not yet implemented");
            }

            // Bookmarks and Quick Directories
            Action::SetQuickDir(n) => {
                let path = self.state.active_panel().current_dir.clone();
                self.state.config_manager.update(|config| {
                    config.general.quick_dirs.insert(n, path);
                })?;
                self.state.set_status_message(format!("Quick directory {} set", n));
            }
            Action::ManageBookmarks => {
                self.state.set_status_message("Bookmark management not yet implemented");
            }

            // Advanced Features
            Action::EnterArchive => {
                self.state.set_status_message("Entering archives not yet implemented");
            }
            Action::ExtractArchive => {
                self.state.set_status_message("Extracting archives not yet implemented");
            }
            Action::CreateArchive => {
                self.state.set_status_message("Creating archives not yet implemented");
            }
            Action::SftpConnect => {
                self.state.set_status_message("SFTP connection not yet implemented");
            }
            Action::FtpConnect => {
                self.state.set_status_message("FTP connection not yet implemented");
            }
            Action::Disconnect => {
                self.state.set_status_message("Disconnect not yet implemented");
            }
            Action::CompareDirs => {
                self.state.set_status_message("Directory comparison not yet implemented");
            }
            Action::CalculateSize => {
                self.state.set_status_message("Directory size calculation not yet implemented");
            }
            Action::Properties => {
                self.state.set_status_message("File properties not yet implemented");
            }

            // Selection
            Action::SelectItem => {
                self.state.active_panel_mut().toggle_mark_current();
            }
            Action::SelectAll => self.mark_all_files(),
            Action::SelectNone => self.unmark_all_files(),

            // System
            Action::ShowShortcuts => {
                self.state.set_status_message("Show shortcuts not yet implemented");
            }
            Action::ToggleFullscreen => {
                self.state.set_status_message("Toggle fullscreen not yet implemented");
            }
            Action::OpenSystemTerminal => {
                self.state.set_status_message("Open system terminal not yet implemented");
            }
            Action::ContextHelp => {
                self.state.set_status_message("Context help not yet implemented");
            }
            Action::About => {
                self.state.set_status_message("About not yet implemented");
            }
            // Macros
            Action::StartMacroRecord => {
                self.state.set_status_message("Macro recording not yet implemented");
            }
            Action::PlayMacro => {
                self.state.set_status_message("Macro playback not yet implemented");
            }
            Action::ManageMacros => {
                self.state.set_status_message("Macro management not yet implemented");
            }
            Action::Quit | Action::QuickExit => return self.handle_exit().await,
            Action::Settings => {
                let config = self.state.config_manager.get();
                self.dialog = Some(Dialog::Config(cortex_tui::ConfigDialog::new(
                    config,
                    &self.state.theme_manager,
                )));
            }
            Action::Help => {
                self.dialog = Some(Dialog::Help(cortex_tui::HelpDialog::new()));
            }

            // Command Line
            Action::ShellCommand => {
                self.state.set_status_message("Shell command not yet implemented");
            }
            Action::RunInTerminal => {
                self.state.set_status_message("Run in terminal not yet implemented");
            }
            Action::Autocomplete => {
                self.state.set_status_message("Autocomplete not yet implemented");
            }
            Action::CommandLine => {
                // This might be used to focus the command line, which is the default
            }
            Action::ExecuteCommand => {
                if !self.state.command_line.is_empty() {
                    return self.handle_command_execution().await;
                }
            }

            _ => {
                // TODO: Implement all other actions
                self.state.set_status_message(format!("Action not yet implemented: {:?}", action));
            }
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
                        if let Some(op) = self.state.pending_operation.take() {
                            match op {
                                FileOperation::CreateDir { path } => {
                                    let new_operation = FileOperation::CreateDir {
                                        path: path.join(&dialog.value),
                                    };
                                    self.execute_operation(new_operation).await?;
                                }
                                FileOperation::CreateFile { path } => {
                                    let new_operation = FileOperation::CreateFile {
                                        path: path.join(&dialog.value),
                                    };
                                    self.execute_operation(new_operation).await?;
                                }
                                FileOperation::Rename { old_path, .. } => {
                                    let new_operation = FileOperation::Rename {
                                        old_path,
                                        new_name: dialog.value.clone(),
                                    };
                                    self.execute_operation(new_operation).await?;
                                }
                                FileOperation::CopyAs { source, destination, .. } => {
                                    let new_operation = FileOperation::CopyAs {
                                        source,
                                        destination,
                                        new_name: dialog.value.clone(),
                                    };
                                    self.execute_operation(new_operation).await?;
                                }
                                FileOperation::Filter { .. } => {
                                    self.state.active_panel_mut().apply_filter(&dialog.value);
                                }
                                _ => {}
                            }
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
                        KeyCode::Up => {
                            if dialog.is_boolean_field() {
                                dialog.toggle_current_boolean_value();
                            } else if dialog.current_tab == cortex_tui::ConfigTab::Themes {
                                dialog.cycle_theme_backward();
                            } else if dialog.current_tab == cortex_tui::ConfigTab::AI && dialog.selected_index == 0 {
                                dialog.cycle_provider_backward();
                            }
                        }
                        KeyCode::Down => {
                            if dialog.is_boolean_field() {
                                dialog.toggle_current_boolean_value();
                            } else if dialog.current_tab == cortex_tui::ConfigTab::Themes {
                                dialog.cycle_theme_forward();
                            } else if dialog.current_tab == cortex_tui::ConfigTab::AI && dialog.selected_index == 0 {
                                dialog.cycle_provider_forward();
                            }
                        }
                        KeyCode::Char(c) => {
                            if !dialog.is_dropdown_field() && !dialog.is_boolean_field() {
                                dialog.insert_char(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if !dialog.is_dropdown_field() && !dialog.is_boolean_field() {
                                dialog.delete_char();
                            }
                        }
                        KeyCode::Left => {
                            if !dialog.is_dropdown_field() && !dialog.is_boolean_field() {
                                dialog.move_cursor_left();
                            }
                        }
                        KeyCode::Right => {
                            if !dialog.is_dropdown_field() && !dialog.is_boolean_field() {
                                dialog.move_cursor_right();
                            }
                        }
                        KeyCode::Enter => {
                            if dialog.is_boolean_field() || dialog.is_dropdown_field() {
                                dialog.cancel_edit(); // Just exit edit mode, changes are already applied
                            } else {
                                dialog.confirm_edit(); // For text fields
                            }
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
        // Stop all background services
        self.shutdown_background_services().await?;

        // Properly clean up terminal state
        self.cleanup_terminal()?;

        Ok(false) // Signal to exit
    }

    /// Shutdown all background services
    async fn shutdown_background_services(&mut self) -> Result<()> {
        // Stop event handler background task
        self.events.shutdown();

        // Stop file monitoring if active
        if let Some(ref monitor) = self.state.file_monitor {
            if let Err(e) = monitor.stop().await {
                log::warn!("Failed to stop file monitor: {}", e);
            }
        }

        // Close all channels by dropping the receivers
        self.operation_rx = None;
        self.search_rx = None;
        self.file_change_rx = None;
        self.command_output_rx = None;
        self.file_event_rx = None;
        self.ai_response_rx = None;

        // Small delay to ensure background tasks have time to clean up
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Ok(())
    }

    /// Clean up terminal and restore normal mode
    fn cleanup_terminal(&mut self) -> Result<()> {
        // Flush any pending output
        let _ = self.terminal.flush();

        // Clear screen and reset cursor
        let _ = execute!(
            self.terminal.backend_mut(),
            ResetColor,
            Clear(ClearType::All)
        );

        // Leave alternate screen and disable mouse capture
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );

        // Disable raw mode to restore normal terminal behavior
        let _ = disable_raw_mode();

        // Force flush to ensure all commands are sent
        let _ = execute!(
            io::stdout(),
            ResetColor
        );

        Ok(())
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
        self.state.pending_operation = Some(FileOperation::CreateDir {
            path: self.state.active_panel().current_dir.clone(),
        });
        Ok(())
    }

    async fn handle_new_file_operation(&mut self) -> Result<()> {
        self.dialog = Some(Dialog::Input(
            cortex_tui::InputDialog::new("Create File", "Enter file name:")
        ));
        self.state.pending_operation = Some(FileOperation::CreateFile {
            path: self.state.active_panel().current_dir.clone(),
        });
        Ok(())
    }

    async fn handle_rename_operation(&mut self) -> Result<()> {
        if let Some(entry) = self.state.active_panel().current_entry().cloned() {
            if entry.name != ".." {
                self.dialog = Some(Dialog::Input(
                    cortex_tui::InputDialog::new("Rename", "Enter new name:").with_initial_value(&entry.name),
                ));
                self.state.pending_operation = Some(FileOperation::Rename {
                    old_path: entry.path,
                    new_name: String::new(),
                });
            }
        }
        Ok(())
    }

    async fn handle_copy_as_operation(&mut self) -> Result<()> {
        if let Some(entry) = self.state.active_panel().current_entry().cloned() {
            if entry.name != ".." {
                self.dialog = Some(Dialog::Input(
                    cortex_tui::InputDialog::new("Copy As", "Enter new name:").with_initial_value(&entry.name),
                ));
                let destination = self.state.inactive_panel().current_dir.clone();
                self.state.pending_operation = Some(FileOperation::CopyAs {
                    source: entry.path,
                    destination,
                    new_name: String::new(),
                });
            }
        }
        Ok(())
    }

    async fn handle_quick_filter_operation(&mut self) -> Result<()> {
        self.dialog = Some(Dialog::Input(
            cortex_tui::InputDialog::new("Quick Filter", "Enter filter string:")
        ));
        self.state.pending_operation = Some(FileOperation::Filter {
            filter: String::new(),
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

    async fn handle_edit_file_operation(&mut self) -> Result<()> {
        if let Some(entry) = self.state.active_panel().current_entry().cloned() {
            if entry.file_type == cortex_core::fs::FileType::File {
                let editor = self.state.config_manager.get().general.editor;
                self.suspend_and_run_command(&editor, &[&entry.path.to_string_lossy()])
                    .await?;
                self.refresh_needed = true;
            }
        }
        Ok(())
    }

    async fn handle_view_file_operation(&mut self) -> Result<()> {
        if let Some(entry) = self.state.active_panel().current_entry().cloned() {
            if entry.file_type == cortex_core::fs::FileType::File {
                match cortex_tui::viewer::FileViewer::new(&entry.path) {
                    Ok(mut viewer) => {
                        // Load initial content
                        let height = self.terminal.size()?.height as usize;
                        if let Err(e) = viewer.load_content(height.saturating_sub(5)) {
                            self.dialog = Some(Dialog::Error(cortex_tui::ErrorDialog::new(format!(
                                "Failed to read file: {}",
                                e
                            ))));
                        } else {
                            self.dialog = Some(Dialog::Viewer(cortex_tui::ViewerDialog::new(viewer)));
                        }
                    }
                    Err(e) => {
                        self.dialog = Some(Dialog::Error(cortex_tui::ErrorDialog::new(format!(
                            "Failed to open file: {}",
                            e
                        ))));
                    }
                }
            }
        }
        Ok(())
    }
}