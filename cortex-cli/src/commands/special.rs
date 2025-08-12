// Special command handling (commands starting with /)
use anyhow::Result;
use cortex_tui::{
    CommandPaletteDialog, ConfigDialog, ConnectionDialog,
    Dialog, FilterDialog, PluginDialog, SearchDialog
};
use crate::app::App;
use crate::connections::{connect_sftp, connect_ftp};

/// Handle special commands (those starting with /)
pub async fn handle_special_command(app: &mut App, command: &str) -> Result<()> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd = parts.get(0).map(|s| s.to_lowercase());
    
    match cmd.as_deref() {
        Some("help") | Some("h") | Some("?") => {
            show_help_menu(app);
        }
        Some("filter") | Some("f") => {
            app.dialog = Some(Dialog::Filter(FilterDialog::new()));
        }
        Some("search") | Some("s") => {
            app.dialog = Some(Dialog::Search(SearchDialog::new()));
        }
        Some("config") | Some("cfg") => {
            let config = app.state.config_manager.get().clone();
            app.dialog = Some(Dialog::Config(ConfigDialog::new(config)));
        }
        Some("plugins") | Some("p") => {
            let plugins = app.state.plugin_manager.get_plugin_info();
            app.dialog = Some(Dialog::Plugin(PluginDialog::new(plugins)));
        }
        Some("commands") | Some("cmd") => {
            app.dialog = Some(Dialog::CommandPalette(CommandPaletteDialog::new()));
        }
        Some("connect") | Some("c") => {
            handle_connect_command(app, &parts[1..]).await?;
        }
        Some("reload") | Some("r") => {
            app.mark_refresh_needed();
            app.state.set_status_message("Panels refreshed");
        }
        Some("theme") | Some("t") => {
            if let Some(theme_name) = parts.get(1) {
                set_theme(app, theme_name)?;
            } else {
                app.state.set_status_message("Usage: /theme <name>");
            }
        }
        Some("quit") | Some("q") | Some("exit") => {
            return Ok(()); // This will be handled by the caller as exit
        }
        _ => {
            app.state.set_status_message(format!("Unknown command: /{}", command));
        }
    }
    
    Ok(())
}

fn show_help_menu(app: &mut App) {
    let help_text = vec![
        "Special Commands:",
        "/help, /h, /?     - Show this help",
        "/filter, /f       - Filter files in current panel",
        "/search, /s       - Search for files",
        "/config, /cfg     - Open configuration",
        "/plugins, /p      - Manage plugins",
        "/commands, /cmd   - Command palette",
        "/connect, /c      - Connect to remote (sftp/ftp)",
        "/reload, /r       - Reload panels",
        "/theme, /t        - Change theme",
        "/quit, /q         - Exit application",
        "",
        "Press any key to continue...",
    ];
    
    app.state.set_status_message(help_text.join("\n"));
}

async fn handle_connect_command(app: &mut App, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        app.dialog = Some(Dialog::Connection(ConnectionDialog::new()));
        return Ok(());
    }
    
    match args[0].to_lowercase().as_str() {
        "sftp" | "ssh" => {
            if args.len() >= 4 {
                // Direct connection: /connect sftp host port username
                let host = args[1].to_string();
                let port = args[2].parse().unwrap_or(22);
                let username = args[3].to_string();
                connect_sftp(app, host, port, username).await?;
            } else {
                app.dialog = Some(Dialog::Connection(ConnectionDialog::new()));
            }
        }
        "ftp" => {
            if args.len() >= 4 {
                let host = args[1].to_string();
                let port = args[2].parse().unwrap_or(21);
                let username = args[3].to_string();
                connect_ftp(app, host, port, username).await?;
            } else {
                app.dialog = Some(Dialog::Connection(ConnectionDialog::new()));
            }
        }
        _ => {
            app.state.set_status_message("Usage: /connect [sftp|ftp] [host port username]");
        }
    }
    
    Ok(())
}

fn set_theme(app: &mut App, theme_name: &str) -> Result<()> {
    use cortex_core::ThemeMode;
    
    let theme = match theme_name.to_lowercase().as_str() {
        "dark" => ThemeMode::Dark,
        "light" => ThemeMode::Light,
        "gruvbox" => ThemeMode::Gruvbox,
        "nord" => ThemeMode::Nord,
        "modern" => ThemeMode::Modern,
        _ => {
            app.state.set_status_message(format!("Unknown theme: {}", theme_name));
            return Ok(());
        }
    };
    
    app.state.theme_manager.set_theme(theme);
    
    // Update terminal background
    use crate::app::lifecycle::set_terminal_background;
    set_terminal_background(&mut app.terminal, &app.state)?;
    
    app.state.set_status_message(format!("Theme changed to: {}", theme_name));
    Ok(())
}