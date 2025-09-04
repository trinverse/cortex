use anyhow::Result;
use cortex_core::{AppState, Theme, shortcuts::ShortcutManager};
use cortex_tui::{EventHandler, MouseHandler, MouseRegionManager, NotificationManager};
use crossterm::{
    event::{EnableMouseCapture},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, time::Duration};
use tokio::sync::mpsc;

use super::App;
use crate::operations::OperationManager;

impl App {
    /// Create a new App instance with initial setup
    pub async fn new(initial_path: Option<PathBuf>) -> Result<Self> {
        // Initialize terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initialize application state
        let mut state = AppState::new()?;

        // Set theme from config
        let theme_mode = state.config_manager.get().general.theme.as_str().into();
        state.theme_manager.set_theme(theme_mode);

        // Set initial terminal background color based on theme
        let theme = state.theme_manager.get_current_theme();
        set_terminal_background(&mut terminal, theme)?;
        terminal.clear()?;

        // Set initial paths if provided
        if let Some(path) = initial_path {
            if path.is_dir() {
                state.left_panel.current_dir = path.clone();
                state.right_panel.current_dir = path;
            }
        }

        // Create channels for various communications
        let (file_change_tx, file_change_rx) = mpsc::unbounded_channel();
        let (file_event_tx, file_event_rx) = mpsc::unbounded_channel();
        let (ai_response_tx, ai_response_rx) = mpsc::unbounded_channel();
        let (config_reload_tx, config_reload_rx) = std::sync::mpsc::channel();

        // Set up configuration watcher
        if let Err(e) = state.config_manager.watch_for_changes(config_reload_tx) {
            eprintln!("Warning: Failed to start config watcher: {}", e);
        }

        // Create the app instance
        let mut app = Self {
            state,
            terminal,
            events: EventHandler::new(Duration::from_millis(100)),
            shortcut_manager: ShortcutManager::new(),
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
            suggestions_dismissed: false,
            ai_response_rx: Some(ai_response_rx),
            ai_response_tx,
            config_reload_rx,
        };

        // Perform post-initialization setup - refresh both panels with current directory contents
        {
            use cortex_core::FileSystem;
            
            // Get configuration setting for show_hidden
            let config = app.state.config_manager.get();
            let show_hidden = config.general.show_hidden;
            
            // Refresh left panel
            let entries = FileSystem::list_directory(&app.state.left_panel.current_dir, show_hidden)?;
            app.state.left_panel.entries = entries;
            app.state.left_panel.selected_index = app.state.left_panel.selected_index.min(app.state.left_panel.entries.len().saturating_sub(1));
            
            // Refresh right panel
            let entries = FileSystem::list_directory(&app.state.right_panel.current_dir, show_hidden)?;
            app.state.right_panel.entries = entries;
            app.state.right_panel.selected_index = app.state.right_panel.selected_index.min(app.state.right_panel.entries.len().saturating_sub(1));
        }
        
        app.apply_configuration()?;
        
        if let Err(e) = app.load_plugins().await {
            eprintln!("Warning: Failed to load plugins: {}", e);
        }

        if let Err(e) = app.init_file_monitor_with_callback(
            file_change_tx.clone(),
            file_event_tx.clone(),
        ).await {
            eprintln!("Warning: Failed to initialize file monitor: {}", e);
        }

        Ok(app)
    }

    /// Set terminal background color based on theme
    fn set_terminal_background(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, _theme: &Theme) -> Result<()> {
        use crossterm::{style::{SetBackgroundColor, Color}, execute};
        
        // Use a default dark background for now
        // TODO: Extract color information from theme properly
        let bg_color = Color::Black;

        execute!(terminal.backend_mut(), SetBackgroundColor(bg_color))?;
        Ok(())
    }

    /// Load and initialize plugins
    pub async fn load_plugins(&mut self) -> Result<()> {
        // TODO: Implement plugin loading
        // The plugin manager needs to be properly implemented in cortex-plugins
        // For now, just log that plugins would be loaded
        log::info!("Plugin loading not yet implemented");
        Ok(())
    }

    /// Apply current configuration to the application
    pub fn apply_configuration(&mut self) -> Result<()> {
        let config = self.state.config_manager.get();
        
        // Apply theme settings
        let theme_mode = config.general.theme.as_str().into();
        self.state.theme_manager.set_theme(theme_mode);
        
        // Apply custom color configuration to override theme defaults
        self.apply_color_configuration(&config);
        
        // Update terminal background
        let theme = self.state.theme_manager.get_current_theme();
        set_terminal_background(&mut self.terminal, theme)?;
        
        // Apply panel configuration - sync show_hidden setting to panels
        self.state.left_panel.show_hidden = config.general.show_hidden;
        self.state.right_panel.show_hidden = config.general.show_hidden;
        
        // Apply editor and terminal configuration
        self.apply_editor_terminal_configuration(&config);
        
        // Apply auto-reload configuration
        if config.general.auto_reload {
            // Enable auto-reload if configured
            // TODO: Set up file monitoring based on this setting
        }
        
        // Apply plugin configuration
        self.apply_plugin_configuration(&config);
        
        // If show_hidden setting changed, refresh panels to show/hide files accordingly
        self.refresh_needed = true;
        
        Ok(())
    }

