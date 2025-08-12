use anyhow::Result;
use cortex_core::{
    AppState, DefaultOperationHandler, FileOperation, Operation, OperationHandler,
    OperationProgress,
};
use cortex_tui::{ConfirmDialog, Dialog};
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct OperationManager {
    _handler: DefaultOperationHandler,
}

#[allow(dead_code)]
impl OperationManager {
    pub fn new() -> Self {
        Self {
            _handler: DefaultOperationHandler,
        }
    }

    pub async fn copy_files(&mut self, sources: Vec<PathBuf>, destination: PathBuf) -> Result<()> {
        for source in sources {
            let op = Operation::Copy {
                src: source.clone(),
                dst: destination.clone(),
            };

            // Create a channel for progress updates
            let (tx, mut rx) = mpsc::channel(100);

            // Execute the operation
            self._handler.execute(op, tx).await?;

            // Drain any remaining progress messages
            while rx.try_recv().is_ok() {}
        }

        Ok(())
    }

    pub async fn move_files(&mut self, sources: Vec<PathBuf>, destination: PathBuf) -> Result<()> {
        for source in sources {
            let op = Operation::Move {
                src: source.clone(),
                dst: destination.clone(),
            };

            // Create a channel for progress updates
            let (tx, mut rx) = mpsc::channel(100);

            // Execute the operation
            self._handler.execute(op, tx).await?;

            // Drain any remaining progress messages
            while rx.try_recv().is_ok() {}
        }

        Ok(())
    }

