use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Operation {
    Copy { src: PathBuf, dst: PathBuf },
    Move { src: PathBuf, dst: PathBuf },
    Delete { path: PathBuf },
    CreateDir { path: PathBuf },
    Rename { old: PathBuf, new: PathBuf },
}

#[derive(Debug, Clone)]
pub enum OperationProgress {
    Started { operation: String },
    Progress { current: u64, total: u64, message: String },
    Completed { operation: String },
    Failed { operation: String, error: String },
}

#[async_trait]
pub trait OperationHandler: Send + Sync {
    async fn execute(&self, operation: Operation, progress: mpsc::Sender<OperationProgress>) -> Result<()>;
    async fn can_execute(&self, operation: &Operation) -> bool;
}

pub struct DefaultOperationHandler;

#[async_trait]
impl OperationHandler for DefaultOperationHandler {
    async fn execute(&self, operation: Operation, progress: mpsc::Sender<OperationProgress>) -> Result<()> {
        use crate::fs::FileSystem;
        
        let op_string = format!("{:?}", operation);
        let _ = progress.send(OperationProgress::Started {
            operation: op_string.clone(),
        }).await;

        let result = match operation {
            Operation::Copy { src, dst } => {
                tokio::task::spawn_blocking(move || FileSystem::copy_entry(&src, &dst)).await?
            }
            Operation::Move { src, dst } => {
                tokio::task::spawn_blocking(move || FileSystem::move_entry(&src, &dst)).await?
            }
            Operation::Delete { path } => {
                tokio::task::spawn_blocking(move || FileSystem::delete_entry(&path)).await?
            }
            Operation::CreateDir { path } => {
                tokio::task::spawn_blocking(move || FileSystem::create_directory(&path)).await?
            }
            Operation::Rename { old, new } => {
                tokio::task::spawn_blocking(move || FileSystem::move_entry(&old, &new)).await?
            }
        };

        match result {
            Ok(_) => {
                let _ = progress.send(OperationProgress::Completed {
                    operation: op_string,
                }).await;
                Ok(())
            }
            Err(e) => {
                let _ = progress.send(OperationProgress::Failed {
                    operation: op_string,
                    error: e.to_string(),
                }).await;
                Err(e)
            }
        }
    }

    async fn can_execute(&self, _operation: &Operation) -> bool {
        true
    }
}

pub struct OperationQueue {
    operations: Vec<Operation>,
    handler: Box<dyn OperationHandler>,
}

impl OperationQueue {
    pub fn new(handler: Box<dyn OperationHandler>) -> Self {
        Self {
            operations: Vec::new(),
            handler,
        }
    }

    pub fn add(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    pub async fn execute_all(&mut self, progress: mpsc::Sender<OperationProgress>) -> Result<()> {
        for operation in self.operations.drain(..) {
            if self.handler.can_execute(&operation).await {
                self.handler.execute(operation, progress.clone()).await?;
            }
        }
        Ok(())
    }
}