use anyhow::Result;
use clap::Parser;
use cortex_core::window::{WindowConfig, WindowManager, WindowMode};
use std::path::PathBuf;

mod app;
mod command;
mod operations;
mod update;

use app::App;
use update::UpdateManager;

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

    #[arg(short = 'w', long, help = "Run in windowed mode (opens in new window)")]
    windowed: bool,

    #[arg(short = 't', long, help = "Force terminal mode (stay in current terminal)")]
    terminal: bool,

    #[arg(long, help = "Start in fullscreen mode")]
    fullscreen: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    if args.version {
        println!("Cortex v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Determine window mode
    let window_mode = if args.terminal || !args.windowed {
        WindowMode::Terminal
    } else if args.fullscreen {
        WindowMode::Fullscreen
    } else if args.windowed {
        WindowMode::Windowed
    } else {
        WindowMode::Terminal
    };

    // Handle windowed mode (experimental)
    if window_mode != WindowMode::Terminal {
        #[cfg(target_os = "macos")]
        {
            return run_windowed_app_sync(args.path, window_mode);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let runtime = tokio::runtime::Runtime::new()?;
            return runtime.block_on(run_windowed_app(args.path, window_mode));
        }
    }

    // Run the terminal app
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_main(args))
}

async fn async_main(args: Args) -> Result<()> {
    // Handle update operations
    if args.check_updates || args.update {
        return handle_update_operations(args.check_updates, args.update).await;
    }

    // Create and run the main application
    let mut app = App::new(args.path).await?;
    app.run().await
}

