// Context menu handling
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use cortex_tui::ContextMenuAction;
use cortex_core::FileOperation;
use crate::app::App;

/// Handle context menu input
pub async fn handle_context_menu_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.context_menu = None;
        }
        KeyCode::Enter => {
            if let Some(menu) = &app.context_menu {
                if let Some(action) = menu.get_selected_action() {
                    execute_context_menu_action(app, action).await?;
                }
            }
            app.context_menu = None;
        }
        KeyCode::Up => {
            if let Some(menu) = &mut app.context_menu {
                if menu.selected_index > 0 {
                    menu.selected_index -= 1;
                }
            }
        }
        KeyCode::Down => {
            if let Some(menu) = &mut app.context_menu {
                if menu.selected_index < menu.items.len() - 1 {
                    menu.selected_index += 1;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn execute_context_menu_action(app: &mut App, action: ContextMenuAction) -> Result<()> {
    use cortex_tui::{InputDialog, Dialog, ViewerDialog, FileViewer, EditorDialog, TextEditor};
    
    match action {
        ContextMenuAction::ViewFile => {
            if let Some(entry) = app.state.active_panel().current_entry() {
                if entry.file_type == cortex_core::FileType::File {
                    let viewer = FileViewer::new(&entry.path)?;
                    app.dialog = Some(Dialog::Viewer(ViewerDialog::new(viewer)));
                }
            }
        }
        ContextMenuAction::EditFile => {
            if let Some(entry) = app.state.active_panel().current_entry() {
                if entry.file_type == cortex_core::FileType::File {
                    let editor = TextEditor::new(&entry.path)?;
                    app.pending_editor = Some(EditorDialog::new(editor));
                    app.dialog = Some(Dialog::Editor(app.pending_editor.as_ref().unwrap().clone()));
                }
            }
        }
        ContextMenuAction::Copy => {
            let sources = app.get_selected_files();
            if !sources.is_empty() {
                let dest = app.state.inactive_panel().current_dir.clone();
                app.state.pending_operation = Some(FileOperation::Copy {
                    sources: sources.clone(),
                    destination: dest.clone(),
                });
                app.state.set_status_message(format!(
                    "Copy {} file(s) to {}? (Y/N)",
                    sources.len(),
                    dest.display()
                ));
            }
        }
        ContextMenuAction::Cut => {
            let sources = app.get_selected_files();
            if !sources.is_empty() {
                let dest = app.state.inactive_panel().current_dir.clone();
                app.state.pending_operation = Some(FileOperation::Move {
                    sources: sources.clone(),
                    destination: dest.clone(),
                });
                app.state.set_status_message(format!(
                    "Move {} file(s) to {}? (Y/N)",
                    sources.len(),
                    dest.display()
                ));
            }
        }
        ContextMenuAction::Delete => {
            let targets = app.get_selected_files();
            if !targets.is_empty() {
                app.state.pending_operation = Some(FileOperation::Delete {
                    targets: targets.clone(),
                });
                app.state.set_status_message(format!(
                    "Delete {} file(s)? (Y/N)",
                    targets.len()
                ));
            }
        }
        ContextMenuAction::Rename => {
            if let Some(_entry) = app.state.active_panel().current_entry() {
                app.dialog = Some(Dialog::Input(InputDialog::new(
                    "Rename",
                    "Enter new name:",
                )));
            }
        }
        ContextMenuAction::Properties => {
            if let Some(entry) = app.state.active_panel().current_entry() {
                let info = format!(
                    "Name: {}\nSize: {}\nModified: {:?}\nPermissions: {}",
                    entry.name,
                    entry.size_display,
                    entry.modified,
                    entry.permissions
                );
                app.state.set_status_message(info);
            }
        }
        _ => {}
    }
    
    Ok(())
}