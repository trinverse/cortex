// File system event handling
use cortex_core::FileMonitorEvent;
use crate::app::App;

/// Handle file system events
pub fn handle_file_event(app: &mut App, event: FileMonitorEvent) {
    use cortex_tui::NotificationType;
    
    // Check if the event affects either panel
    let left_dir = &app.state.left_panel.current_dir;
    let right_dir = &app.state.right_panel.current_dir;
    
    let (path, message) = match &event {
        FileMonitorEvent::Created(path) => (path, format!("File created: {}", path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown"))),
        FileMonitorEvent::Modified(path) => (path, format!("File modified: {}", path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown"))),
        FileMonitorEvent::Deleted(path) => (path, format!("File deleted: {}", path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown"))),
        FileMonitorEvent::Renamed { from: _, to } => (to, format!("File renamed to: {}", to.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown"))),
    };
    
    let affects_left = path.starts_with(left_dir) || 
                       path.parent() == Some(left_dir);
    let affects_right = path.starts_with(right_dir) || 
                        path.parent() == Some(right_dir);
    
    if affects_left || affects_right {
        // Mark for refresh
        app.mark_refresh_needed();
        
        // Show notification
        app.notification_manager.add_notification(
            message,
            "File System Event",
            NotificationType::Info,
        );
    }
}