use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

// Module declarations
mod app;
mod command;
mod commands;
mod connections;
mod handlers;
mod operations;
mod update;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    if args.version {
        println!("Cortex v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Handle update checking
    if args.check_updates {
        use update::UpdateManager;

        println!("Checking for updates...");
        let manager = UpdateManager::new()?;

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

    // Handle update installation
    if args.update {
        use update::UpdateManager;

        println!("Checking for updates...");
        let manager = UpdateManager::new()?;

        match manager.check_for_updates().await {
            Ok(Some(update_info)) => {
                println!("Found update: v{}", update_info.version);
                println!("Downloading...");

                if let Err(e) = manager.install_update(update_info).await {
                    eprintln!("Failed to install update: {}", e);
                    std::process::exit(1);
                }

                println!("Update installed successfully! Please restart cortex.");
                std::process::exit(0);
            }
            Ok(None) => {
                println!("You are already running the latest version");
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Run the main application
    app::start(args.path).await
}