async fn handle_update_operations(check_updates: bool, update: bool) -> Result<()> {
    let manager = UpdateManager::new()?;

    if check_updates {
        println!("Checking for updates...");
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

    if update {
        println!("Checking for updates...");
        match manager.check_for_updates().await {
            Ok(Some(update_info)) => {
                println!("Found update: v{}", update_info.version);
                println!("Downloading...");

                if let Err(e) = manager.install_update(update_info).await {
                    eprintln!("Failed to install update: {}", e);
                } else {
                    println!("Update installed successfully!");
                    println!("Please restart Cortex to use the new version");
                }
            }
            Ok(None) => {
                println!("You are already running the latest version");
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn run_windowed_app_sync(_initial_path: Option<PathBuf>, mode: WindowMode) -> Result<()> {
    println!("Starting Cortex in windowed mode (macOS)...");

    let config = WindowConfig {
        title: format!("Cortex File Manager v{}", env!("CARGO_PKG_VERSION")),
        width: 1280,
        height: 800,
        mode,
        resizable: true,
        decorations: true,
    };

    let mut manager = WindowManager::new(config.clone());
    let _window = manager.create_window()?;
    manager.run_event_loop()?;

    Ok(())
}

#[allow(dead_code)]
async fn run_windowed_app(_initial_path: Option<PathBuf>, _mode: WindowMode) -> Result<()> {
    println!("Windowed mode is not yet fully implemented for this platform");
    println!("Please use terminal mode instead: cortex --terminal");
    Ok(())
}
        height: 800,
        mode,
        resizable: true,
        decorations: true,
    };

    // Use spawn_window_thread to create window in its own thread
    let mut event_rx = WindowManager::spawn_window_thread(config.clone())?;

    // Initialize renderer (simplified for now)
    let mut renderer = TerminalRenderer::new(config.width, config.height);

    // Create app state
    let mut state = AppState::new()?;
    if let Some(path) = initial_path {
        state.left_panel.current_dir = path.clone();
        state.left_panel.entries = cortex_core::FileSystem::list_directory(&path, false)?;
    }

    // Initialize terminal manager for embedded terminals
    let terminal_manager = Arc::new(TerminalManager::new());
    state.terminal_manager = Some(terminal_manager.clone());

    // Create a channel for app events
    let (app_tx, mut app_rx) = mpsc::unbounded_channel::<AppEvent>();

    // Main application loop
    loop {
        // Process window events
        while let Ok(event) = event_rx.try_recv() {
            match event {
                WinEvent::Close => {
                    println!("Window closed");
                    return Ok(());
                }
                WinEvent::Resize(width, height) => {
                    renderer.resize(width, height)?;
                }
                WinEvent::KeyPress(ch) => {
                    // Handle character input
                    let _ = app_tx.send(AppEvent::KeyPress(ch));
                }
                WinEvent::KeyDown(keycode) => {
                    // Handle special keys
                    let _ = app_tx.send(AppEvent::KeyDown(keycode));
                }
                WinEvent::MouseClick(x, y, button) => {
                    // Handle mouse clicks
                    let _ = app_tx.send(AppEvent::MouseClick(x, y, button));
                }
                WinEvent::Redraw => {
                    // Render current state
                    render_windowed_ui(&mut renderer, &state)?;
                }
                _ => {}
            }
        }

        // Process app events
        while let Ok(event) = app_rx.try_recv() {
            handle_app_event(&mut state, event)?;
        }

        // Small delay to prevent busy waiting
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum AppEvent {
    KeyPress(char),
    KeyDown(winit::event::VirtualKeyCode),
    MouseClick(f64, f64, MouseButton),
}

#[allow(dead_code)]
fn handle_app_event(state: &mut AppState, event: AppEvent) -> Result<()> {
    match event {
        AppEvent::KeyPress(ch) => {
            // Handle character input
            state.command_line.push(ch);
        }
        AppEvent::KeyDown(keycode) => {
            use winit::event::VirtualKeyCode;
            match keycode {
                VirtualKeyCode::Return => {
                    // Execute command
                    if !state.command_line.is_empty() {
                        // Process command
                        state.command_history.push(state.command_line.clone());
                        state.command_line.clear();
                    }
                }
                VirtualKeyCode::Escape => {
                    // Clear command line
                    state.command_line.clear();
                }
                VirtualKeyCode::Up => {
                    // Navigate up in file list
                    if state.active_panel().selected_index > 0 {
                        state.active_panel_mut().selected_index -= 1;
                    }
                }
                VirtualKeyCode::Down => {
                    // Navigate down in file list
                    let entries_len = state.active_panel().entries.len();
                    if state.active_panel().selected_index < entries_len.saturating_sub(1) {
                        state.active_panel_mut().selected_index += 1;
                    }
                }
                _ => {}
            }
        }
        AppEvent::MouseClick(_x, _y, _button) => {
            // Handle mouse clicks
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn render_windowed_ui(renderer: &mut TerminalRenderer, state: &AppState) -> Result<()> {
    // Create simple text representation of the UI
    let mut lines = Vec::new();

    // Title bar
    lines.push(format!("═══ Cortex File Manager v{} ═══", env!("CARGO_PKG_VERSION")));
    lines.push(String::new());

    // Current directory
    lines.push(format!("Directory: {}", state.left_panel.current_dir.display()));
    lines.push(String::new());

    // File list
    lines.push("Files:".to_string());
    lines.push("─".repeat(40));

    let visible_entries = state.left_panel.get_visible_entries();
    for (i, entry) in visible_entries.iter().enumerate() {
        let prefix = if i == state.left_panel.selected_index {
            "► "
        } else {
            "  "
        };

        let type_indicator = match entry.file_type {
            FileType::Directory => "/",
            FileType::File => "",
            FileType::Symlink => "@",
            _ => "",
        };

        lines.push(format!("{}{}{}", prefix, entry.name, type_indicator));
    }

    // Command line
    lines.push(String::new());
    lines.push("─".repeat(40));
    lines.push(format!("Command: {}_", state.command_line));

    // Status
    if let Some(ref msg) = state.status_message {
        lines.push(String::new());
        lines.push(format!("Status: {}", msg));
    }

    // Render to window
    renderer.render_terminal_content(&lines);
    renderer.present()?;

    Ok(())
}

struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    events: EventHandler,
    dialog: Option<Dialog>,
    pending_editor: Option<EditorDialog>,
    pending_config_dialog: Option<ConfigDialog>,

    operation_manager: OperationManager,
    operation_rx: Option<mpsc::UnboundedReceiver<cortex_core::OperationProgress>>,
    search_rx: Option<mpsc::UnboundedReceiver<cortex_core::SearchProgress>>,
    refresh_needed: bool,
    file_change_rx: Option<mpsc::UnboundedReceiver<()>>,
    command_output_rx: Option<mpsc::Receiver<String>>,
    file_event_rx: Option<mpsc::UnboundedReceiver<cortex_core::FileMonitorEvent>>,
    notification_manager: NotificationManager,
    mouse_handler: MouseHandler,
    context_menu: Option<ContextMenu>,
    _mouse_regions: MouseRegionManager,
    suggestions_dismissed: bool,  // Track if user dismissed suggestions with Esc
    ai_response_rx: Option<mpsc::UnboundedReceiver<(String, bool)>>,  // (response, is_error)
    ai_response_tx: mpsc::UnboundedSender<(String, bool)>,  // Channel sender for AI responses
    config_reload_rx: std::sync::mpsc::Receiver <()>
}

impl App {
    async fn new(initial_path: Option<PathBuf>) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut state = AppState::new()?;

        // Set theme from config
        let theme_mode = state.config_manager.get().general.theme.as_str().into();
        state.theme_manager.set_theme(theme_mode);

        // Set initial terminal background color based on theme
        let theme = state.theme_manager.get_current_theme();
        Self::set_terminal_background(&mut terminal, theme)?;
        terminal.clear()?;

        if let Some(path) = initial_path {
            if path.is_dir() {
                state.left_panel.current_dir = path.clone();
                state.right_panel.current_dir = path;
            }
        }

        Self::refresh_panel_with_cache(&mut state.left_panel, &state.directory_cache)?;
        Self::refresh_panel_with_cache(&mut state.right_panel, &state.directory_cache)?;

        // Apply initial configuration
        Self::apply_configuration(&mut state, &mut self.terminal)?;

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
        .await {
            eprintln!("Warning: Failed to initialize file monitor: {}", e);
        }

        // Start cache refresher - disabled by default to prevent resource usage
        // Can be enabled with /monitor command
        // let cache_refresher = Arc::new(CacheRefresher::new(state.directory_cache.clone()));
        // cache_refresher.start().await;
        // state.cache_refresher = Some(cache_refresher.clone());

        let events = EventHandler::new(Duration::from_millis(100));
        let (ai_response_tx, ai_response_rx) = mpsc::unbounded_channel();

        // Create channel for config reload notifications
        let (config_reload_tx, config_reload_rx) = std::sync::mpsc::channel();
        if let Err(e) = state.config_manager.watch_for_changes(config_reload_tx) {
            eprintln!("Warning: Failed to start config watcher: {}", e);
        }

        Ok(Self {
            state,
            terminal,
            events,
            dialog: None,
            pending_editor: None,
            pending_config_dialog: None,

            operation_manager: OperationManager::new(),
            operation_rx: None,
            search_rx: None,
            refresh_needed: false,
            file_change_rx: Some(file_change_rx),
            command_output_rx: None,
            file_event_rx: Some(file_event_rx),
            notification_manager: NotificationManager::new(),
            mouse_handler: MouseHandler::new(),
            context_menu: None,
            _mouse_regions: MouseRegionManager::new(),
            suggestions_dismissed: false,  // Track if user dismissed suggestions with Esc
            ai_response_rx: Some(ai_response_rx),
            ai_response_tx,
            config_reload_rx,
        })
    }



    async fn run(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|frame| {
                UI::draw(frame, &self.state);
                if let Some(ref mut dialog) = self.dialog {
                    let theme = self.state.theme_manager.get_current_theme();
                    cortex_tui::dialogs::render_dialog(frame, dialog, theme);
                }
                // Render notifications on top
                self.notification_manager.render(frame);
            })?;

            if let Some(rx) = &mut self.operation_rx {
                if let Ok(progress) = rx.try_recv() {
                    self.handle_operation_progress(progress);
                }
            }

            // Check for AI responses
            if let Some(rx) = &mut self.ai_response_rx {
                if let Ok((response, _is_error)) = rx.try_recv() {
                    if let Some(Dialog::AIChat(dialog)) = &mut self.dialog {
                        dialog.add_assistant_message(response);
                    }
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

            // Check for command output
            let mut command_outputs = Vec::new();
            let mut channel_closed = false;

            if let Some(rx) = &mut self.command_output_rx {
                // Try to receive all available messages
                loop {
                    match rx.try_recv() {
                        Ok(output) => {
                            command_outputs.push(output);
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                            // No more messages available right now
                            break;
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            // Channel is closed, command has finished
                            channel_closed = true;
                            break;
                        }
                    }
                }
            }

            // Add all received outputs to state
            for output in command_outputs {
                self.state.add_command_output(output);
            }

            // Handle channel closure
            if channel_closed {
                self.state.set_command_running(false);
                self.command_output_rx = None;
            }

            // Check for config reloads
            if self.config_reload_rx.try_recv().is_ok() {
                Self::apply_configuration(&mut self.state, &mut self.terminal)?;
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
                self.state.set_status_message("Configuration reloaded");
            }

            match self.events.next().await? {
                Event::Key(key_event) => {
                    // Debug logging for F10 key detection
                    if matches!(key_event.code, KeyCode::F(10)) {
                        eprintln!("DEBUG: F10 key event received! Modifiers: {:?}", key_event.modifiers);
                    }

                    if self.context_menu.is_some() {
                        self.handle_context_menu_input(key_event).await?;
                    } else if matches!(self.dialog, Some(Dialog::Suggestions(_))) {
                        // Suggestions dialog is special - it doesn't block normal input
                        // Let handle_input process everything
                        if !self.handle_input(key_event).await? {
                            break;
                        }
                    } else if self.dialog.is_some() {
                        // Other dialogs block input
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
        if let Some(ref monitor) = self.state.file_monitor {
            if let Err(e) = monitor.stop().await {
                log::warn!("Failed to stop file monitor: {}", e);
            }
        }

        // Reset terminal colors before exit
        execute!(
            self.terminal.backend_mut(),
            ResetColor,
            Clear(ClearType::All)
        )?;

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        // Force exit all spawned tasks by terminating the process
        // This ensures no background tasks keep running
        // Use exit code 130 to signal intentional exit to dev.sh
        std::process::exit(130);
    }

    async fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Special handling for suggestions dialog
        if matches!(self.dialog, Some(Dialog::Suggestions(_))) {
            match key.code {
                // Only intercept Up/Down for navigation within suggestions
                KeyCode::Up => {
                    if let Some(Dialog::Suggestions(dialog)) = &mut self.dialog {
                        dialog.move_up();
                    }
                    return Ok(true); // Consume this key
                }
                KeyCode::Down => {
                    if let Some(Dialog::Suggestions(dialog)) = &mut self.dialog {
                        dialog.move_down();
                    }
                    return Ok(true); // Consume this key
                }
                // Enter accepts the selected suggestion
                KeyCode::Enter => {
                    if let Some(Dialog::Suggestions(dialog)) = &self.dialog {
                        if let Some(suggestion) = dialog.get_selected_suggestion() {
                            // Replace command line with full cd command
                            self.state.command_line = format!("cd {}", suggestion);
                            self.state.command_cursor = self.state.command_line.len();
                        }
                    }
                    // Close dialog and let Enter be processed normally to execute command
                    self.dialog = None;
                    // Don't return - let it fall through to normal Enter handling
                }
                // Esc closes suggestions without clearing command line
                KeyCode::Esc => {
                    self.dialog = None;
                    self.suggestions_dismissed = true;  // Mark that user dismissed suggestions
                    // Don't clear suggestions - just close the dialog
                    // This prevents the dialog from immediately reopening
                    return Ok(true); // Consume this key - don't process further
                }
                // F10 quits the application even when suggestions dialog is open
                KeyCode::F(10) => {
                    return Ok(false);
                }
                // All other keys (including Tab) fall through to normal processing
                _ => {
                    // Don't close dialog - let typing continue with suggestions updating
                }
            }
        }

        // Then check for special keys that work globally
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

            // History navigation with Up/Down when command line has text (suggestions dialog handles these keys separately above)
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
                // Tab toggles panels (suggestions dialog handles Tab separately above)
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
                            .map(|ext|
                                matches!(ext.to_lowercase().as_str(), "zip" | "tar" | "gz" | "7z")
                            )
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
                self.suggestions_dismissed = false;  // Reset flag when executing command

                // Check for special / commands
                if let Some(cmd) = command.strip_prefix("/") {
                    if !self.handle_special_command(cmd).await? {
                        return Ok(false);
                    }
                } else {
                    // Add to history
                    self.state.command_history.push(command.clone());

                    // Check for cd command
                    if let Some(path_str) = command.strip_prefix("cd ") {
                        let path = path_str.trim();
                        if let Some(new_dir) = CommandProcessor::parse_cd_path(
                            path,
                            &self.state.active_panel().current_dir,
                        ) {
                            // Update file monitoring if enabled
                            if self.state.is_file_monitoring_active() {
                                let active = self.state.active_panel;
                                self.state.update_file_monitoring(active, &new_dir).await?;
                            }

                            let panel = self.state.active_panel_mut();
                            panel.current_dir = new_dir.clone();
                            panel.selected_index = 0;
                            panel.view_offset = 0;
                            Self::refresh_panel(panel)?;

                            self.state.set_status_message(format!(
                                "Changed directory to: {}",
                                new_dir.display()
                            ));
                        } else {
                            self.state.set_status_message(format!(
                                "cd: cannot access '{}': No such directory",
                                path
                            ));
                        }
                    } else {
                        // Execute external command with streaming output
                        self.execute_streaming_command(command.clone()).await?;
                    }
                }

                self.state.command_line.clear();
                self.state.command_cursor = 0;
                self.state.command_history_index = None;
                self.state.command_suggestions.clear();
                self.state.selected_suggestion = None;
            }

            // Function keys
            (KeyCode::F(1), _) => {
                // F1 - Help
                self.dialog = Some(Dialog::Input(
                    InputDialog::new("Help", "F3=View F4=Edit F5=Copy F6=Move F7=MkDir F8=Delete F9=Config. Quit: F10, Ctrl+Q, Ctrl+Shift+Q, Alt+F4. Press Esc to close.").with_value("")
                ));
            }
            (KeyCode::F(3), _) => {
                // F3 - View file
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.file_type == FileType::File {
                        match FileViewer::new(&entry.path) {
                            Ok(mut viewer) => {
                                // Load content with reasonable line limit for terminal height
                                let terminal_height = self.terminal.size().unwrap_or_default().height as usize;
                                let max_lines = terminal_height.saturating_sub(10); // Reserve space for UI
                                if let Err(e) = viewer.load_content(max_lines.max(50)) {
                                    self.state.set_status_message(&format!("Failed to load file content: {}", e));
                                } else {
                                    self.dialog = Some(Dialog::Viewer(ViewerDialog::new(viewer)));
                                }
                            }
                            Err(e) => {
                                self.state.set_status_message(&format!("Failed to open file: {}", e));
                            }
                        }
                    }
                }
            }
            (KeyCode::F(4), KeyModifiers::ALT) => {
                return Ok(false);
            }
            (KeyCode::F(4), KeyModifiers::NONE) => {
                // F4 - Edit file
                if let Some(entry) = self.state.active_panel().current_entry() {
                    if entry.file_type == FileType::File {
                        match TextEditor::new(&entry.path) {
                            Ok(editor) => {
                                self.dialog = Some(Dialog::Editor(EditorDialog::new(editor)));
                            }
                            Err(e) => {
                                self.state.set_status_message(&format!("Failed to open file: {}", e));
                            }
                        }
                    }
                }
            }
            (KeyCode::F(5), _) => {
                // F5 - Copy files with progress bar
                if let Some(operation) = OperationManager::prepare_copy(&self.state).await {
                    if self.state.config_manager.get().general.confirm_operations {
                        self.state.pending_operation = Some(operation.clone());
                        self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                    } else {
                        self.execute_operation(operation).await?;
                    }
                }
            }
            (KeyCode::F(6), _) => {
                // F6 - Move files with progress bar
                if let Some(operation) = OperationManager::prepare_move(&self.state).await {
                    if self.state.config_manager.get().general.confirm_operations {
                        self.state.pending_operation = Some(operation.clone());
                        self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                    } else {
                        self.execute_operation(operation).await?;
                    }
                }
            }
            (KeyCode::F(7), _) => {
                // F7 - Create directory
                self.dialog = Some(Dialog::Input(
                    InputDialog::new("Create Directory", "Enter directory name:")
                ));
            }
            (KeyCode::F(8), _) => {
                // F8 - Delete files
                if let Some(operation) = OperationManager::prepare_delete(&self.state).await {
                    if self.state.config_manager.get().general.confirm_delete {
                        self.state.pending_operation = Some(operation.clone());
                        self.dialog = Some(OperationManager::create_confirm_dialog(&operation));
                    } else {
                        self.execute_operation(operation).await?;
                    }
                }
            }
            (KeyCode::F(9), _) => {
                // F9 - Configuration
                let config = self.state.config_manager.get();
                self.dialog = Some(Dialog::Config(ConfigDialog::new(
                    config,
                    &self.state.theme_manager,
                )));
            }
            (KeyCode::F(10), _) => {
                return Ok(false);
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
            (KeyCode::Char('q'), mods) if mods.contains(KeyModifiers::CONTROL) && mods.contains(KeyModifiers::SHIFT) => {
                return Ok(false);
            }
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                if self.state.command_line.is_empty() {
                    if let Some(entry) = self.state.active_panel().current_entry() {
                        if entry.name != ".." {
                            self.dialog = Some(Dialog::Input(InputDialog::new("Rename", "Enter new name:").with_value(&entry.name)));
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
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                // Open API key configuration
                self.state.set_status_message("Opening API key configuration...");
                self.dialog = Some(Dialog::APIKey(cortex_tui::APIKeyDialog::new()));
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
                // Advanced search
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
                // Clear command line and suggestions
                self.state.command_line.clear();
                self.state.command_cursor = 0;
                self.state.command_history_index = None;
                self.state.command_suggestions.clear();
                self.state.selected_suggestion = None;
                self.suggestions_dismissed = false;  // Reset the flag when clearing command line
            }
            (KeyCode::Backspace, _) => {
                if self.state.command_cursor > 0 {
                    self.state.command_cursor -= 1;
                    self.state.command_line.remove(self.state.command_cursor);

                    // Reset dismissed flag when user modifies the command
                    if self.suggestions_dismissed {
                        self.suggestions_dismissed = false;
                    }

                    self.state.update_command_suggestions();

                    // Update suggestions dialog only if not dismissed
                    if !self.state.command_suggestions.is_empty() && !self.suggestions_dismissed {
                        // Only create dialog if it doesn't exist
                        if !matches!(self.dialog, Some(cortex_tui::dialogs::Dialog::Suggestions(_))) {
                            self.dialog = Some(cortex_tui::dialogs::Dialog::Suggestions(
                                cortex_tui::dialogs::SuggestionsDialog::new(
                                    self.state.command_suggestions.clone()
                                )
                            ));
                        } else if let Some(cortex_tui::dialogs::Dialog::Suggestions(dialog)) = &mut self.dialog {
                            // Update existing dialog
                            dialog.suggestions = self.state.command_suggestions.clone();
                            if dialog.selected_index >= dialog.suggestions.len() && !dialog.suggestions.is_empty() {
                                dialog.selected_index = dialog.suggestions.len() - 1;
                            }
                        }
                    } else {
                        if matches!(self.dialog, Some(cortex_tui::dialogs::Dialog::Suggestions(_))) {
                            self.dialog = None;
                        }
                    }
                }
            }
            (KeyCode::Delete, _) => {
                if self.state.command_cursor < self.state.command_line.len() {
                    self.state.command_line.remove(self.state.command_cursor);

                    // Reset dismissed flag when user modifies the command
                    if self.suggestions_dismissed {
                        self.suggestions_dismissed = false;
                    }

                    self.state.update_command_suggestions();

                    // Update suggestions dialog only if not dismissed
                    if !self.state.command_suggestions.is_empty() && !self.suggestions_dismissed {
                        // Only create dialog if it doesn't exist
                        if !matches!(self.dialog, Some(cortex_tui::dialogs::Dialog::Suggestions(_))) {
                            self.dialog = Some(cortex_tui::dialogs::Dialog::Suggestions(
                                cortex_tui::dialogs::SuggestionsDialog::new(
                                    self.state.command_suggestions.clone()
                                )
                            ));
                        } else if let Some(cortex_tui::dialogs::Dialog::Suggestions(dialog)) = &mut self.dialog {
                            // Update existing dialog
                            dialog.suggestions = self.state.command_suggestions.clone();
                            if dialog.selected_index >= dialog.suggestions.len() && !dialog.suggestions.is_empty() {
                                dialog.selected_index = dialog.suggestions.len() - 1;
                            }
                        }
                    } else {
                        if matches!(self.dialog, Some(cortex_tui::dialogs::Dialog::Suggestions(_))) {
                            self.dialog = None;
                        }
                    }
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
            (KeyCode::Char('o'), KeyModifiers::SHIFT) | (KeyCode::Char('O'), _) 
                if self.state.command_line.is_empty() =>
            {
                // Toggle command output visibility with 'O' key
                self.state.toggle_command_output();
            }
            (KeyCode::Char(c), _) => {
                self.state.command_line.insert(self.state.command_cursor, c);
                self.state.command_cursor += 1;

                // Reset dismissed flag when user modifies the command
                if self.suggestions_dismissed {
                    self.suggestions_dismissed = false;
                }

                self.state.update_command_suggestions();

                // Only show suggestions dialog if:
                // 1. We have suggestions
                // 2. User hasn't dismissed them with Esc
                // 3. Dialog is not already showing (or is not a suggestions dialog)
                if !self.state.command_suggestions.is_empty() && !self.suggestions_dismissed {
                    // Only create dialog if it doesn't exist or isn't a suggestions dialog
                    if !matches!(self.dialog, Some(cortex_tui::dialogs::Dialog::Suggestions(_))) {
                        self.dialog = Some(cortex_tui::dialogs::Dialog::Suggestions(
                            cortex_tui::dialogs::SuggestionsDialog::new(
                                self.state.command_suggestions.clone()
                            )
                        ));
                    } else if let Some(cortex_tui::dialogs::Dialog::Suggestions(dialog)) = &mut self.dialog {
                        // Update existing dialog with new suggestions
                        dialog.suggestions = self.state.command_suggestions.clone();
                        if dialog.selected_index >= dialog.suggestions.len() && !dialog.suggestions.is_empty() {
                            dialog.selected_index = dialog.suggestions.len() - 1;
                        }
                    }
                } else {
                    // Close suggestions dialog if no more suggestions or dismissed
                    if matches!(self.dialog, Some(cortex_tui::dialogs::Dialog::Suggestions(_))) {
                        self.dialog = None;
                    }
                }
            }

            _ => {}
        }

        Ok(true)
    }

    async fn handle_special_command(&mut self, command: &str) -> Result<bool> {
        // Debug log to track which command is being executed
        log::debug!("Executing special command: {}", command);

        match command {
            "exit" | "quit" | "q" => {
                return Ok(false); // signal to exit
            }
            "restart" => {
                let exe = std::env::current_exe()?;
                let args: Vec<String> = std::env::args().skip(1).collect();
                std::process::Command::new(exe).args(&args).spawn()?;
                return Ok(false); // also exit after restart
            }
            _ => {}
        }
        Ok(true) // continue running
    }

    async fn handle_dialog_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Debug logging for F10 key
        if matches!(key.code, KeyCode::F(10)) {
            eprintln!("DEBUG: F10 key detected in handle_dialog_input!");
        }

        // Global F10 key handling - quit application even when dialogs are open
        if key.code == KeyCode::F(10) {
            // Stop file monitoring if active
            if let Some(ref monitor) = self.state.file_monitor {
                if let Err(e) = monitor.stop().await {
                    log::warn!("Failed to stop file monitor: {}", e);
                }
            }

            // Reset terminal colors before exit
            execute!(
                self.terminal.backend_mut(),
                ResetColor,
                Clear(ClearType::All)
            )?;

            // Cleanup terminal
            disable_raw_mode()?;
            execute!(
                self.terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;

            // Force exit all spawned tasks by terminating the process
            // This ensures no background tasks keep running
            // Use exit code 130 to signal intentional exit to dev.sh
            std::process::exit(130);
        }

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
                KeyCode::Esc => {
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
                        (KeyCode::Esc, _) => {
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
                        KeyCode::Esc | KeyCode::Char('q') => {
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
                            let command = cmd.strip_prefix('/').unwrap_or(&cmd);
                            if !self.handle_special_command(command).await? {
                                return Ok(false);
                            }
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
                        if key.code == KeyCode::Esc {
                            // Cancel search
                            self.search_rx = None; // Drop the receiver to stop processing
                            dialog.state = SearchState::Results; // Show results collected so far
                            self.state.set_status_message(format!(
                                "Search cancelled: {} results found",
                                dialog.results.len()
                            ));
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
                                        plugin_name,
                                        status
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
                        KeyCode::Up => {
                            if dialog.is_boolean_field() {
                                dialog.toggle_current_boolean_value();
                            } else if dialog.current_tab == ConfigTab::Themes {
                                dialog.cycle_theme_backward();
                            } else if dialog.current_tab == ConfigTab::AI && dialog.selected_index == 0 {
                                dialog.cycle_provider_backward();
                            }
                        }
                        KeyCode::Down => {
                            if dialog.is_boolean_field() {
                                dialog.toggle_current_boolean_value();
                            } else if dialog.current_tab == ConfigTab::Themes {
                                dialog.cycle_theme_forward();
                            } else if dialog.current_tab == ConfigTab::AI && dialog.selected_index == 0 {
                                dialog.cycle_provider_forward();
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
                        // For text fields
                        KeyCode::Char(c) => {
                            if !dialog.is_boolean_field() && !dialog.is_dropdown_field() {
                                dialog.insert_char(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if !dialog.is_boolean_field() && !dialog.is_dropdown_field() {
                                dialog.delete_char();
                            }
                        }
                        KeyCode::Left => {
                            if !dialog.is_boolean_field() && !dialog.is_dropdown_field() {
                                dialog.move_cursor_left();
                            }
                        }
                        KeyCode::Right => {
                            if !dialog.is_boolean_field() && !dialog.is_dropdown_field() {
                                dialog.move_cursor_right();
                            }
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
                                Self::apply_configuration(&mut self.state, &mut self.terminal)?;
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
                                dialog.dirty = false;
                                self.dialog = None;
                            }
                        }
                        KeyCode::Esc => {
                            if dialog.dirty {
                                if let Some(Dialog::Config(config_dialog)) = self.dialog.take() {
                                    self.pending_config_dialog = Some(config_dialog);
                                }
                                self.dialog = Some(Dialog::SaveConfirm(SaveConfirmDialog::new(
                                    "You have unsaved changes. Save before closing?".to_string(),
                                )));
                            } else {
                                self.dialog = None;
                            }
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
                            self.dialog = None;
                        } else if let Some(mut config_dialog) = self.pending_config_dialog.take() {
                            // Save configuration
                            config_dialog.dirty = false;
                            let config = config_dialog.config.clone();
                            if let Err(e) = self.state.config_manager.update(|c| *c = config) {
                                self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                                    "Failed to save configuration: {}",
                                    e
                                ))));
                            } else {
                                Self::apply_configuration(&mut self.state, &mut self.terminal)?;
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
                                self.dialog = None;
                            }
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        // Quick select Don't Save - close without saving
                        self.dialog = None;
                        self.pending_editor = None;
                        self.pending_config_dialog = None;
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => {
                        // Cancel - return to editor or config
                        if let Some(editor_dialog) = self.pending_editor.take() {
                            self.dialog = Some(Dialog::Editor(editor_dialog));
                        } else if let Some(config_dialog) = self.pending_config_dialog.take() {
                            self.dialog = Some(Dialog::Config(config_dialog));
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
                                    self.dialog = None;
                                } else if let Some(config_dialog) = self.pending_config_dialog.take() {
                                    // Save configuration
                                    let config = config_dialog.config.clone();
                                    if let Err(e) = self.state.config_manager.update(|c| *c = config) {
                                        self.dialog = Some(Dialog::Error(ErrorDialog::new(format!(
                                            "Failed to save configuration: {}",
                                            e
                                        ))));
                                    } else {
                                        Self::apply_configuration(&mut self.state, &mut self.terminal)?;
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
                                        self.dialog = None;
                                    }
                                }
                            }
                            SaveChoice::DontSave => {
                                // Close without saving
                                self.dialog = None;
                                self.pending_editor = None;
                                self.pending_config_dialog = None;
                            }
                            SaveChoice::Cancel => {
                                // Return to editor or config
                                if let Some(editor_dialog) = self.pending_editor.take() {
                                    self.dialog = Some(Dialog::Editor(editor_dialog));
                                } else if let Some(config_dialog) = self.pending_config_dialog.take() {
                                    self.dialog = Some(Dialog::Config(config_dialog));
                                } else {
                                    self.dialog = None;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Some(Dialog::ThemeSelection(dialog)) => match key.code {
                KeyCode::Up => {
                    dialog.move_up();
                }
                KeyCode::Down => {
                    dialog.move_down();
                }
                KeyCode::Enter => {
                    let selected_theme = dialog.get_selected_theme();
                    self.state.theme_manager.set_theme(selected_theme);
                    self.state.set_status_message(format!(
                        "Theme changed to: {}",
                        dialog.themes[dialog.selected_index].1
                    ));
                    self.dialog = None;
                }
                KeyCode::Esc => {
                    self.dialog = None;
                }
                _ => {}
            },
            Some(Dialog::Suggestions(_)) => {
                // Suggestions dialog is handled at the beginning of the function
            },
            Some(Dialog::AIChat(dialog)) => {
                match key.code {
                    KeyCode::Char(c) => {
                        dialog.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        dialog.delete_char();
                    }
                    KeyCode::Left if key.modifiers.is_empty() => {
                        dialog.move_cursor_left();
                    }
                    KeyCode::Right if key.modifiers.is_empty() => {
                        dialog.move_cursor_right();
                    }
                    KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        dialog.scroll_up();
                    }
                    KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        dialog.scroll_down();
                    }
                    KeyCode::PageUp => {
                        // Scroll up by multiple lines
                        for _ in 0..10 {
                            dialog.scroll_up();
                        }
                    }
                    KeyCode::PageDown => {
                        // Scroll down by multiple lines
                        for _ in 0..10 {
                            dialog.scroll_down();
                        }
                    }
                    KeyCode::Home if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Scroll to top
                        dialog.scroll_position = 0;
                    }
                    KeyCode::End if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Scroll to bottom
                        dialog.scroll_to_bottom();
                    }
                    KeyCode::Enter => {
                        if let Some(message) = dialog.submit_message() {
                            // Debug: Log the message
                            log::info!("AI Chat: User message: {}", message);

                            // Get AI response
                            if let Some(ai_manager) = self.state.ai_manager.clone() {
                                log::info!("AI Chat: Manager is available");

                                // Create context from current state
                                let context = cortex_core::ai::AIContext::new(
                                    self.state.active_panel().current_dir.clone(),
                                )
                                .with_files(
                                    self.state.active_panel().marked_files.clone(),
                                );

                                // Spawn async task to get AI response
                                let tx = self.ai_response_tx.clone();
                                tokio::spawn(async move {
                                    match ai_manager.complete(&message, context).await {
                                        Ok(response) => {
                                            log::info!("AI Chat: Got response: {}", response.content);
                                            let _ = tx.send((response.content, false));
                                        }
                                        Err(e) => {
                                            log::error!("AI Chat: Error: {}", e);
                                            let _ = tx.send((format!("Error: {}", e), true));
                                        }
                                    }
                                });
                            } else {
                                log::warn!("AI Chat: Manager not initialized");
                                dialog.add_assistant_message(
                                    "AI is not configured. Please check your settings.".to_string(),
                                );
                            }
                        }
                    }
                    KeyCode::Up => {
                        dialog.scroll_up();
                    }
                    KeyCode::Down => {
                        dialog.scroll_down();
                    }
                    KeyCode::Esc => {
                        self.dialog = None;
                    }
                    _ => {}
                }
            }
            Some(Dialog::APIKey(dialog)) => {
                match key.code {
                    KeyCode::Tab if !dialog.input_mode => {
                        dialog.toggle_dropdown();
                    }
                    KeyCode::Enter => {
                        if dialog.input_mode {
                            // Save the API key
                            if !dialog.api_key.is_empty() {
                                let _ = self.state.config_manager.set_api_key(
                                    dialog.selected_provider.as_str(),
                                    dialog.api_key.clone(),
                                );
                                self.state.set_status_message(format!(
                                    "API key saved for {}",
                                    dialog.selected_provider.as_str()
                                ));
                                
                                // Reinitialize AI manager with new config
                                let config = self.state.config_manager.get();
                                self.state.ai_manager = Some(std::sync::Arc::new(cortex_core::ai::AIManager::new(config.ai)));
                                self.dialog = None;
                            }
                        } else if dialog.provider_dropdown_open {
                            dialog.toggle_dropdown();
                        } else {
                            dialog.toggle_input_mode();
                        }
                    }
                    KeyCode::Up if dialog.provider_dropdown_open => {
                        dialog.prev_provider();
                    }
                    KeyCode::Down if dialog.provider_dropdown_open => {
                        dialog.next_provider();
                    }
                    KeyCode::Char(c) if dialog.input_mode => {
                        dialog.add_char(c);
                    }
                    KeyCode::Backspace if dialog.input_mode => {
                        dialog.delete_char();
                    }
                    KeyCode::Left if dialog.input_mode => {
                        dialog.move_cursor_left();
                    }
                    KeyCode::Right if dialog.input_mode => {
                        dialog.move_cursor_right();
                    }
                    KeyCode::Tab if dialog.input_mode => {
                        dialog.toggle_show_key();
                    }
                    KeyCode::Esc => {
                        if dialog.input_mode {
                            dialog.toggle_input_mode();
                        } else if dialog.provider_dropdown_open {
                            dialog.toggle_dropdown();
                        } else {
                            self.dialog = None;
                        }
                    }
                    _ => {}
                }
            }
            None => {}
        }

        Ok(true)
    }
}