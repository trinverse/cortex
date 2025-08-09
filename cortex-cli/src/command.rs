use anyhow::Result;
use cortex_core::AppState;
use std::path::PathBuf;
use tokio::process::Command as AsyncCommand;

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
        let current_file = active_panel.current_entry()
            .map(|e| e.name.clone())
            .unwrap_or_default();
        
        let current_path = active_panel.current_entry()
            .map(|e| e.path.to_string_lossy().to_string())
            .unwrap_or_default();
        
        // Get marked files or current file
        let marked_files = if !active_panel.marked_files.is_empty() {
            active_panel.marked_files
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
            active_panel.marked_files
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
            "cd" => {
                // Handle cd command internally
                if parts.len() > 1 {
                    Ok(format!("cd: {}", parts[1]))
                } else {
                    Ok("cd: missing argument".to_string())
                }
            }
            "pwd" => {
                Ok(state.active_panel().current_dir.to_string_lossy().to_string())
            }
            "exit" | "quit" => {
                Ok("exit".to_string())
            }
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
    
    pub fn parse_cd_path(args: &str, current_dir: &PathBuf) -> Option<PathBuf> {
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