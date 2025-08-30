use anyhow::Result;
use clap::Parser;
use cortex_core::window::{WindowConfig, WindowManager, WindowMode};
use std::path::PathBuf;

mod app;
mod command;
mod operations;
mod update;

use app::App;
use update::UpdateManager;

#[derive(Parser, Debug)]
#[command(name = "cortex")]
#[command(about = "A modern orthodox file manager", long_about = None)]
struct Args {
    #[arg(help = "Directory to open")]
    path: Option<PathBuf>,

    #[arg(short, long, help = "Show version information")]
    version: bool,

    #[arg(long, help = "Check for updates")]
    check_updates: bool,

    #[arg(long, help = "Update to latest version")]
    update: bool,

    #[arg(short = 'w', long, help = "Run in windowed mode (opens in new window)")]
    windowed: bool,

    #[arg(short = 't', long, help = "Force terminal mode (stay in current terminal)")]
    terminal: bool,

    #[arg(long, help = "Start in fullscreen mode")]
    fullscreen: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    if args.version {
        println!("Cortex v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Determine window mode
    let window_mode = if args.terminal || !args.windowed {
        WindowMode::Terminal
    } else if args.fullscreen {
        WindowMode::Fullscreen
    } else if args.windowed {
        WindowMode::Windowed
    } else {
        WindowMode::Terminal
    };

    // Handle windowed mode (experimental)
    if window_mode != WindowMode::Terminal {
        #[cfg(target_os = "macos")]
        {
            return run_windowed_app_sync(args.path, window_mode);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let runtime = tokio::runtime::Runtime::new()?;
            return runtime.block_on(run_windowed_app(args.path, window_mode));
        }
    }

    // Run the terminal app
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_main(args))
}

async fn async_main(args: Args) -> Result<()> {
    // Handle update operations
    if args.check_updates || args.update {
        return handle_update_operations(args.check_updates, args.update).await;
    }

    // Create and run the main application
    let mut app = App::new(args.path).await?;
    app.run().await
}

async fn handle_update_operations(check_updates: bool, update: bool) -> Result<()> {
    let manager = UpdateManager::new()?;

    if check_updates {
        println!("Checking for updates...");
        match manager.check_for_updates().await {
            Ok(Some(update_info)) => {
                println!("Update available: v{}", update_info.version);
                println!("Release date: {}", update_info.release_date);
                println!("Download size: {} bytes", update_info.size);
                println!("\nRelease notes:\n{}", update_info.release_notes);
                println!("\nRun 'cortex --update' to install");
            }
            Ok(None) => {
                println!("You are running the latest version");
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
            }
        }
        return Ok(());
    }

    if update {
        println!("Checking for updates...");
        match manager.check_for_updates().await {
            Ok(Some(update_info)) => {
                println!("Found update: v{}", update_info.version);
                println!("Downloading...");

                if let Err(e) = manager.install_update(update_info).await {
                    eprintln!("Failed to install update: {}", e);
                } else {
                    println!("Update installed successfully!");
                    println!("Please restart Cortex to use the new version");
                }
            }
            Ok(None) => {
                println!("You are already running the latest version");
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn run_windowed_app_sync(_initial_path: Option<PathBuf>, mode: WindowMode) -> Result<()> {
    println!("Starting Cortex in windowed mode (macOS)...");

    let config = WindowConfig {
        title: format!("Cortex File Manager v{}", env!("CARGO_PKG_VERSION")),
        width: 1280,
        height: 800,
        mode,
        resizable: true,
        decorations: true,
    };

    let mut manager = WindowManager::new(config.clone());
    let _window = manager.create_window()?;
    manager.run_event_loop()?;

    Ok(())
}

#[allow(dead_code)]
async fn run_windowed_app(_initial_path: Option<PathBuf>, _mode: WindowMode) -> Result<()> {
    println!("Windowed mode is not yet fully implemented for this platform");
    println!("Please use terminal mode instead: cortex --terminal");
    Ok(())
}