    /// Initialize file monitoring with callback
    pub async fn init_file_monitor_with_callback(
        &mut self,
        _file_change_tx: mpsc::UnboundedSender<()>,
        _file_event_tx: mpsc::UnboundedSender<cortex_core::FileMonitorEvent>,
    ) -> Result<()> {
        // TODO: Implement file monitoring setup
        // This would set up the file system watcher to monitor changes
        // and send notifications through the provided channels
        Ok(())
    }

    /// Apply color configuration settings
    fn apply_color_configuration(&mut self, config: &cortex_core::Config) {
        // Apply custom color overrides to the current theme
        use ratatui::style::Color;
        
        // Helper function to parse color strings
        let parse_color = |color_str: &str| -> Option<Color> {
            match color_str.to_lowercase().as_str() {
                "red" => Some(Color::Red),
                "green" => Some(Color::Green),
                "blue" => Some(Color::Blue),
                "yellow" => Some(Color::Yellow),
                "cyan" => Some(Color::Cyan),
                "magenta" => Some(Color::Magenta),
                "white" => Some(Color::White),
                "black" => Some(Color::Black),
                "gray" | "grey" => Some(Color::Gray),
                "dark_gray" | "dark_grey" => Some(Color::DarkGray),
                "light_red" => Some(Color::LightRed),
                "light_green" => Some(Color::LightGreen),
                "light_blue" => Some(Color::LightBlue),
                "light_yellow" => Some(Color::LightYellow),
                "light_cyan" => Some(Color::LightCyan),
                "light_magenta" => Some(Color::LightMagenta),
                _ => None,
            }
        };
        
        // Apply color overrides if they can be parsed
        if let Some(selection_bg) = parse_color(&config.colors.selection_bg) {
            self.state.theme_manager.override_selection_bg(selection_bg);
        }
        if let Some(directory_fg) = parse_color(&config.colors.directory_fg) {
            self.state.theme_manager.override_directory_color(directory_fg);
        }
        if let Some(executable_fg) = parse_color(&config.colors.executable_fg) {
            self.state.theme_manager.override_executable_color(executable_fg);
        }
        if let Some(symlink_fg) = parse_color(&config.colors.symlink_fg) {
            self.state.theme_manager.override_symlink_color(symlink_fg);
        }
    }
    
    /// Apply editor and terminal configuration
    fn apply_editor_terminal_configuration(&mut self, config: &cortex_core::Config) {
        // Store editor and terminal settings for when they're needed
        // The actual usage happens when F4 (edit) or terminal commands are executed
        log::debug!("Editor configured: {}", config.general.editor);
        log::debug!("Terminal configured: {}", config.general.terminal);
        
        // TODO: Set up terminal manager with the configured terminal
        // TODO: Set up editor preferences for F4 key functionality
    }
    
    /// Apply plugin configuration
    fn apply_plugin_configuration(&mut self, config: &cortex_core::Config) {
        if config.plugins.enable_plugins {
            // Enable plugin system
            log::debug!("Plugins enabled, directory: {}", config.general.plugin_directory);
            
            // Apply plugin settings
            for disabled_plugin in &config.plugins.disabled_plugins {
                log::debug!("Plugin disabled: {}", disabled_plugin);
                // TODO: Disable specific plugins in the plugin manager
            }
            
            if config.plugins.auto_reload_plugins {
                log::debug!("Plugin auto-reload enabled");
                // TODO: Set up plugin file watching for auto-reload
            }
            
            if !config.plugins.allow_unsafe_plugins {
                log::debug!("Unsafe plugins disabled");
                // TODO: Configure plugin security settings
            }
        } else {
            log::debug!("Plugins disabled");
            // TODO: Disable the entire plugin system
        }
    }
    
    /// Reload configuration from disk
    pub fn reload_configuration(&mut self) -> Result<()> {
        self.state.config_manager.reload()?;
        self.apply_configuration()?;
        Ok(())
    }
}

/// Helper function for setting terminal background (static version)
fn set_terminal_background(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, theme: &Theme) -> Result<()> {
    App::set_terminal_background(terminal, theme)
}