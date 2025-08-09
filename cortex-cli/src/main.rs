use anyhow::Result;
use clap::Parser;
use cortex_core::{AppState, FileSystem, FileOperation};
use cortex_tui::{Dialog, InputDialog, ProgressDialog, ErrorDialog, HelpDialog, Event, EventHandler, KeyBinding, UI};
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, time::Duration};
use tokio::sync::mpsc;

mod command;
mod operations;
use command::CommandProcessor;
use operations::OperationManager;

#[derive(Parser, Debug)]
#[command(name = "cortex")]
#[command(about = "A modern orthodox file manager", long_about = None)]
struct Args {
    #[arg(help = "Directory to open")]
    path: Option<PathBuf>,

    #[arg(short, long, help = "Show version information")]
    version: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args = Args::parse();
    
    if args.version {
        println!("Cortex v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut app = App::new(args.path)?;
    app.run().await
}

struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    events: EventHandler,
    dialog: Option<Dialog>,
    operation_manager: OperationManager,
    operation_rx: Option<mpsc::UnboundedReceiver<cortex_core::OperationProgress>>,
}

impl App {
    fn new(initial_path: Option<PathBuf>) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        let mut state = AppState::new()?;
        
        if let Some(path) = initial_path {
            if path.is_dir() {
                state.left_panel.current_dir = path.clone();
                state.right_panel.current_dir = path;
            }
        }
        
        Self::refresh_panel(&mut state.left_panel)?;
        Self::refresh_panel(&mut state.right_panel)?;
        
        let events = EventHandler::new(Duration::from_millis(100));
        
        Ok(Self {
            state,
            terminal,
            events,
            dialog: None,
            operation_manager: OperationManager::new(),
            operation_rx: None,
        })
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|frame| {
                UI::draw(frame, &self.state);
                if let Some(dialog) = &self.dialog {
                    cortex_tui::dialogs::render_dialog(frame, dialog);
                }
            })?;
            
            if let Some(rx) = &mut self.operation_rx {
                if let Ok(progress) = rx.try_recv() {
                    self.handle_operation_progress(progress);
                }
            }
            
