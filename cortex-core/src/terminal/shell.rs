use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub shell_type: ShellType,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub working_dir: Option<PathBuf>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self::detect_system_shell()
    }
}

impl ShellConfig {
    pub fn detect_system_shell() -> Self {
        if cfg!(windows) {
            // On Windows, prefer PowerShell if available, otherwise cmd
            if which::which("pwsh").is_ok() {
                Self::powershell_core()
            } else if which::which("powershell").is_ok() {
                Self::powershell()
            } else {
                Self::cmd()
            }
        } else {
            // On Unix-like systems, check SHELL environment variable
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            
            if shell.contains("zsh") {
                Self::zsh()
            } else if shell.contains("fish") {
                Self::fish()
            } else if shell.contains("bash") {
                Self::bash()
            } else {
                Self::custom(shell)
            }
        }
    }
    
    pub fn bash() -> Self {
        Self {
            shell_type: ShellType::Bash,
            command: "bash".to_string(),
            args: vec!["--login".to_string()],
            env: vec![
                ("TERM".to_string(), "xterm-256color".to_string()),
                ("COLORTERM".to_string(), "truecolor".to_string()),
            ],
            working_dir: None,
        }
    }
    
    pub fn zsh() -> Self {
        Self {
            shell_type: ShellType::Zsh,
            command: "zsh".to_string(),
            args: vec!["-l".to_string()],
            env: vec![
                ("TERM".to_string(), "xterm-256color".to_string()),
                ("COLORTERM".to_string(), "truecolor".to_string()),
            ],
            working_dir: None,
        }
    }
    
    pub fn fish() -> Self {
        Self {
            shell_type: ShellType::Fish,
            command: "fish".to_string(),
            args: vec!["-l".to_string()],
            env: vec![
                ("TERM".to_string(), "xterm-256color".to_string()),
                ("COLORTERM".to_string(), "truecolor".to_string()),
            ],
            working_dir: None,
        }
    }
    
    pub fn powershell() -> Self {
        Self {
            shell_type: ShellType::PowerShell,
            command: "powershell".to_string(),
            args: vec!["-NoLogo".to_string()],
            env: vec![],
            working_dir: None,
        }
    }
    
    pub fn powershell_core() -> Self {
        Self {
            shell_type: ShellType::PowerShell,
            command: "pwsh".to_string(),
            args: vec!["-NoLogo".to_string()],
            env: vec![],
            working_dir: None,
        }
    }
    
    pub fn cmd() -> Self {
        Self {
            shell_type: ShellType::Cmd,
            command: "cmd.exe".to_string(),
            args: vec![],
            env: vec![],
            working_dir: None,
        }
    }
    
    pub fn custom(command: String) -> Self {
        Self {
            shell_type: ShellType::Custom(command.clone()),
            command,
            args: vec![],
            env: vec![
                ("TERM".to_string(), "xterm-256color".to_string()),
                ("COLORTERM".to_string(), "truecolor".to_string()),
            ],
            working_dir: None,
        }
    }
    
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }
    
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.env.push((key, value));
        self
    }
    
    pub fn build_command(&self) -> String {
        let mut cmd = self.command.clone();
        for arg in &self.args {
            cmd.push(' ');
            cmd.push_str(arg);
        }
        cmd
    }
}