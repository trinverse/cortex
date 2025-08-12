// Keyboard input handling
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use cortex_core::{FileType, VfsPath};
use cortex_tui::Dialog;

use crate::app::App;
use crate::commands::process_command;

/// Main keyboard input handler
pub async fn handle_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Handle suggestions dialog first
    if handle_suggestions_dialog(app, key).await? {
        return Ok(false);
    }

    // Handle navigation keys
    if handle_navigation(app, key).await? {
        return Ok(false);
    }

    // Handle command line input
    if handle_command_line(app, key).await? {
        return Ok(false);
    }

    // Handle function keys
    if handle_function_keys(app, key).await? {
        return Ok(false);
    }

    // Handle file operations
    if handle_file_operations(app, key).await? {
        return Ok(false);
    }

    // Handle special keys
    handle_special_keys(app, key).await
}

/// Handle suggestions dialog input
async fn handle_suggestions_dialog(app: &mut App, key: KeyEvent) -> Result<bool> {
    if !matches!(app.dialog, Some(Dialog::Suggestions(_))) {
        return Ok(false);
    }

    match key.code {
        KeyCode::Up => {
            if let Some(Dialog::Suggestions(dialog)) = &mut app.dialog {
                dialog.move_up();
            }
            Ok(true)
        }
        KeyCode::Down => {
            if let Some(Dialog::Suggestions(dialog)) = &mut app.dialog {
                dialog.move_down();
            }
            Ok(true)
        }
        KeyCode::Enter => {
            if let Some(Dialog::Suggestions(dialog)) = &app.dialog {
                if let Some(suggestion) = dialog.get_selected_suggestion() {
                    app.state.command_line = format!("cd {}", suggestion);
                    app.state.command_cursor = app.state.command_line.len();
                }
            }
            app.dialog = None;
            Ok(false) // Let Enter be processed normally
        }
        KeyCode::Esc => {
            app.dialog = None;
            app.suggestions_dismissed = true;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Handle navigation keys
async fn handle_navigation(app: &mut App, key: KeyEvent) -> Result<bool> {
    if !app.state.command_line.is_empty() {
        return Ok(false);
    }

    match (key.code, key.modifiers) {
        (KeyCode::Up, modifiers) if modifiers.is_empty() => {
            let panel = app.state.active_panel_mut();
            panel.move_selection_up();
            let height = app.terminal.size()?.height as usize - 5;
            panel.update_view_offset(height);
            Ok(true)
        }
        (KeyCode::Down, modifiers) if modifiers.is_empty() => {
            let panel = app.state.active_panel_mut();
            panel.move_selection_down();
            let height = app.terminal.size()?.height as usize - 5;
            panel.update_view_offset(height);
            Ok(true)
        }
        (KeyCode::Left, modifiers) if modifiers.is_empty() => {
            navigate_parent(app).await?;
            Ok(true)
        }
        (KeyCode::Right, modifiers) if modifiers.is_empty() => {
            navigate_into(app).await?;
            Ok(true)
        }
        (KeyCode::PageUp, _) => {
            let page_size = app.terminal.size()?.height as usize - 10;
            app.state.active_panel_mut().move_selection_page_up(page_size);
            Ok(true)
        }
        (KeyCode::PageDown, _) => {
            let page_size = app.terminal.size()?.height as usize - 10;
            app.state.active_panel_mut().move_selection_page_down(page_size);
            Ok(true)
        }
        (KeyCode::Home, _) => {
            app.state.active_panel_mut().move_selection_home();
            Ok(true)
        }
        (KeyCode::End, _) => {
            app.state.active_panel_mut().move_selection_end();
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Handle command line input
async fn handle_command_line(app: &mut App, key: KeyEvent) -> Result<bool> {
    if app.state.command_line.is_empty() && !matches!(key.code, KeyCode::Char(_)) {
        return Ok(false);
    }

    match key.code {
        KeyCode::Char(c) => {
            app.state.command_line.insert(app.state.command_cursor, c);
            app.state.command_cursor += 1;
            app.state.update_command_suggestions();
            update_suggestions_dialog(app);
            Ok(true)
        }
        KeyCode::Backspace => {
            if app.state.command_cursor > 0 {
                app.state.command_cursor -= 1;
                app.state.command_line.remove(app.state.command_cursor);
                app.state.update_command_suggestions();
                update_suggestions_dialog(app);
            }
            Ok(true)
        }
        KeyCode::Delete => {
            if app.state.command_cursor < app.state.command_line.len() {
                app.state.command_line.remove(app.state.command_cursor);
                app.state.update_command_suggestions();
                update_suggestions_dialog(app);
            }
            Ok(true)
        }
        KeyCode::Left if !app.state.command_line.is_empty() => {
            if app.state.command_cursor > 0 {
                app.state.command_cursor -= 1;
            }
            Ok(true)
        }
        KeyCode::Right if !app.state.command_line.is_empty() => {
            if app.state.command_cursor < app.state.command_line.len() {
                app.state.command_cursor += 1;
            }
            Ok(true)
        }
        KeyCode::Up if !app.state.command_line.is_empty() => {
            navigate_history_up(app);
            Ok(true)
        }
        KeyCode::Down if !app.state.command_line.is_empty() => {
            navigate_history_down(app);
            Ok(true)
        }
        KeyCode::Enter if !app.state.command_line.is_empty() => {
            execute_command(app).await?;
            Ok(true)
        }
        KeyCode::Esc => {
            handle_esc_key(app);
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Handle function keys
async fn handle_function_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    use cortex_tui::{HelpDialog, ThemeSelectionDialog};

    match key.code {
        KeyCode::F(1) => {
            app.dialog = Some(Dialog::Help(HelpDialog::new()));
            Ok(true)
        }
        KeyCode::F(2) => {
            app.state.command_line = "/".to_string();
            app.state.command_cursor = 1;
            Ok(true)
        }
        KeyCode::F(3) => {
            view_file(app).await?;
            Ok(true)
        }
        KeyCode::F(4) => {
            edit_file(app).await?;
            Ok(true)
        }
        KeyCode::F(5) => {
            copy_files(app).await?;
            Ok(true)
        }
        KeyCode::F(6) => {
            move_files(app).await?;
            Ok(true)
        }
        KeyCode::F(7) => {
            create_directory(app).await?;
            Ok(true)
        }
        KeyCode::F(8) => {
            delete_files(app).await?;
            Ok(true)
        }
        KeyCode::F(9) => {
            let current_theme = app.state.theme_manager.get_current_theme().mode;
            app.dialog = Some(Dialog::ThemeSelection(ThemeSelectionDialog::new(current_theme)));
            Ok(true)
        }
        KeyCode::F(10) => Ok(true), // Exit
        _ => Ok(false),
    }
}

/// Handle file operations
async fn handle_file_operations(app: &mut App, key: KeyEvent) -> Result<bool> {
    match (key.code, key.modifiers) {
        (KeyCode::Char(' '), _) if app.state.command_line.is_empty() => {
            app.state.active_panel_mut().toggle_mark_current();
            let panel = app.state.active_panel_mut();
            panel.move_selection_down();
            let height = app.terminal.size()?.height as usize - 5;
            panel.update_view_offset(height);
            Ok(true)
        }
        (KeyCode::Char('a'), KeyModifiers::CONTROL) if app.state.command_line.is_empty() => {
            mark_all_files(app);
            Ok(true)
        }
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            refresh_panels(app).await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Handle special keys
async fn handle_special_keys(app: &mut App, key: KeyEvent) -> Result<bool> {
    match (key.code, key.modifiers) {
        (KeyCode::Tab, _) => {
            app.state.toggle_panel();
            Ok(true)
        }
        (KeyCode::Enter, _) if app.state.command_line.is_empty() => {
            enter_directory_or_archive(app).await?;
            Ok(true)
        }
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(true), // Exit
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            if app.state.command_running {
                app.state.set_command_running(false);
                app.state.add_command_output("[CANCELLED] Command execution cancelled".to_string());
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

// Helper functions

fn handle_esc_key(app: &mut App) {
    if app.state.command_output_visible {
        app.state.command_output_visible = false;
        app.state.set_status_message("Command output panel closed");
    } else {
        app.state.command_line.clear();
        app.state.command_cursor = 0;
        app.state.command_history_index = None;
        app.state.command_suggestions.clear();
        app.state.selected_suggestion = None;
        app.suggestions_dismissed = false;
    }
}

fn update_suggestions_dialog(app: &mut App) {
    use cortex_tui::dialogs::SuggestionsDialog;
    
    if !app.suggestions_dismissed && !app.state.command_suggestions.is_empty() {
        let suggestions = app.state.command_suggestions.clone();
        app.dialog = Some(Dialog::Suggestions(SuggestionsDialog::new(suggestions)));
    } else if app.state.command_suggestions.is_empty() {
        if matches!(app.dialog, Some(Dialog::Suggestions(_))) {
            app.dialog = None;
        }
    }
}

fn navigate_history_up(app: &mut App) {
    if !app.state.command_history.is_empty() {
        let new_index = match app.state.command_history_index {
            None => app.state.command_history.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
        };
        app.state.command_history_index = Some(new_index);
        app.state.command_line = app.state.command_history[new_index].clone();
        app.state.command_cursor = app.state.command_line.len();
    }
}

fn navigate_history_down(app: &mut App) {
    if let Some(index) = app.state.command_history_index {
        if index < app.state.command_history.len() - 1 {
            app.state.command_history_index = Some(index + 1);
            app.state.command_line = app.state.command_history[index + 1].clone();
        } else {
            app.state.command_history_index = None;
            app.state.command_line.clear();
        }
        app.state.command_cursor = app.state.command_line.len();
    }
}

async fn navigate_parent(app: &mut App) -> Result<()> {
    let current_dir = app.state.active_panel().current_dir.clone();
    if let Some(parent) = current_dir.parent() {
        navigate_to_directory(app, parent.to_path_buf()).await?;
    }
    Ok(())
}

async fn navigate_into(app: &mut App) -> Result<()> {
    let current_entry = app.state.active_panel().current_entry().cloned();
    let current_dir = app.state.active_panel().current_dir.clone();

    if let Some(entry) = current_entry {
        if entry.file_type == FileType::Directory {
            let new_dir = if entry.name == ".." {
                current_dir.parent().map(|p| p.to_path_buf())
            } else {
                Some(entry.path.clone())
            };

            if let Some(dir) = new_dir {
                navigate_to_directory(app, dir).await?;
            }
        }
    }
    Ok(())
}

async fn navigate_to_directory(app: &mut App, new_path: std::path::PathBuf) -> Result<()> {
    use crate::app::lifecycle::refresh_panel_with_cache;
    
    let cache = app.state.directory_cache.clone();
    {
        let panel = app.state.active_panel_mut();
        panel.current_dir = new_path.clone();
        panel.selected_index = 0;
        panel.view_offset = 0;
        
        refresh_panel_with_cache(panel, &cache)?;
    }
    
    if let Some(ref monitor) = app.state.file_monitor {
        monitor.watch_directory(&new_path, false).await?;
    }
    
    Ok(())
}

pub async fn enter_directory_or_archive(app: &mut App) -> Result<()> {
    use crate::app::lifecycle::refresh_panel_with_cache;
    
    if app.state.active_panel().is_using_vfs() {
        let panel = app.state.active_panel();
        if let Some(entry) = panel.current_vfs_entry().cloned() {
            if entry.name == ".." {
                app.state.navigate_back_from_vfs()?;
                let cache = app.state.directory_cache.clone();
                let panel = app.state.active_panel_mut();
                refresh_panel_with_cache(panel, &cache)?;
            } else {
                app.state.set_status_message("VFS navigation not available in this build");
            }
        }
    } else {
        let panel = app.state.active_panel();
        if let Some(entry) = panel.current_entry().cloned() {
            if entry.file_type == FileType::Directory {
                let new_dir = if entry.name == ".." {
                    panel.current_dir.parent().map(|p| p.to_path_buf())
                } else {
                    Some(entry.path.clone())
                };

                if let Some(dir) = new_dir {
                    let cache = app.state.directory_cache.clone();
                    let panel = app.state.active_panel_mut();
                    panel.current_dir = dir;
                    panel.selected_index = 0;
                    panel.view_offset = 0;
                    refresh_panel_with_cache(panel, &cache)?;
                }
            } else if is_archive(&entry.path) {
                let vfs_path = VfsPath::Archive {
                    archive_path: entry.path.clone(),
                    internal_path: String::new(),
                };
                app.state.navigate_into_vfs(vfs_path)?;
            }
        }
    }
    Ok(())
}

fn is_archive(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "zip" | "tar" | "gz" | "7z"))
        .unwrap_or(false)
}

async fn execute_command(app: &mut App) -> Result<()> {
    let command = app.state.command_line.clone();
    
    if !command.is_empty() {
        app.state.command_history.push(command.clone());
        app.state.command_history_index = None;
    }
    
    app.state.command_line.clear();
    app.state.command_cursor = 0;
    app.state.command_suggestions.clear();
    app.state.selected_suggestion = None;
    app.suggestions_dismissed = false;
    
    if let Some(Dialog::Suggestions(_)) = app.dialog {
        app.dialog = None;
    }
    
    process_command(app, &command).await?;
    Ok(())
}

async fn view_file(app: &mut App) -> Result<()> {
    use cortex_tui::{ViewerDialog, FileViewer};
    
    if let Some(entry) = app.state.active_panel().current_entry() {
        if entry.file_type == FileType::File {
            let viewer = FileViewer::new(&entry.path)?;
            app.dialog = Some(Dialog::Viewer(ViewerDialog::new(viewer)));
        }
    }
    Ok(())
}

async fn edit_file(app: &mut App) -> Result<()> {
    use cortex_tui::{EditorDialog, TextEditor};
    
    if let Some(entry) = app.state.active_panel().current_entry() {
        if entry.file_type == FileType::File {
            let editor = TextEditor::new(&entry.path)?;
            app.pending_editor = Some(EditorDialog::new(editor));
            app.dialog = Some(Dialog::Editor(app.pending_editor.as_ref().unwrap().clone()));
        }
    }
    Ok(())
}

async fn copy_files(app: &mut App) -> Result<()> {
    use cortex_core::FileOperation;
    
    let sources = app.get_selected_files();
    if !sources.is_empty() {
        let dest = app.state.inactive_panel().current_dir.clone();
        let operation = FileOperation::Copy {
            sources: sources.clone(),
            destination: dest.clone(),
        };
        app.state.pending_operation = Some(operation);
        app.state.set_status_message(format!(
            "Copy {} file(s) to {}? (Y/N)",
            sources.len(),
            dest.display()
        ));
    }
    Ok(())
}

async fn move_files(app: &mut App) -> Result<()> {
    use cortex_core::FileOperation;
    
    let sources = app.get_selected_files();
    if !sources.is_empty() {
        let dest = app.state.inactive_panel().current_dir.clone();
        let operation = FileOperation::Move {
            sources: sources.clone(),
            destination: dest.clone(),
        };
        app.state.pending_operation = Some(operation);
        app.state.set_status_message(format!(
            "Move {} file(s) to {}? (Y/N)",
            sources.len(),
            dest.display()
        ));
    }
    Ok(())
}

async fn create_directory(app: &mut App) -> Result<()> {
    use cortex_tui::InputDialog;
    
    app.dialog = Some(Dialog::Input(InputDialog::new(
        "Create Directory",
        "Enter directory name:",
    )));
    Ok(())
}

async fn delete_files(app: &mut App) -> Result<()> {
    use cortex_core::FileOperation;
    
    let targets = app.get_selected_files();
    if !targets.is_empty() {
        let operation = FileOperation::Delete {
            targets: targets.clone(),
        };
        app.state.pending_operation = Some(operation);
        app.state.set_status_message(format!(
            "Delete {} file(s)? (Y/N)",
            targets.len()
        ));
    }
    Ok(())
}

fn mark_all_files(app: &mut App) {
    let panel = app.state.active_panel_mut();
    let entries = panel.get_visible_entries().clone();
    
    for entry in entries {
        if entry.name != ".." && !panel.is_marked(&entry.path) {
            panel.marked_files.push(entry.path);
        }
    }
}

async fn refresh_panels(app: &mut App) -> Result<()> {
    use crate::app::lifecycle::refresh_panel_with_cache;
    
    let cache = app.state.directory_cache.clone();
    refresh_panel_with_cache(&mut app.state.left_panel, &cache)?;
    refresh_panel_with_cache(&mut app.state.right_panel, &cache)?;
    app.state.set_status_message("Panels refreshed");
    Ok(())
}