            match self.events.next().await? {
                Event::Key(key_event) => {
                    if self.dialog.is_some() {
                        if !self.handle_dialog_input(key_event).await? {
                            break;
                        }
                    } else if self.state.command_mode {
                        if !self.handle_command_mode_input(key_event).await? {
                            break;
                        }
                    } else if let Some(binding) = KeyBinding::from_key_event(key_event) {
                        if !self.handle_input(binding).await? {
                            break;
                        }
                    }
                }
                Event::Resize(_, _) => {
                    self.terminal.autoresize()?;
                }
                Event::Tick => {}
            }
        }
        
        Ok(())
    }

    async fn handle_command_mode_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        use crossterm::event::KeyModifiers;
        
        match key.code {
            KeyCode::Esc => {
                // Exit command mode
                self.state.command_mode = false;
                self.state.command_line.clear();
                self.state.command_cursor = 0;
                self.state.command_history_index = None;
            }
            KeyCode::Enter => {
                // Execute command
                if !self.state.command_line.is_empty() {
                    let command = self.state.command_line.clone();
                    
                    // Add to history
                    self.state.command_history.push(command.clone());
                    
                    // Check for cd command
                    if command.starts_with("cd ") {
                        let path = &command[3..].trim();
                        if let Some(new_dir) = CommandProcessor::parse_cd_path(
                            path, 
                            &self.state.active_panel().current_dir
                        ) {
                            let panel = self.state.active_panel_mut();
                            panel.current_dir = new_dir;
                            panel.selected_index = 0;
                            panel.view_offset = 0;
                            Self::refresh_panel(panel)?;
                        } else {
                            self.state.set_status_message(format!("cd: cannot access '{}': No such directory", path));
                        }
                    } else if command == "exit" || command == "quit" {
                        return Ok(false);
                    } else {
                        // Execute external command
                        match CommandProcessor::execute_command(&command, &self.state).await {
                            Ok(output) => {
                                if !output.is_empty() {
                                    self.state.set_status_message(output);
                                }
                            }
                            Err(e) => {
                                self.state.set_status_message(format!("Error: {}", e));
                            }
                        }
                    }
                    
                    self.state.command_line.clear();
                    self.state.command_cursor = 0;
                    self.state.command_mode = false;
                    self.state.command_history_index = None;
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Clear line
                self.state.command_line.clear();
                self.state.command_cursor = 0;
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Delete word before cursor
                if self.state.command_cursor > 0 {
                    let mut pos = self.state.command_cursor - 1;
                    // Skip trailing spaces
                    while pos > 0 && self.state.command_line.chars().nth(pos) == Some(' ') {
                        pos -= 1;
                    }
                    // Find word boundary
                    while pos > 0 && self.state.command_line.chars().nth(pos - 1) != Some(' ') {
                        pos -= 1;
                    }
                    self.state.command_line.drain(pos..self.state.command_cursor);
                    self.state.command_cursor = pos;
                }
            }
            KeyCode::Char(c) => {
                // Insert character
                self.state.command_line.insert(self.state.command_cursor, c);
                self.state.command_cursor += 1;
            }
            KeyCode::Backspace => {
                // Delete character before cursor
                if self.state.command_cursor > 0 {
                    self.state.command_cursor -= 1;
                    self.state.command_line.remove(self.state.command_cursor);
                }
            }
            KeyCode::Delete => {
                // Delete character at cursor
                if self.state.command_cursor < self.state.command_line.len() {
                    self.state.command_line.remove(self.state.command_cursor);
                }
            }
            KeyCode::Left => {
                // Move cursor left
                if self.state.command_cursor > 0 {
                    self.state.command_cursor -= 1;
                }
            }
            KeyCode::Right => {
                // Move cursor right
                if self.state.command_cursor < self.state.command_line.len() {
                    self.state.command_cursor += 1;
                }
            }
            KeyCode::Home => {
                // Move cursor to beginning
                self.state.command_cursor = 0;
            }
            KeyCode::End => {
                // Move cursor to end
                self.state.command_cursor = self.state.command_line.len();
            }
            KeyCode::Up => {
                // Navigate history up
                if !self.state.command_history.is_empty() {
                    let new_index = match self.state.command_history_index {
                        None => self.state.command_history.len() - 1,
                        Some(i) if i > 0 => i - 1,
                        Some(i) => i,
                    };
                    self.state.command_history_index = Some(new_index);
                    self.state.command_line = self.state.command_history[new_index].clone();
                    self.state.command_cursor = self.state.command_line.len();
                }
            }
            KeyCode::Down => {
                // Navigate history down
                if let Some(index) = self.state.command_history_index {
                    if index < self.state.command_history.len() - 1 {
                        self.state.command_history_index = Some(index + 1);
                        self.state.command_line = self.state.command_history[index + 1].clone();
                    } else {
                        self.state.command_history_index = None;
                        self.state.command_line.clear();
                    }
                    self.state.command_cursor = self.state.command_line.len();
                }
            }
            KeyCode::Tab => {
                // Tab completion (basic - just adds selected file name)
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.name != ".." {
                        let name = if entry.name.contains(' ') {
                            format!("\"{}\"", entry.name)
                        } else {
                            entry.name.clone()
                        };
                        self.state.command_line.insert_str(self.state.command_cursor, &name);
                        self.state.command_cursor += name.len();
                    }
                }
            }
            _ => {}
        }
        
        Ok(true)
    }

    async fn handle_dialog_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
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
                        if let Some(FileOperation::CreateDir { .. }) = &self.state.pending_operation {
                            let path = self.state.active_panel().current_dir.join(&dialog.value);
                            self.state.pending_operation = Some(FileOperation::CreateDir { path });
                            
                            if let Some(operation) = self.state.pending_operation.take() {
                                self.execute_operation(operation).await?;
                            }
                        } else if let Some(FileOperation::Rename { old_path, .. }) = &self.state.pending_operation {
                            self.state.pending_operation = Some(FileOperation::Rename {
                                old_path: old_path.clone(),
                                new_name: dialog.value.clone(),
                            });
                            
                            if let Some(operation) = self.state.pending_operation.take() {
                                self.execute_operation(operation).await?;
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
            Some(Dialog::Progress(_)) => {
                if key.code == KeyCode::Esc {
                    self.dialog = None;
                    self.operation_rx = None;
                }
            }
            Some(Dialog::Error(_)) => {
                if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                    self.dialog = None;
                }
            }
            Some(Dialog::Help(dialog)) => {
                match key.code {
                    KeyCode::Up => dialog.scroll_up(),
                    KeyCode::Down => dialog.scroll_down(),
                    KeyCode::Esc | KeyCode::F(1) => {
                        self.dialog = None;
                    }
                    _ => {}
                }
            }
            None => {}
        }
        
        Ok(true)
    }

    async fn handle_input(&mut self, binding: KeyBinding) -> Result<bool> {
        use cortex_core::FileType;
        
        match binding {
            KeyBinding::Quit => return Ok(false),
            
            KeyBinding::Help => {
                self.dialog = Some(Dialog::Help(HelpDialog::new()));
            }
            
            KeyBinding::Up => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_up();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::Down => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_down();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::PageUp => {
                let height = self.terminal.size()?.height as usize - 5;
                let panel = self.state.active_panel_mut();
                panel.move_selection_page_up(height);
                panel.update_view_offset(height);
            }
            
            KeyBinding::PageDown => {
                let height = self.terminal.size()?.height as usize - 5;
                let panel = self.state.active_panel_mut();
                panel.move_selection_page_down(height);
                panel.update_view_offset(height);
            }
            
            KeyBinding::Home => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_home();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::End => {
                let panel = self.state.active_panel_mut();
                panel.move_selection_end();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::Enter | KeyBinding::Right => {
                let panel = self.state.active_panel_mut();
                if let Some(entry) = panel.current_entry() {
                    if entry.file_type == FileType::Directory {
                        let new_dir = if entry.name == ".." {
                            panel.current_dir.parent().map(|p| p.to_path_buf())
                        } else {
                            Some(entry.path.clone())
                        };
                        
                        if let Some(dir) = new_dir {
                            panel.current_dir = dir;
                            panel.selected_index = 0;
                            panel.view_offset = 0;
                            Self::refresh_panel(panel)?;
                        }
                    }
                }
            }
            
            KeyBinding::Back | KeyBinding::Left => {
                let panel = self.state.active_panel_mut();
                if let Some(parent) = panel.current_dir.parent() {
                    panel.current_dir = parent.to_path_buf();
                    panel.selected_index = 0;
                    panel.view_offset = 0;
                    Self::refresh_panel(panel)?;
                }
            }
            
            KeyBinding::Tab => {
                self.state.toggle_panel();
            }
            
            KeyBinding::Copy => {
                if let Some(operation) = OperationManager::prepare_copy(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            
            KeyBinding::Move => {
                if let Some(operation) = OperationManager::prepare_move(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            
            KeyBinding::Delete => {
                if let Some(operation) = OperationManager::prepare_delete(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            
            KeyBinding::MakeDir => {
                self.dialog = Some(Dialog::Input(
                    InputDialog::new("Create Directory", "Enter directory name:")
                ));
                self.state.pending_operation = Some(FileOperation::CreateDir {
                    path: PathBuf::new(),
                });
            }
            
            KeyBinding::Rename => {
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.name != ".." {
                        self.dialog = Some(Dialog::Input(
                            InputDialog::new("Rename", "Enter new name:")
                                .with_value(&entry.name)
                        ));
                        self.state.pending_operation = Some(FileOperation::Rename {
                            old_path: entry.path.clone(),
                            new_name: entry.name.clone(),
                        });
                    }
                }
            }
            
            KeyBinding::ToggleHidden => {
                let panel = self.state.active_panel_mut();
                panel.show_hidden = !panel.show_hidden;
                Self::refresh_panel(panel)?;
            }
            
            KeyBinding::ToggleMark => {
                let panel = self.state.active_panel_mut();
                panel.toggle_mark_current();
                panel.move_selection_down();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            
            KeyBinding::MarkAll => {
                let panel = self.state.active_panel_mut();
                for entry in &panel.entries {
                    if entry.name != ".." {
                        panel.marked_files.push(entry.path.clone());
                    }
                }
            }
            
            KeyBinding::UnmarkAll => {
                let panel = self.state.active_panel_mut();
                panel.clear_marks();
            }
            
            KeyBinding::Refresh => {
                Self::refresh_panel(self.state.active_panel_mut())?;
                let inactive = match self.state.active_panel {
                    cortex_core::ActivePanel::Left => &mut self.state.right_panel,
                    cortex_core::ActivePanel::Right => &mut self.state.left_panel,
                };
                Self::refresh_panel(inactive)?;
            }
            
            KeyBinding::CommandMode => {
                self.state.command_mode = true;
                self.state.command_cursor = self.state.command_line.len();
            }
            
            _ => {}
        }
        
        Ok(true)
    }

    async fn execute_operation(&mut self, operation: FileOperation) -> Result<()> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.operation_rx = Some(rx);
        
        let title = match &operation {
            FileOperation::Copy { .. } => "Copying Files",
            FileOperation::Move { .. } => "Moving Files",
            FileOperation::Delete { .. } => "Deleting Files",
            FileOperation::CreateDir { .. } => "Creating Directory",
            FileOperation::Rename { .. } => "Renaming",
        };
        
        self.dialog = Some(Dialog::Progress(
            ProgressDialog::new(title, "Processing...")
        ));
        
        let result = self.operation_manager.execute_operation(operation, tx).await;
        
        if let Err(e) = result {
            self.dialog = Some(Dialog::Error(
                ErrorDialog::new(format!("Operation failed: {}", e))
            ));
        } else {
            self.dialog = None;
            Self::refresh_panel(self.state.active_panel_mut())?;
            let inactive = match self.state.active_panel {
                cortex_core::ActivePanel::Left => &mut self.state.right_panel,
                cortex_core::ActivePanel::Right => &mut self.state.left_panel,
            };
            Self::refresh_panel(inactive)?;
            self.state.active_panel_mut().clear_marks();
        }
        
        self.operation_rx = None;
        Ok(())
    }

    fn handle_operation_progress(&mut self, progress: cortex_core::OperationProgress) {
        if let Some(Dialog::Progress(ref mut dialog)) = self.dialog {
            match progress {
                cortex_core::OperationProgress::Started { operation } => {
                    dialog.operation = operation;
                }
                cortex_core::OperationProgress::Progress { current, total, message } => {
                    dialog.update(current, total, message);
                }
                cortex_core::OperationProgress::Completed { .. } => {
                    dialog.message = "Operation completed".to_string();
                }
                cortex_core::OperationProgress::Failed { operation, error } => {
                    self.dialog = Some(Dialog::Error(
                        ErrorDialog::new(format!("Failed: {}", error))
                            .with_details(operation)
                    ));
                }
            }
        }
    }

    fn refresh_panel(panel: &mut cortex_core::PanelState) -> Result<()> {
        panel.entries = FileSystem::list_directory(&panel.current_dir, panel.show_hidden)?;
        panel.sort_entries();
        
        if panel.selected_index >= panel.entries.len() && !panel.entries.is_empty() {
            panel.selected_index = panel.entries.len() - 1;
        }
        
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}