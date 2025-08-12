// Application lifecycle management
use anyhow::Result;
use cortex_core::{AppState, DirectoryCache, FileSystem};
use cortex_tui::{EventHandler, MouseHandler, MouseRegionManager, NotificationManager, dialogs::render_dialog};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{Color as CrosstermColor, ResetColor, SetBackgroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, sync::Arc};
use tokio::sync::mpsc;

use super::state::App;
use super::config::apply_configuration;
use crate::operations::OperationManager;

/// Initialize the application
pub async fn initialize_app(initial_path: Option<PathBuf>) -> Result<App> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Initialize app state
    let mut state = AppState::new()?;
    
    // Set initial directory
    if let Some(path) = initial_path {
        if path.is_dir() {
            state.left_panel.current_dir = path.clone();
            state.right_panel.current_dir = path;
        }
    }

    // Apply configuration
    apply_configuration(&mut state);

    // Load plugins
    load_plugins(&mut state).await?;

    // Initialize file monitor
    let file_event_rx = init_file_monitor_with_callback(&mut state).await?;

    // Set terminal background based on theme
    set_terminal_background(&mut terminal, &state)?;

    // Refresh both panels
    refresh_panels(&mut state)?;

    Ok(App {
        state,
        terminal,
        events: EventHandler::new(std::time::Duration::from_millis(100)),
        dialog: None,
        pending_editor: None,
        _operation_manager: OperationManager::new(),
        operation_rx: None,
        search_rx: None,
        refresh_needed: false,
        _file_change_rx: None,
        command_output_rx: None,
        file_event_rx,
        notification_manager: NotificationManager::new(),
        _mouse_handler: MouseHandler::new(),
        context_menu: None,
        _mouse_regions: MouseRegionManager::new(),
        suggestions_dismissed: false,
    })
}

