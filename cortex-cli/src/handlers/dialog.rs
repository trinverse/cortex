// Dialog input handling
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use cortex_tui::Dialog;
use crate::app::App;

/// Handle dialog input
pub async fn handle_dialog_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    // First check what type of dialog we have
    let dialog_type = app.dialog.as_ref().map(|d| match d {
        Dialog::Input(_) => "input",
        Dialog::Filter(_) => "filter",
        Dialog::Search(_) => "search",
        Dialog::Config(_) => "config",
        Dialog::Help(_) => "help",
        Dialog::ThemeSelection(_) => "theme",
        Dialog::Connection(_) => "connection",
        _ => "other",
    });
    
    let should_close = match dialog_type {
        Some("input") => handle_simple_dialog(key),
        Some("filter") => handle_simple_dialog(key),
        Some("search") => handle_simple_dialog(key),
        Some("config") => handle_simple_dialog(key),
        Some("help") => matches!(key.code, KeyCode::Esc | KeyCode::Enter),
        Some("theme") => handle_theme_dialog_keys(app, key).await?,
        Some("connection") => handle_simple_dialog(key),
        _ => false,
    };
    
    if should_close {
        app.dialog = None;
    }
    
    Ok(false) // Don't exit app
}

fn handle_simple_dialog(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter | KeyCode::Esc)
}

async fn handle_theme_dialog_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    if let Some(Dialog::ThemeSelection(dialog)) = &mut app.dialog {
        match key.code {
            KeyCode::Enter => {
                let theme = dialog.get_selected_theme();
                app.state.theme_manager.set_theme(theme);
                
                // Update terminal background
                use crate::app::lifecycle::set_terminal_background;
                set_terminal_background(&mut app.terminal, &app.state)?;
                
                app.state.set_status_message(format!("Theme changed to: {:?}", theme));
                Ok(true)
            }
            KeyCode::Esc => Ok(true),
            KeyCode::Up => {
                if dialog.selected_index > 0 {
                    dialog.selected_index -= 1;
                }
                Ok(false)
            }
            KeyCode::Down => {
                if dialog.selected_index < 4 {
                    dialog.selected_index += 1;
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    } else {
        Ok(false)
    }
}