    pub async fn prepare_copy(state: &AppState) -> Option<FileOperation> {
        let source_panel = state.active_panel();
        let dest_panel = state.inactive_panel();

        let sources = if !source_panel.marked_files.is_empty() {
            source_panel.marked_files.clone()
        } else if let Some(entry) = source_panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::Copy {
            sources,
            destination: dest_panel.current_dir.clone(),
        })
    }

    pub async fn prepare_move(state: &AppState) -> Option<FileOperation> {
        let source_panel = state.active_panel();
        let dest_panel = state.inactive_panel();

        let sources = if !source_panel.marked_files.is_empty() {
            source_panel.marked_files.clone()
        } else if let Some(entry) = source_panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::Move {
            sources,
            destination: dest_panel.current_dir.clone(),
        })
    }

    pub async fn prepare_delete(state: &AppState) -> Option<FileOperation> {
        let panel = state.active_panel();

        let targets = if !panel.marked_files.is_empty() {
            panel.marked_files.clone()
        } else if let Some(entry) = panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::Delete { targets })
    }

    pub async fn prepare_delete_to_trash(state: &AppState) -> Option<FileOperation> {
        let panel = state.active_panel();

        let targets = if !panel.marked_files.is_empty() {
            panel.marked_files.clone()
        } else if let Some(entry) = panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::DeleteToTrash { targets })
    }

    pub async fn prepare_copy_to_clipboard(state: &AppState) -> Option<FileOperation> {
        let panel = state.active_panel();

        let paths = if !panel.marked_files.is_empty() {
            panel.marked_files.clone()
        } else if let Some(entry) = panel.current_entry() {
            if entry.name != ".." {
                vec![entry.path.clone()]
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FileOperation::CopyToClipboard { paths })
    }

    pub async fn prepare_paste_from_clipboard(state: &AppState) -> Option<FileOperation> {
        let panel = state.active_panel();
        let destination = panel.current_dir.clone();

        Some(FileOperation::PasteFromClipboard { destination })
    }

    pub async fn execute_operation(
        &mut self,
        operation: FileOperation,
        progress_tx: mpsc::UnboundedSender<OperationProgress>,
    ) -> Result<()> {
        match operation {
            FileOperation::Copy {
                sources,
                destination,
            } => {
                for source in sources {
                    let dest_path = destination.join(source.file_name().unwrap_or_default());
                    let op = Operation::Copy {
                        src: source,
                        dst: dest_path,
                    };

                    let (tx, mut rx) = mpsc::channel(100);
                    let progress_tx_clone = progress_tx.clone();
                    tokio::spawn(async move {
                        while let Some(progress) = rx.recv().await {
                            let _ = progress_tx_clone.send(progress);
                        }
                    });

                    self._handler.execute(op, tx).await?;
                }
            }
            FileOperation::Move {
                sources,
                destination,
            } => {
                for source in sources {
                    let dest_path = destination.join(source.file_name().unwrap_or_default());
                    let op = Operation::Move {
                        src: source,
                        dst: dest_path,
                    };

                    let (tx, mut rx) = mpsc::channel(100);
                    let progress_tx_clone = progress_tx.clone();
                    tokio::spawn(async move {
                        while let Some(progress) = rx.recv().await {
                            let _ = progress_tx_clone.send(progress);
                        }
                    });

                    self._handler.execute(op, tx).await?;
                }
            }
            FileOperation::Delete { targets } => {
                for target in targets {
                    let op = Operation::Delete { path: target };

                    let (tx, mut rx) = mpsc::channel(100);
                    let progress_tx_clone = progress_tx.clone();
                    tokio::spawn(async move {
                        while let Some(progress) = rx.recv().await {
                            let _ = progress_tx_clone.send(progress);
                        }
                    });

                    self._handler.execute(op, tx).await?;
                }
            }
            FileOperation::DeleteToTrash { targets } => {
                for target in targets {
                    let op = Operation::DeleteToTrash { path: target };

                    let (tx, mut rx) = mpsc::channel(100);
                    let progress_tx_clone = progress_tx.clone();
                    tokio::spawn(async move {
                        while let Some(progress) = rx.recv().await {
                            let _ = progress_tx_clone.send(progress);
                        }
                    });

                    self._handler.execute(op, tx).await?;
                }
            }
            FileOperation::RestoreFromTrash { targets } => {
                for target in targets {
                    let op = Operation::RestoreFromTrash { path: target };

                    let (tx, mut rx) = mpsc::channel(100);
                    let progress_tx_clone = progress_tx.clone();
                    tokio::spawn(async move {
                        while let Some(progress) = rx.recv().await {
                            let _ = progress_tx_clone.send(progress);
                        }
                    });

                    self._handler.execute(op, tx).await?;
                }
            }
            FileOperation::CopyToClipboard { paths } => {
                let op = Operation::CopyToClipboard { paths };

                let (tx, mut rx) = mpsc::channel(100);
                let progress_tx_clone = progress_tx.clone();
                tokio::spawn(async move {
                    while let Some(progress) = rx.recv().await {
                        let _ = progress_tx_clone.send(progress);
                    }
                });

                self._handler.execute(op, tx).await?;
            }
            FileOperation::PasteFromClipboard { destination } => {
                let op = Operation::PasteFromClipboard { dst: destination };

                let (tx, mut rx) = mpsc::channel(100);
                let progress_tx_clone = progress_tx.clone();
                tokio::spawn(async move {
                    while let Some(progress) = rx.recv().await {
                        let _ = progress_tx_clone.send(progress);
                    }
                });

                self._handler.execute(op, tx).await?;
            }
            FileOperation::CreateDir { path } => {
                let op = Operation::CreateDir { path };

                let (tx, mut rx) = mpsc::channel(100);
                let progress_tx_clone = progress_tx.clone();
                tokio::spawn(async move {
                    while let Some(progress) = rx.recv().await {
                        let _ = progress_tx_clone.send(progress);
                    }
                });

                self._handler.execute(op, tx).await?;
            }
            FileOperation::Rename { old_path, new_name } => {
                let new_path = old_path
                    .parent()
                    .map(|p| p.join(&new_name))
                    .unwrap_or_else(|| PathBuf::from(&new_name));

                let op = Operation::Rename {
                    old: old_path,
                    new: new_path,
                };

                let (tx, mut rx) = mpsc::channel(100);
                let progress_tx_clone = progress_tx.clone();
                tokio::spawn(async move {
                    while let Some(progress) = rx.recv().await {
                        let _ = progress_tx_clone.send(progress);
                    }
                });

                self._handler.execute(op, tx).await?;
            }
        }

        Ok(())
    }

    pub fn create_confirm_dialog(operation: &FileOperation) -> Dialog {
        let (title, message) = match operation {
            FileOperation::Copy {
                sources,
                destination,
            } => {
                let count = sources.len();
                let dest_name = destination
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                (
                    "Confirm Copy",
                    format!("Copy {} item(s) to {}?", count, dest_name),
                )
            }
            FileOperation::Move {
                sources,
                destination,
            } => {
                let count = sources.len();
                let dest_name = destination
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                (
                    "Confirm Move",
                    format!("Move {} item(s) to {}?", count, dest_name),
                )
            }
            FileOperation::Delete { targets } => {
                let count = targets.len();
                (
                    "Confirm Delete",
                    format!("Delete {} item(s)? This cannot be undone.", count),
                )
            }
            FileOperation::DeleteToTrash { targets } => {
                let count = targets.len();
                (
                    "Confirm Move to Trash",
                    format!("Move {} item(s) to trash?", count),
                )
            }
            FileOperation::RestoreFromTrash { targets } => {
                let count = targets.len();
                (
                    "Confirm Restore",
                    format!("Restore {} item(s) from trash?", count),
                )
            }
            FileOperation::CopyToClipboard { paths } => {
                let count = paths.len();
                (
                    "Copy to Clipboard",
                    format!("Copy {} item(s) to clipboard?", count),
                )
            }
            FileOperation::PasteFromClipboard { destination } => {
                let dest_name = destination
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                (
                    "Paste from Clipboard",
                    format!("Paste clipboard contents to {}?", dest_name),
                )
            }
            FileOperation::CreateDir { path } => {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                ("Create Directory", format!("Create directory '{}'?", name))
            }
            FileOperation::Rename { old_path, new_name } => {
                let old_name = old_path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                (
                    "Rename",
                    format!("Rename '{}' to '{}'?", old_name, new_name),
                )
            }
        };

        Dialog::Confirm(ConfirmDialog::new(title, message))
    }
}
