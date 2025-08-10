use anyhow::Result;
use cortex_core::AppState;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use tokio::sync::mpsc;

pub struct CommandProcessor;

impl CommandProcessor {
    /// Process command line input with special variables
    /// %f - current file name
    /// %F - list of marked files or current file
    /// %d - current directory
    /// %D - opposite panel directory
    /// %p - full path of current file
    /// %P - full paths of marked files
    pub fn expand_command(command: &str, state: &AppState) -> String {
        let mut expanded = command.to_string();
        let active_panel = state.active_panel();
        let inactive_panel = state.inactive_panel();

        // Get current file
        let current_file = active_panel
            .current_entry()
            .map(|e| e.name.clone())
            .unwrap_or_default();

        let current_path = active_panel
            .current_entry()
            .map(|e| e.path.to_string_lossy().to_string())
            .unwrap_or_default();

        // Get marked files or current file
        let marked_files = if !active_panel.marked_files.is_empty() {
            active_panel
                .marked_files
                .iter()
                .filter_map(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        } else if !current_file.is_empty() && current_file != ".." {
            current_file.clone()
        } else {
            String::new()
        };

        let marked_paths = if !active_panel.marked_files.is_empty() {
            active_panel
                .marked_files
                .iter()
                .map(|p| format!("\"{}\"", p.to_string_lossy()))
                .collect::<Vec<_>>()
                .join(" ")
        } else if !current_path.is_empty() && current_file != ".." {
            format!("\"{}\"", current_path)
        } else {
            String::new()
        };

        // Replace variables
        expanded = expanded.replace("%f", &current_file);
        expanded = expanded.replace("%F", &marked_files);
        expanded = expanded.replace("%d", &active_panel.current_dir.to_string_lossy());
        expanded = expanded.replace("%D", &inactive_panel.current_dir.to_string_lossy());
        expanded = expanded.replace("%p", &current_path);
        expanded = expanded.replace("%P", &marked_paths);

        expanded
    }

    pub async fn execute_command(command: &str, state: &AppState) -> Result<String> {
        let expanded = Self::expand_command(command, state);

        // Check for built-in commands
        let parts: Vec<&str> = expanded.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }

        match parts[0] {
            "pwd" => Ok(state
                .active_panel()
                .current_dir
                .to_string_lossy()
                .to_string()),
            "exit" | "quit" => Ok("exit".to_string()),
            "/monitor" => {
                if state.is_file_monitoring_active() {
                    Ok("File monitoring is currently ENABLED".to_string())
                } else {
                    Ok("File monitoring is currently DISABLED".to_string())
                }
            }
            "/watch" => {
                let watched = state.left_panel.current_dir.to_string_lossy().to_string()
                    + ", "
                    + state.right_panel.current_dir.to_string_lossy().as_ref();
                Ok(format!("Watched directories: {}", watched))
            }
            "/notifications" => Ok("Notifications toggle command - handled by UI".to_string()),
            "/config" => Ok("Opening configuration - handled by UI".to_string()),
            _ => {
                // Execute external command
                Self::execute_external_command(&expanded).await
            }
        }
    }

    async fn execute_external_command(command: &str) -> Result<String> {
        let output = if cfg!(target_os = "windows") {
            AsyncCommand::new("cmd")
                .args(["/C", command])
                .output()
                .await?
        } else {
            AsyncCommand::new("sh")
                .args(["-c", command])
                .output()
                .await?
        };

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok(format!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub async fn execute_streaming_command_in_dir(
        command: &str,
        current_dir: &Path,
        output_sender: mpsc::Sender<String>,
    ) -> Result<i32> {
        // Send start message
        let _ = output_sender.send(format!("[STARTED] Running: {}", command)).await;
        let _ = output_sender.send(format!("[WORKING DIR] {}", current_dir.display())).await;
        
        let mut child = if cfg!(target_os = "windows") {
            AsyncCommand::new("cmd")
                .args(["/C", command])
                .current_dir(current_dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?
        } else {
            AsyncCommand::new("sh")
                .args(["-c", command])
                .current_dir(current_dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?
        };

        // Read stdout
        let stdout_handle = if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let sender_clone = output_sender.clone();
            
            Some(tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        let _ = sender_clone.send(line).await;
                    }
                }
            }))
        } else {
            None
        };

        // Read stderr
        let stderr_handle = if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let sender_clone = output_sender.clone();
            
            Some(tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        let _ = sender_clone.send(format!("[ERROR] {}", line)).await;
                    }
                }
            }))
        } else {
            None
        };

        // Wait for process to complete
        let status = child.wait().await?;
        let exit_code = status.code().unwrap_or(-1);
        
        // Wait for output streams to finish
        if let Some(handle) = stdout_handle {
            let _ = handle.await;
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.await;
        }
        
        // Send completion message
        let _ = output_sender.send(format!("[COMPLETED] Process finished with exit code: {}", exit_code)).await;
        
        Ok(exit_code)
    }

    pub fn parse_cd_path(args: &str, current_dir: &Path) -> Option<PathBuf> {
        if args.is_empty() {
            return dirs::home_dir();
        }

        let path = if args.starts_with('/') || args.starts_with('~') {
            // Absolute path or home directory
            if args.starts_with('~') {
                dirs::home_dir()?.join(&args[2..])
            } else {
                PathBuf::from(args)
            }
        } else {
            // Relative path
            current_dir.join(args)
        };

        if path.exists() && path.is_dir() {
            Some(path.canonicalize().ok()?)
        } else {
            None
        }
    }
}
