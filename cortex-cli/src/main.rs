use anyhow::Result;
use clap::Parser;
use cortex_core::{
    AppState, CacheRefresher, DirectoryCache, FileOperation, FileSystem, FileType, LuaPlugin,
    PluginEvent, RemoteCredentials, VfsPath,
};
use cortex_plugins::Plugin;
use cortex_tui::{
    CommandPaletteDialog, ConfigDialog, ConnectionDialog, ConnectionType, ContextMenu,
    ContextMenuAction, Dialog, EditorDialog, ErrorDialog, Event, EventHandler, FileViewer,
    FilterDialog, HelpDialog, InputDialog, MouseAction, MouseHandler, MouseRegionManager,
    NotificationManager, NotificationType, PluginDialog, Position, ProgressDialog, SaveChoice,
    SaveConfirmDialog, SearchDialog, SearchState, TextEditor, ViewerDialog, UI,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::mpsc;

mod command;
mod operations;
mod update;
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

    #[arg(long, help = "Check for updates")]
    check_updates: bool,

    #[arg(long, help = "Update to latest version")]
    update: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    if args.version {
        println!("Cortex v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Handle update checking
    if args.check_updates {
        use update::UpdateManager;

        println!("Checking for updates...");
        let manager = UpdateManager::new()?;

        match manager.check_for_updates().await {
            Ok(Some(update_info)) => {
                println!("Update available: v{}", update_info.version);
                println!("Release date: {}", update_info.release_date);
                println!("Download size: {} bytes", update_info.size);
                println!("\nRelease notes:\n{}", update_info.release_notes);
                println!("\nRun 'cortex --update' to install");
            }
            Ok(None) => {
                println!("You are running the latest version");
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
            }
        }
        return Ok(());
    }

    // Handle update installation
    if args.update {
        use update::UpdateManager;

        println!("Checking for updates...");
        let manager = UpdateManager::new()?;

        match manager.check_for_updates().await {
            Ok(Some(update_info)) => {
                println!("Found update: v{}", update_info.version);
                println!("Downloading...");

                if let Err(e) = manager.install_update(update_info).await {
                    eprintln!("Failed to install update: {}", e);
                    std::process::exit(1);
                }

                println!("Update installed successfully!");
                println!("Please restart Cortex to use the new version");
            }
            Ok(None) => {
                println!("You are already running the latest version");
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
            }
        }
        return Ok(());
    }

    let mut app = App::new(args.path).await?;
    app.run().await
}

struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    events: EventHandler,
    dialog: Option<Dialog>,
    pending_editor: Option<EditorDialog>,
    operation_manager: OperationManager,
    operation_rx: Option<mpsc::UnboundedReceiver<cortex_core::OperationProgress>>,
    search_rx: Option<mpsc::UnboundedReceiver<cortex_core::SearchProgress>>,
    refresh_needed: bool,
    file_change_rx: Option<mpsc::UnboundedReceiver<()>>,
    file_event_rx: Option<mpsc::UnboundedReceiver<cortex_core::FileMonitorEvent>>,
    notification_manager: NotificationManager,
    mouse_handler: MouseHandler,
    context_menu: Option<ContextMenu>,
    _mouse_regions: MouseRegionManager,
}

impl App {
    async fn new(initial_path: Option<PathBuf>) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let mut state = AppState::new()?;

        if let Some(path) = initial_path {
            if path.is_dir() {
                state.left_panel.current_dir = path.clone();
                state.right_panel.current_dir = path;
            }
        }

        Self::refresh_panel_with_cache(&mut state.left_panel, &state.directory_cache)?;
        Self::refresh_panel_with_cache(&mut state.right_panel, &state.directory_cache)?;

        // Apply initial configuration
        Self::apply_configuration(&mut state);

        // Load plugins
        if let Err(e) = Self::load_plugins(&mut state).await {
            eprintln!("Warning: Failed to load plugins: {}", e);
        }

        // Create file change notification channels
        let (file_change_tx, file_change_rx) = mpsc::unbounded_channel();
        let (file_event_tx, file_event_rx) = mpsc::unbounded_channel();

        // Initialize file monitor with callback
        let file_change_tx_clone = file_change_tx.clone();
        let file_event_tx_clone = file_event_tx.clone();
        if let Err(e) = Self::init_file_monitor_with_callback(
            &mut state,
            file_change_tx_clone,
            file_event_tx_clone,
        )
        .await
        {
            eprintln!("Warning: Failed to initialize file monitor: {}", e);
        }

        // Start cache refresher
        let cache_refresher = Arc::new(CacheRefresher::new(state.directory_cache.clone()));
        cache_refresher.start().await;
        state.cache_refresher = Some(cache_refresher.clone());

        let events = EventHandler::new(Duration::from_millis(100));

        Ok(Self {
            state,
            terminal,
            events,
            dialog: None,
            pending_editor: None,
            operation_manager: OperationManager::new(),
            operation_rx: None,
            search_rx: None,
            refresh_needed: false,
            file_change_rx: Some(file_change_rx),
            file_event_rx: Some(file_event_rx),
            notification_manager: NotificationManager::new(),
            mouse_handler: MouseHandler::new(),
            context_menu: None,
            _mouse_regions: MouseRegionManager::new(),
        })
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|frame| {
                UI::draw(frame, &self.state);
                if let Some(ref mut dialog) = self.dialog {
                    cortex_tui::dialogs::render_dialog(frame, dialog);
                }
                // Render notifications on top
                self.notification_manager.render(frame);
            })?;

            if let Some(rx) = &mut self.operation_rx {
                if let Ok(progress) = rx.try_recv() {
                    self.handle_operation_progress(progress);
                }
            }

            // Check for search progress
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

            // Check for file system changes
            if let Some(rx) = &mut self.file_change_rx {
                if rx.try_recv().is_ok() {
                    self.refresh_needed = true;
                }
            }

            // Check for file events for notifications
            let mut events_to_process = Vec::new();
            if let Some(rx) = &mut self.file_event_rx {
                while let Ok(event) = rx.try_recv() {
                    events_to_process.push(event);
                }
            }

            for event in events_to_process {
                self.handle_file_event(event);
            }

            match self.events.next().await? {
                Event::Key(key_event) => {
                    if self.context_menu.is_some() {
                        self.handle_context_menu_input(key_event).await?;
                    } else if self.dialog.is_some() {
                        if !self.handle_dialog_input(key_event).await? {
                            break;
                        }
                    } else {
                        // Handle all input - typing goes to command line by default
                        if !self.handle_input(key_event).await? {
                            break;
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
                    
                    // Check if panels need refreshing due to file system changes
                    if self.refresh_needed && self.state.is_file_monitoring_active() {
                        if let Err(e) = Self::refresh_panel_with_cache(
                            &mut self.state.left_panel,
                            &self.state.directory_cache,
                        ) {
                            log::warn!("Failed to refresh left panel: {}", e);
                        }
                        if let Err(e) = Self::refresh_panel_with_cache(
                            &mut self.state.right_panel,
                            &self.state.directory_cache,
                        ) {
                            log::warn!("Failed to refresh right panel: {}", e);
                        }
                        self.refresh_needed = false;
                    }
                }
            }
        }

        // Ensure cleanup happens when we break from the loop
        self.cleanup_and_exit().await?;
        Ok(())
    }

    async fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // First check for special keys that work globally
        match (key.code, key.modifiers) {
            // Navigation keys - work on panels
            (KeyCode::Up, modifiers)
                if modifiers.is_empty() && self.state.command_line.is_empty() =>
            {
                let panel = self.state.active_panel_mut();
                panel.move_selection_up();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            (KeyCode::Down, modifiers)
                if modifiers.is_empty() && self.state.command_line.is_empty() =>
            {
                let panel = self.state.active_panel_mut();
                panel.move_selection_down();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }
            (KeyCode::Left, modifiers)
                if modifiers.is_empty() && self.state.command_line.is_empty() =>
            {
                let current_dir = self.state.active_panel().current_dir.clone();
                if let Some(parent) = current_dir.parent() {
                    self.navigate_to_directory(parent.to_path_buf()).await?;
                }
            }
            (KeyCode::Right, modifiers)
                if modifiers.is_empty() && self.state.command_line.is_empty() =>
            {
                let current_entry = self.state.active_panel().current_entry().cloned();
                let current_dir = self.state.active_panel().current_dir.clone();

                if let Some(entry) = current_entry {
                    if entry.file_type == FileType::Directory {
                        let new_dir = if entry.name == ".." {
                            current_dir.parent().map(|p| p.to_path_buf())
                        } else {
                            Some(entry.path.clone())
                        };

                        if let Some(dir) = new_dir {
                            self.navigate_to_directory(dir).await?;
                        }
                    }
                }
            }

            // Command line navigation when typing
            (KeyCode::Left, modifiers)
                if modifiers.is_empty() && !self.state.command_line.is_empty() =>
            {
                if self.state.command_cursor > 0 {
                    self.state.command_cursor -= 1;
                }
            }
            (KeyCode::Right, modifiers)
                if modifiers.is_empty() && !self.state.command_line.is_empty() =>
            {
                if self.state.command_cursor < self.state.command_line.len() {
                    self.state.command_cursor += 1;
                }
            }

            // History navigation with Up/Down when command line has text
            (KeyCode::Up, modifiers)
                if modifiers.is_empty() && !self.state.command_line.is_empty() =>
            {
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
            (KeyCode::Down, modifiers)
                if modifiers.is_empty() && !self.state.command_line.is_empty() =>
            {
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

            // Global keys that always work
            (KeyCode::Tab, _) => {
                self.state.toggle_panel();
            }
            (KeyCode::Enter, modifiers)
                if modifiers.is_empty() && self.state.command_line.is_empty() =>
            {
                // Enter on empty command line navigates directories and archives
                if self.state.active_panel().is_using_vfs() {
                    // In VFS mode
                    let panel = self.state.active_panel();
                    if let Some(entry) = panel.current_vfs_entry().cloned() {
                        if entry.name == ".." {
                            // Navigate back from VFS
                            self.state.navigate_back_from_vfs()?;
                            let panel = self.state.active_panel_mut();
                            Self::refresh_panel(panel)?;
                        } else {
                            // VFS navigation temporarily disabled
                            // SSH/FTP support requires OpenSSL
                            self.state
                                .set_status_message("VFS navigation not available in this build");
                        }
                    }
                } else {
                    // In regular filesystem mode
                    let panel = self.state.active_panel();
                    if let Some(entry) = panel.current_entry().cloned() {
                        if entry.file_type == FileType::Directory {
                            let new_dir = if entry.name == ".." {
                                panel.current_dir.parent().map(|p| p.to_path_buf())
                            } else {
                                Some(entry.path.clone())
                            };

                            if let Some(dir) = new_dir {
                                let panel = self.state.active_panel_mut();
                                panel.current_dir = dir;
                                panel.selected_index = 0;
                                panel.view_offset = 0;
                                Self::refresh_panel(panel)?;
                            }
                        } else if entry
                            .path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| {
                                matches!(ext.to_lowercase().as_str(), "zip" | "tar" | "gz" | "7z")
                            })
                            .unwrap_or(false)
                        {
                            // Navigate into archive
                            let vfs_path = VfsPath::Archive {
                                archive_path: entry.path.clone(),
                                internal_path: String::new(),
                            };
                            self.state.navigate_into_vfs(vfs_path)?;
                        }
                    }
                }
            }
            (KeyCode::Enter, _) if !self.state.command_line.is_empty() => {
                // Execute command
                let command = self.state.command_line.clone();

                // Check for special / commands
                if command.starts_with("/") {
                    self.handle_special_command(&command[1..]).await?;
                } else {
                    // Add to history
                    self.state.command_history.push(command.clone());

                    // Check for cd command
                    if command.starts_with("cd ") {
                        let path = &command[3..].trim();
                        if let Some(new_dir) = CommandProcessor::parse_cd_path(
                            path,
                            &self.state.active_panel().current_dir,
                        ) {
                            let panel = self.state.active_panel_mut();
                            panel.current_dir = new_dir;
                            panel.selected_index = 0;
                            panel.view_offset = 0;
                            Self::refresh_panel(panel)?;
                        } else {
                            self.state.set_status_message(format!(
                                "cd: cannot access '{}': No such directory",
                                path
                            ));
                        }
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
                }

                self.state.command_line.clear();
                self.state.command_cursor = 0;
                self.state.command_history_index = None;
            }

            // Function keys
            (KeyCode::F(1), _) => {
                self.dialog = Some(Dialog::Help(HelpDialog::new()));
            }
            (KeyCode::F(3), _) => {
                // View file
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.file_type == FileType::File {
                        match FileViewer::new(&entry.path) {
                            Ok(mut viewer) => {
                                let height =
                                    self.terminal.size().unwrap_or_default().height as usize;
                                let _ = viewer.load_content(height - 8); // Leave space for UI chrome
                                self.dialog = Some(Dialog::Viewer(ViewerDialog::new(viewer)));
                            }
                            Err(e) => {
                                self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                                    "Cannot view file: {}",
                                    e
                                ))));
                            }
                        }
                    }
                }
            }
            (KeyCode::F(4), _) => {
                // Edit file
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.file_type == FileType::File {
                        match TextEditor::new(&entry.path) {
                            Ok(editor) => {
                                self.dialog = Some(Dialog::Editor(EditorDialog::new(editor)));
                            }
                            Err(e) => {
                                self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                                    "Cannot edit file: {}",
                                    e
                                ))));
                            }
                        }
                    }
                }
            }
            (KeyCode::F(5), _) => {
                if let Some(operation) = OperationManager::prepare_copy(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            (KeyCode::F(6), _) => {
                if let Some(operation) = OperationManager::prepare_move(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            (KeyCode::F(7), _) => {
                self.dialog = Some(Dialog::Input(InputDialog::new(
                    "Create Directory",
                    "Enter directory name:",
                )));
                self.state.pending_operation = Some(FileOperation::CreateDir {
                    path: PathBuf::new(),
                });
            }
            (KeyCode::F(8), _) => {
                if let Some(operation) = OperationManager::prepare_delete(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            (KeyCode::F(9), _) => {
                // Cycle through themes
                self.state.theme_manager.next_theme();
                self.state.set_status_message(&format!(
                    "Theme changed to: {:?}",
                    self.state.theme_manager.get_current_theme().mode
                ));
            }
            (KeyCode::F(10), _) => {
                // Open theme selector or toggle random mode
                if self.state.theme_manager.get_current_theme().mode == cortex_core::ThemeMode::Random {
                    self.state.theme_manager.set_theme(cortex_core::ThemeMode::Dark);
                    self.state.set_status_message("Random theme rotation disabled");
                } else {
                    self.state.theme_manager.set_theme(cortex_core::ThemeMode::Random);
                    self.state.set_status_message("Random theme rotation enabled (changes every 10 minutes)");
                }
            }

            // Delete key - move to trash by default
            (KeyCode::Delete, KeyModifiers::NONE) => {
                if let Some(operation) =
                    OperationManager::prepare_delete_to_trash(&self.state).await
                {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }

            // Shift+Delete - permanent delete
            (KeyCode::Delete, KeyModifiers::SHIFT) => {
                if let Some(operation) = OperationManager::prepare_delete(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }

            // Control keys
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                // Properly cleanup and exit
                self.cleanup_and_exit().await?;
                return Ok(false);
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                if self.state.command_line.is_empty() {
                    if let Some(entry) = self.state.active_panel().current_entry() {
                        if entry.name != ".." {
                            self.dialog = Some(Dialog::Input(
                                InputDialog::new("Rename", "Enter new name:")
                                    .with_value(&entry.name),
                            ));
                            self.state.pending_operation = Some(FileOperation::Rename {
                                old_path: entry.path.clone(),
                                new_name: entry.name.clone(),
                            });
                        }
                    }
                } else {
                    // Refresh panels
                    Self::refresh_panel(self.state.active_panel_mut())?;
                    let inactive = match self.state.active_panel {
                        cortex_core::ActivePanel::Left => &mut self.state.right_panel,
                        cortex_core::ActivePanel::Right => &mut self.state.left_panel,
                    };
                    Self::refresh_panel(inactive)?;
                }
            }
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                let panel = self.state.active_panel_mut();
                panel.show_hidden = !panel.show_hidden;
                Self::refresh_panel(panel)?;
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                let panel = self.state.active_panel_mut();
                for entry in &panel.entries {
                    if entry.name != ".." {
                        panel.marked_files.push(entry.path.clone());
                    }
                }
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if !self.state.command_line.is_empty() {
                    // Clear command line
                    self.state.command_line.clear();
                    self.state.command_cursor = 0;
                } else {
                    // Unmark all
                    let panel = self.state.active_panel_mut();
                    panel.clear_marks();
                }
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                // Quick filter
                let current_filter = self.state.active_panel().filter.clone();
                self.dialog = Some(Dialog::Filter(FilterDialog::with_current_filter(
                    current_filter.as_ref(),
                )));
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                // Copy to clipboard
                if let Some(operation) =
                    OperationManager::prepare_copy_to_clipboard(&self.state).await
                {
                    self.execute_operation(operation).await?;
                    self.state.set_status_message("Files copied to clipboard");
                }
            }
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                // Paste from clipboard
                if let Some(operation) =
                    OperationManager::prepare_paste_from_clipboard(&self.state).await
                {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            (KeyCode::Char('7'), KeyModifiers::ALT) => {
                // Advanced search (Alt+F7)
                self.dialog = Some(Dialog::Search(SearchDialog::new()));
            }

            // Special keys
            (KeyCode::PageUp, _) => {
                let height = self.terminal.size()?.height as usize - 5;
                let panel = self.state.active_panel_mut();
                panel.move_selection_page_up(height);
                panel.update_view_offset(height);
            }
            (KeyCode::PageDown, _) => {
                let height = self.terminal.size()?.height as usize - 5;
                let panel = self.state.active_panel_mut();
                panel.move_selection_page_down(height);
                panel.update_view_offset(height);
            }
            (KeyCode::Home, _) => {
                if !self.state.command_line.is_empty() {
                    self.state.command_cursor = 0;
                } else {
                    let panel = self.state.active_panel_mut();
                    panel.move_selection_home();
                    panel.update_view_offset(self.terminal.size()?.height as usize - 5);
                }
            }
            (KeyCode::End, _) => {
                if !self.state.command_line.is_empty() {
                    self.state.command_cursor = self.state.command_line.len();
                } else {
                    let panel = self.state.active_panel_mut();
                    panel.move_selection_end();
                    panel.update_view_offset(self.terminal.size()?.height as usize - 5);
                }
            }
            (KeyCode::Esc, _) => {
                // Clear command line
                self.state.command_line.clear();
                self.state.command_cursor = 0;
                self.state.command_history_index = None;
            }
            (KeyCode::Backspace, _) => {
                if self.state.command_cursor > 0 {
                    self.state.command_cursor -= 1;
                    self.state.command_line.remove(self.state.command_cursor);
                }
            }
            (KeyCode::Delete, _) => {
                if self.state.command_cursor < self.state.command_line.len() {
                    self.state.command_line.remove(self.state.command_cursor);
                }
            }
            (KeyCode::Char(' '), modifiers)
                if modifiers.is_empty() && self.state.command_line.is_empty() =>
            {
                // Space marks file when command line is empty
                let panel = self.state.active_panel_mut();
                panel.toggle_mark_current();
                panel.move_selection_down();
                panel.update_view_offset(self.terminal.size()?.height as usize - 5);
            }

            // Regular character input - goes to command line
            (KeyCode::Char('/'), _) if self.state.command_line.is_empty() => {
                // Show command palette when / is typed on empty command line
                self.dialog = Some(Dialog::CommandPalette(CommandPaletteDialog::new()));
            }
            (KeyCode::Char(c), _) => {
                self.state.command_line.insert(self.state.command_cursor, c);
                self.state.command_cursor += 1;
            }

            _ => {}
        }

        Ok(true)
    }

    async fn handle_special_command(&mut self, command: &str) -> Result<()> {
        // Debug log to track which command is being executed
        log::debug!("Executing special command: {}", command);

        match command {
            "exit" | "quit" | "q" => {
                // Properly cleanup and exit
                self.cleanup_and_exit().await?;
                std::process::exit(0);
            }
            "help" | "?" => {
                self.dialog = Some(Dialog::Help(HelpDialog::new()));
            }
            "reload" | "refresh" => {
                Self::refresh_panel(self.state.active_panel_mut())?;
                let inactive = match self.state.active_panel {
                    cortex_core::ActivePanel::Left => &mut self.state.right_panel,
                    cortex_core::ActivePanel::Right => &mut self.state.left_panel,
                };
                Self::refresh_panel(inactive)?;
                self.state.set_status_message("Panels reloaded");
            }
            "filter" => {
                let current_filter = self.state.active_panel().filter.clone();
                self.dialog = Some(Dialog::Filter(FilterDialog::with_current_filter(
                    current_filter.as_ref(),
                )));
            }
            "find" => {
                self.dialog = Some(Dialog::Search(SearchDialog::new()));
            }
            "sftp" => {
                self.dialog = Some(Dialog::Connection(
                    ConnectionDialog::new().with_type(ConnectionType::Sftp),
                ));
            }
            "ftp" => {
                self.dialog = Some(Dialog::Connection(
                    ConnectionDialog::new().with_type(ConnectionType::Ftp),
                ));
            }
            "plugin" | "plugins" => {
                let plugin_info = self.state.plugin_manager.get_plugin_info();
                let plugin_states: Vec<bool> = plugin_info
                    .iter()
                    .map(|p| self.state.plugin_manager.is_plugin_enabled(&p.name))
                    .collect();
                self.dialog = Some(Dialog::Plugin(PluginDialog::with_states(
                    plugin_info,
                    plugin_states,
                )));
            }
            "config" | "settings" | "preferences" => {
                let config = self.state.config_manager.get();
                self.dialog = Some(Dialog::Config(ConfigDialog::new(config)));
            }
            "monitor" => {
                if let Err(e) = self.state.toggle_auto_reload().await {
                    self.state
                        .set_status_message(format!("Error toggling file monitoring: {}", e));
                } else {
                    let status = if self.state.is_file_monitoring_active() {
                        "File monitoring ENABLED"
                    } else {
                        "File monitoring DISABLED"
                    };
                    self.state.set_status_message(status);
                }
            }
            "watch" => {
                let watched = self.state.get_watched_directories().await;
                if watched.is_empty() {
                    self.state
                        .set_status_message("No directories being watched");
                } else {
                    let dirs = watched
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.state.set_status_message(format!("Watching: {}", dirs));
                }
            }
            "notifications" => {
                self.notification_manager.toggle_visibility();
                let status = if self.notification_manager.is_visible() {
                    "Notifications ENABLED"
                } else {
                    "Notifications DISABLED"
                };
                self.state.set_status_message(status);
            }
            "view" => {
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.file_type == FileType::File {
                        match FileViewer::new(&entry.path) {
                            Ok(mut viewer) => {
                                let height =
                                    self.terminal.size().unwrap_or_default().height as usize;
                                let _ = viewer.load_content(height - 8);
                                self.dialog = Some(Dialog::Viewer(ViewerDialog::new(viewer)));
                            }
                            Err(e) => {
                                self.state.set_status_message(format!("Cannot view: {}", e));
                            }
                        }
                    }
                }
            }
            "edit" => {
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.file_type == FileType::File {
                        match TextEditor::new(&entry.path) {
                            Ok(editor) => {
                                self.dialog = Some(Dialog::Editor(EditorDialog::new(editor)));
                            }
                            Err(e) => {
                                self.state.set_status_message(format!("Cannot edit: {}", e));
                            }
                        }
                    }
                }
            }
            "copy" => {
                if let Some(operation) = OperationManager::prepare_copy(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            "move" => {
                if let Some(operation) = OperationManager::prepare_move(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            "delete" => {
                if let Some(operation) = OperationManager::prepare_delete(&self.state).await {
                    self.state.pending_operation = Some(operation.clone());
                    self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                }
            }
            "mkdir" => {
                self.dialog = Some(Dialog::Input(InputDialog::new(
                    "Create Directory",
                    "Enter directory name:",
                )));
                self.state.pending_operation = Some(FileOperation::CreateDir {
                    path: PathBuf::new(),
                });
            }
            "rename" => {
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.name != ".." {
                        self.dialog = Some(Dialog::Input(
                            InputDialog::new("Rename", "Enter new name:").with_value(&entry.name),
                        ));
                        self.state.pending_operation = Some(FileOperation::Rename {
                            old_path: entry.path.clone(),
                            new_name: entry.name.clone(),
                        });
                    }
                }
            }
            "hidden" => {
                let panel = self.state.active_panel_mut();
                panel.show_hidden = !panel.show_hidden;
                let show_hidden = panel.show_hidden;
                Self::refresh_panel(panel)?;
                self.state.set_status_message(format!(
                    "Hidden files: {}",
                    if show_hidden { "shown" } else { "hidden" }
                ));
            }
            "home" => {
                if let Some(home) = dirs::home_dir() {
                    let panel = self.state.active_panel_mut();
                    panel.current_dir = home;
                    panel.selected_index = 0;
                    panel.view_offset = 0;
                    Self::refresh_panel(panel)?;
                }
            }
            "root" => {
                let panel = self.state.active_panel_mut();
                panel.current_dir = PathBuf::from("/");
                panel.selected_index = 0;
                panel.view_offset = 0;
                Self::refresh_panel(panel)?;
            }
            cmd if cmd.starts_with("cd ") => {
                let path = &cmd[3..].trim();
                if let Some(new_dir) =
                    CommandProcessor::parse_cd_path(path, &self.state.active_panel().current_dir)
                {
                    let panel = self.state.active_panel_mut();
                    panel.current_dir = new_dir;
                    panel.selected_index = 0;
                    panel.view_offset = 0;
                    Self::refresh_panel(panel)?;
                } else {
                    self.state.set_status_message(format!(
                        "cd: cannot access '{}': No such directory",
                        path
                    ));
                }
            }
            _ => {
                // Try plugin commands first
                if self.handle_plugin_command(command).await? {
                    return Ok(()); // Plugin handled the command
                }

                self.state
                    .set_status_message(format!("Unknown command: /{}", command));
            }
        }
        Ok(())
    }

    async fn handle_dialog_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match &mut self.dialog {
            Some(Dialog::Confirm(dialog)) => match key.code {
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
            },
            Some(Dialog::Input(dialog)) => match key.code {
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
                    } else if let Some(FileOperation::Rename { old_path, .. }) =
                        &self.state.pending_operation
                    {
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
            },
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
            Some(Dialog::Help(dialog)) => match key.code {
                KeyCode::Up => dialog.scroll_up(),
                KeyCode::Down => dialog.scroll_down(),
                KeyCode::Esc | KeyCode::F(1) => {
                    self.dialog = None;
                }
                _ => {}
            },
            Some(Dialog::Editor(dialog)) => {
                if dialog.search_mode {
                    // Handle search input
                    match key.code {
                        KeyCode::Char(c) => {
                            dialog.search_input.push(c);
                        }
                        KeyCode::Backspace => {
                            dialog.search_input.pop();
                        }
                        KeyCode::Enter => {
                            if !dialog.search_input.is_empty() {
                                dialog.editor.search(&dialog.search_input);
                                dialog.search_mode = false;
                            }
                        }
                        KeyCode::Esc => {
                            dialog.search_mode = false;
                            dialog.search_input.clear();
                        }
                        _ => {}
                    }
                } else if dialog.replace_mode {
                    // Handle replace input
                    match key.code {
                        KeyCode::Char(c) => {
                            dialog.replace_input.push(c);
                        }
                        KeyCode::Backspace => {
                            dialog.replace_input.pop();
                        }
                        KeyCode::Enter => {
                            if !dialog.replace_input.is_empty()
                                && dialog.editor.search_term.is_some()
                            {
                                let search = dialog.editor.search_term.clone().unwrap();
                                dialog.editor.replace(&search, &dialog.replace_input, false);
                                dialog.replace_mode = false;
                            }
                        }
                        KeyCode::Esc => {
                            dialog.replace_mode = false;
                            dialog.replace_input.clear();
                        }
                        _ => {}
                    }
                } else {
                    // Normal editor controls
                    match (key.code, key.modifiers) {
                        (KeyCode::Esc, _) | (KeyCode::F(4), _) => {
                            if dialog.editor.modified {
                                // Show save confirmation dialog
                                let filename = dialog
                                    .editor
                                    .path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Untitled")
                                    .to_string();
                                // Store the editor dialog temporarily
                                self.pending_editor = Some(dialog.clone());
                                self.dialog =
                                    Some(Dialog::SaveConfirm(SaveConfirmDialog::new(filename)));
                            } else {
                                self.dialog = None;
                            }
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            let _ = dialog.editor.save();
                        }
                        (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                            dialog.search_mode = true;
                            dialog.search_input.clear();
                        }
                        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                            if dialog.editor.search_term.is_some() {
                                dialog.replace_mode = true;
                                dialog.replace_input.clear();
                            }
                        }
                        (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
                            dialog.editor.undo();
                        }
                        (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
                            dialog.editor.redo();
                        }
                        (KeyCode::Up, _) => {
                            dialog.editor.move_cursor_up();
                        }
                        (KeyCode::Down, _) => {
                            dialog.editor.move_cursor_down();
                        }
                        (KeyCode::Left, _) => {
                            dialog.editor.move_cursor_left();
                        }
                        (KeyCode::Right, _) => {
                            dialog.editor.move_cursor_right();
                        }
                        (KeyCode::Home, _) => {
                            dialog.editor.move_cursor_home();
                        }
                        (KeyCode::End, _) => {
                            dialog.editor.move_cursor_end();
                        }
                        (KeyCode::PageUp, _) => {
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            dialog.editor.move_cursor_page_up(height - 8);
                        }
                        (KeyCode::PageDown, _) => {
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            dialog.editor.move_cursor_page_down(height - 8);
                        }
                        (KeyCode::Enter, _) => {
                            dialog.editor.insert_newline();
                        }
                        (KeyCode::Backspace, _) => {
                            dialog.editor.delete_char();
                        }
                        (KeyCode::Delete, _) => {
                            dialog.editor.delete_forward();
                        }
                        (KeyCode::Char(c), _) => {
                            dialog.editor.insert_char(c);
                        }
                        _ => {}
                    }
                }
            }
            Some(Dialog::Viewer(dialog)) => {
                if dialog.search_mode {
                    // Handle search input
                    match key.code {
                        KeyCode::Char(c) => {
                            dialog.search_input.push(c);
                        }
                        KeyCode::Backspace => {
                            dialog.search_input.pop();
                        }
                        KeyCode::Enter => {
                            if !dialog.search_input.is_empty() {
                                dialog.viewer.search(&dialog.search_input);
                                dialog.search_mode = false;
                            }
                        }
                        KeyCode::Esc => {
                            dialog.search_mode = false;
                            dialog.search_input.clear();
                        }
                        _ => {}
                    }
                } else {
                    // Normal viewer controls
                    match key.code {
                        KeyCode::Up => {
                            dialog.viewer.scroll_up(1);
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            let _ = dialog.viewer.load_content(height - 8);
                        }
                        KeyCode::Down => {
                            dialog.viewer.scroll_down(1);
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            let _ = dialog.viewer.load_content(height - 8);
                        }
                        KeyCode::PageUp => {
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            dialog.viewer.page_up(height - 8);
                            let _ = dialog.viewer.load_content(height - 8);
                        }
                        KeyCode::PageDown => {
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            dialog.viewer.page_down(height - 8);
                            let _ = dialog.viewer.load_content(height - 8);
                        }
                        KeyCode::Char('h') | KeyCode::Char('H') => {
                            dialog.viewer.toggle_hex_mode();
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            let _ = dialog.viewer.load_content(height - 8);
                        }
                        KeyCode::Char('w') | KeyCode::Char('W') => {
                            dialog.viewer.toggle_wrap();
                            let height = self.terminal.size().unwrap_or_default().height as usize;
                            let _ = dialog.viewer.load_content(height - 8);
                        }
                        KeyCode::Char('/') => {
                            dialog.search_mode = true;
                            dialog.search_input.clear();
                        }
                        KeyCode::Char('f') | KeyCode::Char('F') | KeyCode::Char('n') => {
                            dialog.viewer.search_next();
                        }
                        KeyCode::Esc | KeyCode::F(3) | KeyCode::Char('q') => {
                            self.dialog = None;
                        }
                        _ => {}
                    }
                }
            }
            Some(Dialog::Filter(dialog)) => {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                        dialog.clear();
                        let panel = self.state.active_panel_mut();
                        panel.clear_filter();
                    }
                    (KeyCode::Char(c), _) => {
                        dialog.insert_char(c);
                        // Real-time filtering
                        let panel = self.state.active_panel_mut();
                        panel.apply_filter(&dialog.input);
                    }
                    (KeyCode::Backspace, _) => {
                        dialog.delete_char();
                        // Real-time filtering
                        let panel = self.state.active_panel_mut();
                        panel.apply_filter(&dialog.input);
                    }
                    (KeyCode::Left, _) => {
                        dialog.move_cursor_left();
                    }
                    (KeyCode::Right, _) => {
                        dialog.move_cursor_right();
                    }
                    (KeyCode::Enter, _) => {
                        // Apply filter and close dialog
                        self.dialog = None;
                    }
                    (KeyCode::Esc, _) => {
                        // Cancel filter
                        let panel = self.state.active_panel_mut();
                        panel.clear_filter();
                        self.dialog = None;
                    }
                    _ => {}
                }
            }
            Some(Dialog::CommandPalette(dialog)) => {
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
                    KeyCode::Up => {
                        dialog.move_selection_up();
                    }
                    KeyCode::Down => {
                        dialog.move_selection_down();
                    }
                    KeyCode::Tab => {
                        // Autocomplete - select current command
                        if let Some(cmd) = dialog.get_selected_command() {
                            dialog.input = cmd;
                            dialog.cursor_position = dialog.input.len();
                            dialog.filter_commands();
                        }
                    }
                    KeyCode::Enter => {
                        // Execute selected command
                        if let Some(cmd) = dialog.get_selected_command() {
                            self.dialog = None;
                            // Remove the leading /
                            let command = if cmd.starts_with('/') {
                                &cmd[1..]
                            } else {
                                &cmd
                            };
                            self.handle_special_command(command).await?;
                        }
                    }
                    KeyCode::Esc => {
                        self.dialog = None;
                    }
                    _ => {}
                }
            }
            Some(Dialog::Search(dialog)) => {
                match dialog.state {
                    SearchState::Setup => {
                        // Handle search setup input
                        match key.code {
                            KeyCode::Enter => {
                                if !dialog.criteria.pattern.is_empty() {
                                    // Start search execution
                                    dialog.state = SearchState::Searching;
                                    dialog.results.clear();
                                    dialog.search_progress = None;

                                    // Execute search in background
                                    let criteria = dialog.criteria.clone();
                                    let search_path = self.state.active_panel().current_dir.clone();

                                    let (tx, rx) = mpsc::unbounded_channel();
                                    let search_criteria = criteria.clone();

                                    // Store the receiver
                                    self.search_rx = Some(rx);

                                    tokio::spawn(async move {
                                        use cortex_core::SearchEngine;

                                        match SearchEngine::new(search_criteria) {
                                            Ok(mut engine) => {
                                                let _ = engine.search(&search_path, tx).await;
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "Failed to create search engine: {}",
                                                    e
                                                );
                                            }
                                        }
                                    });

                                    self.state.set_status_message(format!(
                                        "Searching for '{}'...",
                                        criteria.pattern
                                    ));
                                }
                            }
                            KeyCode::Esc => {
                                self.dialog = None;
                            }
                            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Toggle options with Ctrl+Space
                                dialog.toggle_case_sensitive();
                            }
                            KeyCode::Char(c) => {
                                dialog.criteria.pattern.push(c);
                            }
                            KeyCode::Backspace => {
                                dialog.criteria.pattern.pop();
                            }
                            KeyCode::Tab => {
                                // Cycle through input fields
                            }
                            _ => {}
                        }
                    }
                    SearchState::Searching => {
                        // Handle search in progress
                        match key.code {
                            KeyCode::Esc => {
                                // Cancel search
                                self.search_rx = None; // Drop the receiver to stop processing
                                dialog.state = SearchState::Results; // Show results collected so far
                                self.state.set_status_message(format!(
                                    "Search cancelled: {} results found",
                                    dialog.results.len()
                                ));
                            }
                            _ => {}
                        }
                    }
                    SearchState::Results => {
                        // Handle search results navigation
                        match key.code {
                            KeyCode::Up => dialog.move_selection_up(),
                            KeyCode::Down => dialog.move_selection_down(),
                            KeyCode::Enter => {
                                // Navigate to selected file
                                if let Some(path) = dialog.get_selected_path() {
                                    if let Some(parent) = path.parent() {
                                        let panel = self.state.active_panel_mut();
                                        panel.current_dir = parent.to_path_buf();
                                        Self::refresh_panel(panel)?;
                                    }
                                }
                                self.dialog = None;
                            }
                            KeyCode::F(7) => {
                                // New search
                                self.dialog = Some(Dialog::Search(SearchDialog::new()));
                            }
                            KeyCode::Esc => {
                                self.dialog = None;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Some(Dialog::Connection(dialog)) => {
                match key.code {
                    KeyCode::Tab => {
                        dialog.next_field();
                    }
                    KeyCode::BackTab => {
                        dialog.prev_field();
                    }
                    KeyCode::Char(c) if key.modifiers == KeyModifiers::CONTROL && c == 't' => {
                        dialog.toggle_auth_method();
                    }
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
                        // Attempt to connect
                        if !dialog.host.is_empty() && !dialog.username.is_empty() {
                            let credentials = RemoteCredentials {
                                username: dialog.username.clone(),
                                password: if dialog.use_private_key || dialog.password.is_empty() {
                                    None
                                } else {
                                    Some(dialog.password.clone())
                                },
                                private_key_path: if dialog.use_private_key
                                    && !dialog.private_key_path.is_empty()
                                {
                                    Some(std::path::PathBuf::from(&dialog.private_key_path))
                                } else {
                                    None
                                },
                                passphrase: if dialog.use_private_key && !dialog.password.is_empty()
                                {
                                    Some(dialog.password.clone())
                                } else {
                                    None
                                },
                            };

                            let port = dialog.port.parse::<u16>().unwrap_or(
                                match dialog.connection_type {
                                    ConnectionType::Sftp => 22,
                                    ConnectionType::Ftp => 21,
                                },
                            );

                            let host = dialog.host.clone();
                            let username = dialog.username.clone();

                            match dialog.connection_type {
                                ConnectionType::Sftp => {
                                    let vfs_path = VfsPath::Sftp {
                                        host: host.clone(),
                                        port,
                                        username: username.clone(),
                                        path: "/".to_string(),
                                    };

                                    // Try to connect and navigate
                                    match self.connect_sftp(&credentials, &vfs_path).await {
                                        Ok(_) => {
                                            // Store credentials for future use
                                            self.state.store_connection_credentials(
                                                &host,
                                                port,
                                                &username,
                                                credentials,
                                            );
                                            self.dialog = None;
                                            self.state.set_status_message(format!(
                                                "Connected to {}",
                                                host
                                            ));
                                        }
                                        Err(e) => {
                                            self.dialog = Some(Dialog::Error(ErrorDialog::new(
                                                format!("Connection failed: {}", e),
                                            )));
                                        }
                                    }
                                }
                                ConnectionType::Ftp => {
                                    // FTP support shows a message for now
                                    self.state.set_status_message(
                                        "FTP support is under development - use SFTP for now",
                                    );
                                    self.dialog = None;
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.dialog = None;
                    }
                    _ => {}
                }
            }
            Some(Dialog::Plugin(dialog)) => {
                match key.code {
                    KeyCode::Up => {
                        dialog.move_selection_up();
                    }
                    KeyCode::Down => {
                        dialog.move_selection_down();
                    }
                    KeyCode::Enter => {
                        dialog.toggle_details();
                    }
                    KeyCode::Backspace if dialog.show_details => {
                        dialog.show_details = false;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // Reload plugins
                        if let Err(e) = Self::load_plugins(&mut self.state).await {
                            self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                                "Failed to reload plugins: {}",
                                e
                            ))));
                        } else {
                            let plugin_info = self.state.plugin_manager.get_plugin_info();
                            let plugin_states: Vec<bool> = plugin_info
                                .iter()
                                .map(|p| self.state.plugin_manager.is_plugin_enabled(&p.name))
                                .collect();
                            *dialog = PluginDialog::with_states(plugin_info, plugin_states);
                            self.state.set_status_message("Plugins reloaded");
                        }
                    }
                    KeyCode::Char(' ') => {
                        // Toggle plugin enable/disable
                        if let Some(plugin) = dialog.get_selected_plugin() {
                            let plugin_name = plugin.name.clone();
                            match self.state.plugin_manager.toggle_plugin(&plugin_name) {
                                Ok(new_state) => {
                                    dialog.toggle_selected_plugin();
                                    let status = if new_state { "enabled" } else { "disabled" };
                                    self.state.set_status_message(format!(
                                        "Plugin '{}' {}",
                                        plugin_name, status
                                    ));
                                }
                                Err(e) => {
                                    self.state.set_status_message(format!(
                                        "Failed to toggle plugin: {}",
                                        e
                                    ));
                                }
                            }
                        }
                    }
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
                                self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                                    "Failed to save configuration: {}",
                                    e
                                ))));
                            } else {
                                Self::apply_configuration(&mut self.state);
                                Self::refresh_panel_with_cache(
                                    &mut self.state.left_panel,
                                    &self.state.directory_cache,
                                )
                                .ok();
                                Self::refresh_panel_with_cache(
                                    &mut self.state.right_panel,
                                    &self.state.directory_cache,
                                )
                                .ok();
                                self.state
                                    .set_status_message("Configuration saved and applied");
                            }
                        }
                        KeyCode::Esc => {
                            self.dialog = None;
                        }
                        _ => {}
                    }
                }
            }
            Some(Dialog::SaveConfirm(dialog)) => {
                match key.code {
                    KeyCode::Left | KeyCode::Right => {
                        dialog.next_choice();
                    }
                    KeyCode::Tab => {
                        dialog.next_choice();
                    }
                    KeyCode::BackTab => {
                        dialog.prev_choice();
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        // Quick select Save
                        if let Some(mut editor_dialog) = self.pending_editor.take() {
                            let _ = editor_dialog.editor.save();
                        }
                        self.dialog = None;
                        self.pending_editor = None;
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        // Quick select Don't Save - close without saving
                        self.dialog = None;
                        self.pending_editor = None;
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => {
                        // Cancel - return to editor
                        if let Some(editor_dialog) = self.pending_editor.take() {
                            self.dialog = Some(Dialog::Editor(editor_dialog));
                        } else {
                            self.dialog = None;
                        }
                    }
                    KeyCode::Enter => {
                        match dialog.selection {
                            SaveChoice::Save => {
                                // Save and close
                                if let Some(mut editor_dialog) = self.pending_editor.take() {
                                    let _ = editor_dialog.editor.save();
                                }
                                self.dialog = None;
                                self.pending_editor = None;
                            }
                            SaveChoice::DontSave => {
                                // Close without saving
                                self.dialog = None;
                                self.pending_editor = None;
                            }
                            SaveChoice::Cancel => {
                                // Return to editor
                                if let Some(editor_dialog) = self.pending_editor.take() {
                                    self.dialog = Some(Dialog::Editor(editor_dialog));
                                } else {
                                    self.dialog = None;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            None => {}
        }

        Ok(true)
    }

    async fn connect_sftp(
        &mut self,
        _credentials: &RemoteCredentials,
        _vfs_path: &VfsPath,
    ) -> Result<()> {
        // SSH/SFTP support temporarily disabled - requires OpenSSL
        Err(anyhow::anyhow!("SSH/SFTP connections are not available in this build. Please install OpenSSL development packages."))
    }

    async fn _connect_ftp(
        &mut self,
        _credentials: &RemoteCredentials,
        _vfs_path: &VfsPath,
    ) -> Result<()> {
        // FTP support temporarily disabled - requires OpenSSL
        Err(anyhow::anyhow!("FTP connections are not available in this build. Please install OpenSSL development packages."))
    }

    async fn execute_operation(&mut self, operation: FileOperation) -> Result<()> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.operation_rx = Some(rx);

        let title = match &operation {
            FileOperation::Copy { .. } => "Copying Files",
            FileOperation::Move { .. } => "Moving Files",
            FileOperation::Delete { .. } => "Deleting Files",
            FileOperation::DeleteToTrash { .. } => "Moving to Trash",
            FileOperation::RestoreFromTrash { .. } => "Restoring from Trash",
            FileOperation::CreateDir { .. } => "Creating Directory",
            FileOperation::Rename { .. } => "Renaming",
            FileOperation::CopyToClipboard { .. } => "Copying to Clipboard",
            FileOperation::PasteFromClipboard { .. } => "Pasting from Clipboard",
        };

        self.dialog = Some(Dialog::Progress(ProgressDialog::new(
            title,
            "Processing...",
        )));

        let result = self
            .operation_manager
            .execute_operation(operation, tx)
            .await;

        if let Err(e) = result {
            self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                "Operation failed: {}",
                e
            ))));
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
                cortex_core::OperationProgress::Progress {
                    current,
                    total,
                    message,
                } => {
                    dialog.update(current, total, message);
                }
                cortex_core::OperationProgress::Completed { .. } => {
                    dialog.message = "Operation completed".to_string();
                }
                cortex_core::OperationProgress::Failed { operation, error } => {
                    self.dialog = Some(Dialog::Error(
                        ErrorDialog::new(format!("Failed: {}", error)).with_details(operation),
                    ));
                }
            }
        }
    }

    fn handle_search_progress(&mut self, progress: cortex_core::SearchProgress) {
        use cortex_core::SearchProgress;
        use cortex_tui::SearchProgressInfo;

        if let Some(Dialog::Search(ref mut dialog)) = self.dialog {
            match progress {
                SearchProgress::Started { total_dirs } => {
                    dialog.state = SearchState::Searching;
                    dialog.search_progress = Some(SearchProgressInfo {
                        current_path: PathBuf::new(),
                        searched: 0,
                        total: total_dirs,
                        found: 0,
                    });
                }
                SearchProgress::Searching {
                    current_path,
                    searched,
                    total,
                } => {
                    if let Some(ref mut prog) = dialog.search_progress {
                        prog.current_path = current_path;
                        prog.searched = searched;
                        prog.total = total;
                        prog.found = dialog.results.len();
                    }
                }
                SearchProgress::Found { result } => {
                    dialog.results.push(result);
                    if let Some(ref mut prog) = dialog.search_progress {
                        prog.found = dialog.results.len();
                    }
                }
                SearchProgress::Completed {
                    total_found,
                    elapsed_ms,
                } => {
                    dialog.state = SearchState::Results;
                    dialog.selected_result = 0;
                    self.state.set_status_message(format!(
                        "Search completed: {} results in {}ms",
                        total_found, elapsed_ms
                    ));
                    self.search_rx = None;
                }
                SearchProgress::Error { path, error } => {
                    log::warn!("Search error at {:?}: {}", path, error);
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

    fn refresh_panel_with_cache(
        panel: &mut cortex_core::PanelState,
        cache: &DirectoryCache,
    ) -> Result<()> {
        // Try to get from cache first
        let entries = if let Some(cached_entries) = cache.get(&panel.current_dir) {
            log::debug!("Cache hit for directory: {:?}", panel.current_dir);
            cached_entries
        } else {
            log::debug!("Cache miss for directory: {:?}", panel.current_dir);
            let fresh_entries = FileSystem::list_directory(&panel.current_dir, panel.show_hidden)?;

            // Store in cache for future use
            if let Err(e) = cache.put(&panel.current_dir, fresh_entries.clone()) {
                log::warn!("Failed to cache directory entries: {}", e);
            }

            fresh_entries
        };

        panel.entries = entries;
        panel.sort_entries();

        if panel.selected_index >= panel.entries.len() && !panel.entries.is_empty() {
            panel.selected_index = panel.entries.len() - 1;
        }

        Ok(())
    }

    async fn load_plugins(state: &mut AppState) -> Result<()> {
        let plugin_dir = std::env::current_dir()?.join("plugins");

        if !plugin_dir.exists() {
            return Ok(()); // No plugins directory, that's fine
        }

        for entry in std::fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("lua") {
                match LuaPlugin::new(path.clone()) {
                    Ok(mut plugin) => {
                        if let Err(e) = plugin.load_script() {
                            eprintln!("Warning: Failed to load plugin '{}': {}", path.display(), e);
                            continue;
                        }

                        let plugin_name = plugin.info().name.clone();

                        match state.plugin_manager.load_plugin(Box::new(plugin)).await {
                            Ok(_) => println!("Loaded plugin: {}", plugin_name),
                            Err(e) => eprintln!(
                                "Warning: Failed to register plugin '{}': {}",
                                plugin_name, e
                            ),
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to create plugin from '{}': {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        // Initialize all loaded plugins
        let context = state.create_plugin_context();
        if let Err(e) = state.plugin_manager.initialize_all(context).await {
            eprintln!("Warning: Failed to initialize plugins: {}", e);
        }

        Ok(())
    }

    async fn handle_plugin_command(&mut self, command: &str) -> Result<bool> {
        let context = self.state.create_plugin_context();

        match self
            .state
            .plugin_manager
            .execute_command(command, Vec::new(), context)
            .await
        {
            Ok(result) => {
                if !result.is_empty() {
                    // Show plugin result in a dialog
                    self.state
                        .set_status_message(format!("Plugin {}: {}", command, result));
                }
                Ok(true)
            }
            Err(_) => Ok(false), // Command not handled by any plugin
        }
    }

    async fn _fire_plugin_event(&mut self, event: PluginEvent) {
        let context = self.state.create_plugin_context();
        if let Err(e) = self.state.plugin_manager.handle_event(event, context).await {
            eprintln!("Warning: Plugin event handling failed: {}", e);
        }
    }

    async fn init_file_monitor_with_callback(
        state: &mut AppState,
        file_change_tx: mpsc::UnboundedSender<()>,
        file_event_tx: mpsc::UnboundedSender<cortex_core::FileMonitorEvent>,
    ) -> Result<()> {
        if state.auto_reload_enabled {
            use cortex_core::{ChangeNotification, EventCallback, FileMonitorManager};
            use std::sync::Arc;

            let monitor_manager = Arc::new(FileMonitorManager::new().await?);
            monitor_manager.start().await?;

            // Set up callback for panel refresh and notifications
            let callback: EventCallback = Arc::new(move |notification: ChangeNotification| {
                log::debug!(
                    "File change detected: {} - {:?}",
                    notification.path.display(),
                    notification.event
                );
                let _ = file_change_tx.send(());
                let _ = file_event_tx.send(notification.event);
            });

            monitor_manager.register_change_callback(callback).await;

            // Watch current directories
            monitor_manager
                .watch_directory(&state.left_panel.current_dir, false)
                .await?;
            monitor_manager
                .watch_directory(&state.right_panel.current_dir, false)
                .await?;

            state.file_monitor = Some(monitor_manager);
        }
        Ok(())
    }

    async fn navigate_to_directory(&mut self, new_path: PathBuf) -> Result<()> {
        let current_panel = self.state.active_panel;
        let _old_path = self.state.active_panel().current_dir.clone();

        // Update the panel
        let panel = self.state.active_panel_mut();
        panel.current_dir = new_path.clone();
        panel.selected_index = 0;
        panel.view_offset = 0;
        Self::refresh_panel(panel)?;

        // Update file monitoring if active
        if self.state.is_file_monitoring_active() {
            if let Err(e) = self
                .state
                .update_file_monitoring(current_panel, &new_path)
                .await
            {
                log::warn!("Failed to update file monitoring: {}", e);
            }
        }

        Ok(())
    }

    fn handle_file_event(&mut self, event: cortex_core::FileMonitorEvent) {
        use cortex_core::FileMonitorEvent;

        match event {
            FileMonitorEvent::Created(path) => {
                self.notification_manager
                    .add_file_change(&path, NotificationType::FileCreated);
            }
            FileMonitorEvent::Modified(path) => {
                self.notification_manager
                    .add_file_change(&path, NotificationType::FileModified);
            }
            FileMonitorEvent::Deleted(path) => {
                self.notification_manager
                    .add_file_change(&path, NotificationType::FileDeleted);
            }
            FileMonitorEvent::Renamed { from, to } => {
                self.notification_manager.add_file_rename(&from, &to);
            }
        }
    }

    fn apply_configuration(state: &mut AppState) {
        let config = state.config_manager.get();

        // Apply general settings
        state.left_panel.show_hidden = config.general.show_hidden;
        state.right_panel.show_hidden = config.general.show_hidden;

        // Apply sort settings
        let sort_mode = match config.panels.default_sort.as_str() {
            "name" => cortex_core::SortMode::Name,
            "size" => cortex_core::SortMode::Size,
            "modified" => cortex_core::SortMode::Modified,
            "extension" => cortex_core::SortMode::Extension,
            _ => cortex_core::SortMode::Name,
        };
        state.left_panel.sort_mode = sort_mode;
        state.right_panel.sort_mode = sort_mode;

        // Re-sort panels with new settings
        state.left_panel.sort_entries();
        state.right_panel.sort_entries();

        println!(
            "Configuration applied: show_hidden={}, default_sort={}",
            config.general.show_hidden, config.panels.default_sort
        );
    }

    async fn handle_mouse_event(
        &mut self,
        mouse_event: crossterm::event::MouseEvent,
    ) -> Result<()> {
        if let Some(action) = self.mouse_handler.process_event(mouse_event) {
            match action {
                MouseAction::Click(pos) => {
                    self.handle_mouse_click(pos).await?;
                }
                MouseAction::DoubleClick(pos) => {
                    self.handle_mouse_double_click(pos).await?;
                }
                MouseAction::RightClick(pos) => {
                    self.handle_mouse_right_click(pos).await?;
                }
                MouseAction::ScrollUp(pos) => {
                    self.handle_mouse_scroll_up(pos).await?;
                }
                MouseAction::ScrollDown(pos) => {
                    self.handle_mouse_scroll_down(pos).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_mouse_click(&mut self, pos: Position) -> Result<()> {
        // Close context menu if open
        if self.context_menu.is_some() {
            self.context_menu = None;
            return Ok(());
        }

        // Determine which panel was clicked
        let terminal_size = self.terminal.size()?;
        let panel_width = terminal_size.width / 2;

        if pos.x < panel_width {
            // Left panel clicked
            self.state.active_panel = cortex_core::ActivePanel::Left;

            // Calculate which file was clicked (accounting for panel offset)
            if pos.y >= 2 && pos.y < terminal_size.height - 3 {
                let file_index = (pos.y - 2) as usize + self.state.left_panel.view_offset;
                if file_index < self.state.left_panel.entries.len() {
                    self.state.left_panel.selected_index = file_index;
                }
            }
        } else {
            // Right panel clicked
            self.state.active_panel = cortex_core::ActivePanel::Right;

            // Calculate which file was clicked
            if pos.y >= 2 && pos.y < terminal_size.height - 3 {
                let file_index = (pos.y - 2) as usize + self.state.right_panel.view_offset;
                if file_index < self.state.right_panel.entries.len() {
                    self.state.right_panel.selected_index = file_index;
                }
            }
        }

        Ok(())
    }

    async fn handle_mouse_double_click(&mut self, pos: Position) -> Result<()> {
        // First handle as a click to select the item
        self.handle_mouse_click(pos).await?;

        // Then open/enter the selected item
        let panel = self.state.active_panel_mut();
        if let Some(entry) = panel.current_entry() {
            if entry.file_type == FileType::Directory {
                let path = entry.path.clone();
                self.navigate_to_directory(path).await?;
            } else {
                // View the file
                let path = entry.path.clone();
                if let Ok(mut viewer) = FileViewer::new(&path) {
                    if viewer.load_content(100).is_ok() {
                        self.dialog = Some(Dialog::Viewer(ViewerDialog::new(viewer)));
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_mouse_right_click(&mut self, pos: Position) -> Result<()> {
        // Close existing context menu if any
        self.context_menu = None;

        // Determine which panel was clicked
        let terminal_size = self.terminal.size()?;
        let panel_width = terminal_size.width / 2;

        let has_selection = if pos.x < panel_width {
            self.state.active_panel = cortex_core::ActivePanel::Left;
            !self.state.left_panel.marked_files.is_empty()
        } else {
            self.state.active_panel = cortex_core::ActivePanel::Right;
            !self.state.right_panel.marked_files.is_empty()
        };

        // Check if clicked on a file or empty space
        let clicked_on_file = if pos.y >= 2 && pos.y < terminal_size.height - 3 {
            let panel = self.state.active_panel();
            let file_index = (pos.y - 2) as usize + panel.view_offset;
            file_index < panel.entries.len()
        } else {
            false
        };

        // Create appropriate context menu
        if clicked_on_file {
            self.context_menu = Some(ContextMenu::file_menu(pos, has_selection));
        } else {
            self.context_menu = Some(ContextMenu::panel_menu(pos));
        }

        Ok(())
    }

    async fn handle_mouse_scroll_up(&mut self, pos: Position) -> Result<()> {
        // Determine which panel to scroll
        let terminal_size = self.terminal.size()?;
        let panel_width = terminal_size.width / 2;

        let panel = if pos.x < panel_width {
            &mut self.state.left_panel
        } else {
            &mut self.state.right_panel
        };

        // Scroll up by 3 lines
        for _ in 0..3 {
            panel.move_selection_up();
        }
        panel.update_view_offset(terminal_size.height as usize - 5);

        Ok(())
    }

    async fn handle_mouse_scroll_down(&mut self, pos: Position) -> Result<()> {
        // Determine which panel to scroll
        let terminal_size = self.terminal.size()?;
        let panel_width = terminal_size.width / 2;

        let panel = if pos.x < panel_width {
            &mut self.state.left_panel
        } else {
            &mut self.state.right_panel
        };

        // Scroll down by 3 lines
        for _ in 0..3 {
            panel.move_selection_down();
        }
        panel.update_view_offset(terminal_size.height as usize - 5);

        Ok(())
    }

    async fn handle_context_menu_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        if let Some(ref mut menu) = self.context_menu {
            match key.code {
                KeyCode::Up => menu.move_up(),
                KeyCode::Down => menu.move_down(),
                KeyCode::Enter => {
                    if let Some(action) = menu.get_selected_action() {
                        self.execute_context_menu_action(action).await?;
                    }
                    self.context_menu = None;
                }
                KeyCode::Esc => {
                    self.context_menu = None;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn execute_context_menu_action(&mut self, action: ContextMenuAction) -> Result<()> {
        match action {
            ContextMenuAction::Copy => {
                // Prepare copy operation (F5 functionality)
                let sources = self.get_selected_files();
                if !sources.is_empty() {
                    let destination = self.state.inactive_panel().current_dir.clone();
                    self.state.pending_operation = Some(FileOperation::Copy {
                        sources: sources.clone(),
                        destination: destination.clone(),
                    });
                    self.state
                        .set_status_message(format!("Ready to copy {} file(s)", sources.len()));
                }
            }
            ContextMenuAction::Cut => {
                // Prepare move operation
                let sources = self.get_selected_files();
                if !sources.is_empty() {
                    let destination = self.state.inactive_panel().current_dir.clone();
                    self.state.pending_operation = Some(FileOperation::Move {
                        sources: sources.clone(),
                        destination: destination.clone(),
                    });
                    self.state
                        .set_status_message(format!("Ready to move {} file(s)", sources.len()));
                }
            }
            ContextMenuAction::Paste => {
                // Execute pending operation
                if let Some(ref op) = self.state.pending_operation.clone() {
                    match op {
                        FileOperation::Copy { sources, .. } => {
                            let destination = self.state.active_panel().current_dir.clone();
                            self.operation_manager
                                .copy_files(sources.clone(), destination)
                                .await?;
                        }
                        FileOperation::Move { sources, .. } => {
                            let destination = self.state.active_panel().current_dir.clone();
                            self.operation_manager
                                .move_files(sources.clone(), destination)
                                .await?;
                        }
                        _ => {}
                    }
                }
            }
            ContextMenuAction::Delete => {
                // Delete to trash operation
                let targets = self.get_selected_files();
                if !targets.is_empty() {
                    self.dialog = Some(Dialog::Confirm(cortex_tui::dialogs::ConfirmDialog::new(
                        "Move to Trash",
                        format!("Move {} file(s) to trash?", targets.len()),
                    )));
                    self.state.pending_operation = Some(FileOperation::DeleteToTrash { targets });
                }
            }
            ContextMenuAction::Rename => {
                if let Some(entry) = self.state.active_panel().current_entry() {
                    self.dialog = Some(Dialog::Input(
                        InputDialog::new("Rename", "Enter new name:").with_value(&entry.name),
                    ));
                    self.state.pending_operation = Some(FileOperation::Rename {
                        old_path: entry.path.clone(),
                        new_name: entry.name.clone(),
                    });
                }
            }
            ContextMenuAction::NewFile => {
                self.dialog = Some(Dialog::Input(InputDialog::new(
                    "New File",
                    "Enter file name:",
                )));
            }
            ContextMenuAction::NewFolder => {
                self.dialog = Some(Dialog::Input(InputDialog::new(
                    "New Directory",
                    "Enter directory name:",
                )));
            }
            ContextMenuAction::ViewFile => {
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if let Ok(mut viewer) = FileViewer::new(&entry.path) {
                        if viewer.load_content(100).is_ok() {
                            self.dialog = Some(Dialog::Viewer(ViewerDialog::new(viewer)));
                        }
                    }
                }
            }
            ContextMenuAction::EditFile => {
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if let Ok(editor) = TextEditor::new(&entry.path) {
                        self.dialog = Some(Dialog::Editor(EditorDialog::new(editor)));
                    }
                }
            }
            ContextMenuAction::Refresh => {
                let left_dir = self.state.left_panel.current_dir.clone();
                let right_dir = self.state.right_panel.current_dir.clone();
                self.state.directory_cache.invalidate(&left_dir);
                self.state.directory_cache.invalidate(&right_dir);

                Self::refresh_panel_with_cache(
                    &mut self.state.left_panel,
                    &self.state.directory_cache,
                )?;
                Self::refresh_panel_with_cache(
                    &mut self.state.right_panel,
                    &self.state.directory_cache,
                )?;
            }
            ContextMenuAction::SelectAll => {
                let panel = self.state.active_panel_mut();
                for entry in &panel.entries {
                    if entry.name != ".." {
                        panel.marked_files.push(entry.path.clone());
                    }
                }
            }
            ContextMenuAction::InvertSelection => {
                let panel = self.state.active_panel_mut();
                let mut new_marks = Vec::new();
                for entry in &panel.entries {
                    if entry.name != ".." && !panel.marked_files.contains(&entry.path) {
                        new_marks.push(entry.path.clone());
                    }
                }
                panel.marked_files = new_marks;
            }
            _ => {}
        }
        Ok(())
    }

    fn get_selected_files(&self) -> Vec<PathBuf> {
        let panel = self.state.active_panel();
        if !panel.marked_files.is_empty() {
            panel.marked_files.clone()
        } else if let Some(entry) = panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    async fn cleanup_and_exit(&mut self) -> Result<()> {
        // Stop file monitoring if active
        if let Some(ref monitor) = self.state.file_monitor {
            if let Err(e) = monitor.stop().await {
                log::warn!("Failed to stop file monitor: {}", e);
            }
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        // Force exit all spawned tasks by terminating the process
        // This ensures no background tasks keep running
        std::process::exit(0);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}
