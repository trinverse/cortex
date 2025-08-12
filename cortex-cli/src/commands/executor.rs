// Command execution
use anyhow::Result;
use crate::app::App;
use super::{parse_command, CommandType};

/// Process a command from the command line
pub async fn process_command(app: &mut App, command: &str) -> Result<()> {
    match parse_command(command) {
        CommandType::Special(cmd) => {
            super::handle_special_command(app, &cmd).await?;
        }
        CommandType::Plugin(cmd) => {
            super::handle_plugin_command(app, &cmd).await?;
        }
        CommandType::Shell(cmd) => {
            execute_shell_command(app, cmd).await?;
        }
    }
    Ok(())
}

/// Execute a shell command
async fn execute_shell_command(app: &mut App, command: String) -> Result<()> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;
    use tokio::sync::mpsc;

    // Special handling for cd command
    if command.trim().starts_with("cd ") {
        let path = command.trim()[3..].trim();
        let new_dir = if path.starts_with('/') {
            std::path::PathBuf::from(path)
        } else {
            app.state.active_panel().current_dir.join(path)
        };

        if new_dir.exists() && new_dir.is_dir() {
            let cache = app.state.directory_cache.clone();
            {
                let panel = app.state.active_panel_mut();
                panel.current_dir = new_dir.clone();
                panel.selected_index = 0;
                panel.view_offset = 0;
                
                use crate::app::lifecycle::refresh_panel_with_cache;
                refresh_panel_with_cache(panel, &cache)?;
            }
            
            app.state.set_status_message(format!("Changed to: {}", new_dir.display()));
        } else {
            app.state.set_status_message(format!("Directory not found: {}", new_dir.display()));
        }
        return Ok(());
    }

    // Execute other commands
    app.state.clear_command_output();
    app.state.set_command_running(true);
    
    let working_dir = app.state.active_panel().current_dir.clone();
    app.state.add_command_output(format!("[WORKING DIR] {}", working_dir.display()));
    app.state.add_command_output(format!("[STARTED] {}", command));

    let (tx, rx) = mpsc::channel(100);
    app.command_output_rx = Some(rx);

    tokio::spawn(async move {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .current_dir(&working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .current_dir(&working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        };

        match output {
            Ok(mut child) => {
                // Read stdout
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if tx.send(line).await.is_err() {
                            break;
                        }
                    }
                }

                // Read stderr
                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if tx.send(format!("[ERROR] {}", line)).await.is_err() {
                            break;
                        }
                    }
                }

                match child.wait().await {
                    Ok(status) => {
                        let msg = if status.success() {
                            format!("[COMPLETED] Command finished successfully")
                        } else {
                            format!("[COMPLETED] Command exited with code: {:?}", status.code())
                        };
                        let _ = tx.send(msg).await;
                    }
                    Err(e) => {
                        let _ = tx.send(format!("[ERROR] Failed to wait for command: {}", e)).await;
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(format!("[ERROR] Failed to execute command: {}", e)).await;
            }
        }
    });

    app.state.set_command_running(false);
    Ok(())
}