/// Main application loop
pub async fn run_app(app: &mut App) -> Result<()> {
    use cortex_tui::{Event, UI};
    
    loop {
        // Draw UI
        app.terminal.draw(|frame| {
            UI::draw(frame, &app.state);
            
            // Draw dialog if present
            if let Some(ref mut dialog) = app.dialog {
                render_dialog(frame, dialog, &app.state.theme_manager.get_current_theme());
            }
            
            // Context menu and notifications are drawn by UI components themselves
        })?;

        // Handle events  
        if let Ok(event) = app.events.next().await {
            match event {
                Event::Key(key) => {
                    if app.context_menu.is_some() {
                        use crate::handlers::context_menu::handle_context_menu_input;
                        handle_context_menu_input(app, key).await?;
                    } else if app.dialog.is_some() {
                        use crate::handlers::dialog::handle_dialog_input;
                        if handle_dialog_input(app, key).await? {
                            return Ok(());
                        }
                    } else {
                        use crate::handlers::keyboard::handle_input;
                        if handle_input(app, key).await? {
                            return Ok(());
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    use crate::handlers::mouse::handle_mouse_event;
                    use cortex_tui::MouseAction;
                    
                    // Convert mouse event to action
                    let action = match mouse.kind {
                        crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                            Some(MouseAction::Click(cortex_tui::Position { x: mouse.column, y: mouse.row }))
                        }
                        crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Right) => {
                            Some(MouseAction::RightClick(cortex_tui::Position { x: mouse.column, y: mouse.row }))
                        }
                        crossterm::event::MouseEventKind::ScrollUp => {
                            Some(MouseAction::ScrollUp(cortex_tui::Position { x: mouse.column, y: mouse.row }))
                        }
                        crossterm::event::MouseEventKind::ScrollDown => {
                            Some(MouseAction::ScrollDown(cortex_tui::Position { x: mouse.column, y: mouse.row }))
                        }
                        _ => None,
                    };
                    
                    if let Some(action) = action {
                        handle_mouse_event(app, action).await?;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal will be redrawn on next iteration
                }
                Event::Tick => {
                    // Handle periodic updates
                }
            }
        }

        // Handle operation progress
        if let Some(ref mut rx) = app.operation_rx {
            if let Ok(progress) = rx.try_recv() {
                use crate::handlers::progress::handle_operation_progress;
                handle_operation_progress(app, progress);
            }
        }

        // Handle search progress
        if let Some(ref mut rx) = app.search_rx {
            if let Ok(progress) = rx.try_recv() {
                use crate::handlers::progress::handle_search_progress;
                handle_search_progress(app, progress);
            }
        }

        // Handle file events
        if let Some(ref mut rx) = app.file_event_rx {
            if let Ok(event) = rx.try_recv() {
                use crate::handlers::file_event::handle_file_event;
                handle_file_event(app, event);
            }
        }

        // Handle command output
        if let Some(ref mut rx) = app.command_output_rx {
            while let Ok(line) = rx.try_recv() {
                app.state.add_command_output(line);
            }
        }

        // Refresh panels if needed
        if app.refresh_needed {
            refresh_panels(&mut app.state)?;
            app.refresh_needed = false;
        }
    }
}

/// Clean up and exit application
pub async fn cleanup_app(app: &mut App) -> Result<()> {
    // Stop file monitor if active
    if let Some(monitor) = app.state.file_monitor.take() {
        monitor.stop().await?;
    }

    // Clean up terminal
    disable_raw_mode()?;
    execute!(
        app.terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        ResetColor
    )?;
    app.terminal.show_cursor()?;

    Ok(())
}

/// Load plugins
async fn load_plugins(state: &mut AppState) -> Result<()> {
    use cortex_core::LuaPlugin;
    use cortex_plugins::Plugin;
    
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("cortex");
    let plugin_dir = config_dir.join("plugins");
    
    if plugin_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                    match LuaPlugin::new(path.clone()) {
                        Ok(plugin) => {
                            log::info!("Loaded plugin: {}", plugin.info().name);
                            let _ = state.plugin_manager.load_plugin(Box::new(plugin)).await;
                        }
                        Err(e) => {
                            log::error!("Failed to load plugin {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Initialize file monitor with callback
async fn init_file_monitor_with_callback(
    state: &mut AppState,
) -> Result<Option<mpsc::UnboundedReceiver<cortex_core::FileMonitorEvent>>> {
    use cortex_core::FileMonitorManager;
    use std::sync::Arc;

    if state.auto_reload_enabled {
        let (tx, rx) = mpsc::unbounded_channel();
        let monitor_manager = Arc::new(FileMonitorManager::new().await?);
        monitor_manager.start().await?;

        let tx_clone = tx.clone();
        let callback: cortex_core::EventCallback = Arc::new(move |notification| {
            let _ = tx_clone.send(notification.event.clone());
        });

        monitor_manager.register_change_callback(callback).await;

        monitor_manager
            .watch_directory(&state.left_panel.current_dir, false)
            .await?;
        monitor_manager
            .watch_directory(&state.right_panel.current_dir, false)
            .await?;

        state.file_monitor = Some(monitor_manager);
        Ok(Some(rx))
    } else {
        Ok(None)
    }
}

/// Refresh both panels
fn refresh_panels(state: &mut AppState) -> Result<()> {
    refresh_panel_with_cache(&mut state.left_panel, &state.directory_cache)?;
    refresh_panel_with_cache(&mut state.right_panel, &state.directory_cache)?;
    Ok(())
}

/// Refresh a single panel with cache
pub fn refresh_panel_with_cache(
    panel: &mut cortex_core::PanelState,
    cache: &Arc<DirectoryCache>,
) -> Result<()> {
    if !panel.is_using_vfs() {
        // Try to get from cache first
        if let Some(cached_entries) = cache.get(&panel.current_dir) {
            panel.entries = cached_entries;
        } else {
            // Load from filesystem
            panel.entries = FileSystem::list_directory(&panel.current_dir, panel.show_hidden)?;
            // Store in cache
            cache.put(&panel.current_dir, panel.entries.clone())?;
        }

        // Update git info
        panel.git_info = cortex_core::git::get_git_info(&panel.current_dir);

        // Apply filter if present
        if let Some(ref filter) = panel.filter.clone() {
            panel.apply_filter(filter);
        }

        // Sort entries
        panel.sort_entries();
    }
    Ok(())
}

/// Set terminal background color based on theme
pub fn set_terminal_background(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &AppState,
) -> Result<()> {
    let theme = state.theme_manager.get_current_theme();
    let bg_color = match theme.background {
        ratatui::style::Color::Rgb(r, g, b) => CrosstermColor::Rgb { r, g, b },
        ratatui::style::Color::Black => CrosstermColor::Black,
        ratatui::style::Color::White => CrosstermColor::White,
        ratatui::style::Color::Gray => CrosstermColor::Grey,
        ratatui::style::Color::DarkGray => CrosstermColor::DarkGrey,
        _ => CrosstermColor::Reset,
    };

    execute!(terminal.backend_mut(), SetBackgroundColor(bg_color), Clear(ClearType::All))?;
    Ok(())
}