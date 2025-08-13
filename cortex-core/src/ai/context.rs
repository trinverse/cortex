use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContext {
    pub current_dir: PathBuf,
    pub selected_files: Vec<PathBuf>,
    pub command_history: Vec<String>,
    pub additional_context: Option<String>,
}

impl AIContext {
    pub fn new(current_dir: PathBuf) -> Self {
        Self {
            current_dir,
            selected_files: Vec::new(),
            command_history: Vec::new(),
            additional_context: None,
        }
    }
    
    pub fn with_files(mut self, files: Vec<PathBuf>) -> Self {
        self.selected_files = files;
        self
    }
    
    pub fn with_history(mut self, history: Vec<String>) -> Self {
        self.command_history = history;
        self
    }
    
    pub fn with_additional_context(mut self, context: String) -> Self {
        self.additional_context = Some(context);
        self
    }
    
    pub fn to_prompt_context(&self) -> String {
        let mut context = format!("Current directory: {}\n", self.current_dir.display());
        
        if !self.selected_files.is_empty() {
            context.push_str("Selected files:\n");
            for file in &self.selected_files {
                if let Some(name) = file.file_name() {
                    context.push_str(&format!("  - {}\n", name.to_string_lossy()));
                }
            }
        }
        
        if !self.command_history.is_empty() {
            context.push_str("\nRecent commands:\n");
            for cmd in self.command_history.iter().take(5) {
                context.push_str(&format!("  - {}\n", cmd));
            }
        }
        
        if let Some(additional) = &self.additional_context {
            context.push_str(&format!("\nAdditional context:\n{}\n", additional));
        }
        
        context
    }
}