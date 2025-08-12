// Mouse event handling
use anyhow::Result;
use cortex_tui::{MouseAction, Position, ContextMenu};
use crate::app::App;

/// Handle mouse events
pub async fn handle_mouse_event(app: &mut App, action: MouseAction) -> Result<()> {
    match action {
        MouseAction::Click(pos) => handle_mouse_click(app, pos).await?,
        MouseAction::DoubleClick(pos) => handle_mouse_double_click(app, pos).await?,
        MouseAction::RightClick(pos) => handle_mouse_right_click(app, pos).await?,
        MouseAction::ScrollUp(pos) => handle_mouse_scroll_up(app, pos).await?,
        MouseAction::ScrollDown(pos) => handle_mouse_scroll_down(app, pos).await?,
        _ => {}
    }
    Ok(())
}

async fn handle_mouse_click(app: &mut App, pos: Position) -> Result<()> {
    // Determine which panel was clicked
    let terminal_width = app.terminal.size()?.width;
    let panel_width = terminal_width / 2;
    
    if pos.x < panel_width {
        // Left panel clicked
        if app.state.active_panel != cortex_core::ActivePanel::Left {
            app.state.toggle_panel();
        }
        
        // Calculate which item was clicked
        if pos.y > 0 && pos.y < app.terminal.size()?.height - 4 {
            let clicked_index = (pos.y as usize - 1) + app.state.left_panel.view_offset;
            if clicked_index < app.state.left_panel.entries.len() {
                app.state.left_panel.selected_index = clicked_index;
            }
        }
    } else {
        // Right panel clicked
        if app.state.active_panel != cortex_core::ActivePanel::Right {
            app.state.toggle_panel();
        }
        
        if pos.y > 0 && pos.y < app.terminal.size()?.height - 4 {
            let clicked_index = (pos.y as usize - 1) + app.state.right_panel.view_offset;
            if clicked_index < app.state.right_panel.entries.len() {
                app.state.right_panel.selected_index = clicked_index;
            }
        }
    }
    
    Ok(())
}

async fn handle_mouse_double_click(app: &mut App, pos: Position) -> Result<()> {
    // First handle single click to select
    handle_mouse_click(app, pos).await?;
    
    // Then navigate into directory or open file
    use crate::handlers::keyboard::enter_directory_or_archive;
    enter_directory_or_archive(app).await?;
    
    Ok(())
}

async fn handle_mouse_right_click(app: &mut App, pos: Position) -> Result<()> {
    // First handle single click to select
    handle_mouse_click(app, pos).await?;
    
    // Show context menu
    let has_selection = !app.state.active_panel().marked_files.is_empty();
    app.context_menu = Some(ContextMenu::file_menu(pos, has_selection));
    Ok(())
}

async fn handle_mouse_scroll_up(app: &mut App, pos: Position) -> Result<()> {
    let terminal_width = app.terminal.size()?.width;
    let panel_width = terminal_width / 2;
    
    let panel = if pos.x < panel_width {
        &mut app.state.left_panel
    } else {
        &mut app.state.right_panel
    };
    
    if panel.view_offset > 0 {
        panel.view_offset = panel.view_offset.saturating_sub(3);
    }
    
    Ok(())
}

async fn handle_mouse_scroll_down(app: &mut App, pos: Position) -> Result<()> {
    let terminal_width = app.terminal.size()?.width;
    let panel_width = terminal_width / 2;
    
    let panel = if pos.x < panel_width {
        &mut app.state.left_panel
    } else {
        &mut app.state.right_panel
    };
    
    let max_offset = panel.entries.len().saturating_sub(10);
    if panel.view_offset < max_offset {
        panel.view_offset = (panel.view_offset + 3).min(max_offset);
    }
    
    Ok(())
}