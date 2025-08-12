// Progress handling for operations
use cortex_core::{OperationProgress, SearchProgress};
use cortex_tui::{ProgressDialog, Dialog};
use crate::app::App;

/// Handle operation progress updates
pub fn handle_operation_progress(app: &mut App, progress: OperationProgress) {
    match progress {
        OperationProgress::Started { operation } => {
            app.dialog = Some(Dialog::Progress(ProgressDialog::new(
                "File Operation",
                &operation,
            )));
            app.state.set_status_message(format!("Started: {}", operation));
        }
        OperationProgress::Progress {
            current,
            total,
            message,
        } => {
            if let Some(Dialog::Progress(dialog)) = &mut app.dialog {
                dialog.update(current, total, &message);
            }
        }
        OperationProgress::Completed { operation } => {
            app.dialog = None;
            app.mark_refresh_needed();
            app.state.set_status_message(format!("Completed: {}", operation));
        }
        OperationProgress::Failed { operation, error } => {
            app.dialog = None;
            app.state.set_status_message(format!("Failed {}: {}", operation, error));
        }
    }
}

/// Handle search progress updates
pub fn handle_search_progress(app: &mut App, progress: SearchProgress) {
    match progress {
        SearchProgress::Started { .. } => {
            app.state.set_status_message("Search started...");
        }
        SearchProgress::Searching { .. } => {
            app.state.set_status_message("Searching...");
        }
        SearchProgress::Found { result } => {
            app.state.add_command_output(format!("Found: {}", result.path.display()));
        }
        SearchProgress::Completed { total_found, .. } => {
            app.state.set_status_message(format!("Search complete: {} matches found", total_found));
        }
        SearchProgress::Error { error, .. } => {
            app.state.set_status_message(format!("Search error: {}", error));
        }
    }
}