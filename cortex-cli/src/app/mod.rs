// Core application module
mod state;
pub mod lifecycle;
pub mod config;

pub use state::App;
pub use lifecycle::{run_app, initialize_app, cleanup_app};

use anyhow::Result;
use std::path::PathBuf;

/// Main entry point for the application
pub async fn start(initial_path: Option<PathBuf>) -> Result<()> {
    let mut app = initialize_app(initial_path).await?;
    run_app(&mut app).await?;
    cleanup_app(&mut app).await?;
    Ok(())
}