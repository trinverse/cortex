use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub shortcut: Option<String>,
    pub category: CommandCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
    Navigation,
    FileOperations,
    View,
    Edit,
    Search,
    System,
    Help,
}

impl CommandCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Navigation => "Navigation",
            Self::FileOperations => "File Operations",
            Self::View => "View",
            Self::Edit => "Edit",
            Self::Search => "Search",
            Self::System => "System",
            Self::Help => "Help",
        }
    }
}

pub struct CommandPalette {
    commands: Vec<Command>,
    filtered_commands: Vec<Command>,
    selected_index: usize,
}

impl CommandPalette {
    pub fn new() -> Self {
        let commands = vec![
            // System commands
            Command {
                name: "/exit".to_string(),
                description: "Exit Cortex".to_string(),
                shortcut: Some("Ctrl+Q".to_string()),
                category: CommandCategory::System,
            },
            Command {
                name: "/quit".to_string(),
                description: "Quit Cortex (alias for exit)".to_string(),
                shortcut: Some("Ctrl+Q".to_string()),
                category: CommandCategory::System,
            },
            Command {
                name: "/reload".to_string(),
                description: "Reload file panels".to_string(),
                shortcut: Some("Ctrl+R".to_string()),
                category: CommandCategory::System,
            },
            Command {
                name: "/refresh".to_string(),
                description: "Refresh current panel".to_string(),
                shortcut: None,
                category: CommandCategory::System,
            },
            
            // Help commands
            Command {
                name: "/keys".to_string(),
                description: "Show keyboard shortcuts".to_string(),
                shortcut: None,
                category: CommandCategory::Help,
            },
            Command {
                name: "/about".to_string(),
                description: "About Cortex".to_string(),
                shortcut: None,
                category: CommandCategory::Help,
            },
            
            // View commands
            Command {
                name: "/hex".to_string(),
                description: "View file in hex mode".to_string(),
                shortcut: None,
                category: CommandCategory::View,
            },
            Command {
                name: "/preview".to_string(),
                description: "Quick preview in opposite panel".to_string(),
                shortcut: Some("Ctrl+Q".to_string()),
                category: CommandCategory::View,
            },
            
            // Search commands
            Command {
                name: "/find".to_string(),
                description: "Find files by name".to_string(),
                shortcut: Some("Ctrl+F".to_string()),
                category: CommandCategory::Search,
            },
            Command {
                name: "/grep".to_string(),
                description: "Search in file contents".to_string(),
                shortcut: None,
                category: CommandCategory::Search,
            },
            Command {
                name: "/filter".to_string(),
                description: "Quick filter current panel".to_string(),
                shortcut: Some("Ctrl+F".to_string()),
                category: CommandCategory::Search,
            },
            Command {
                name: "/locate".to_string(),
                description: "Locate file in system".to_string(),
                shortcut: None,
                category: CommandCategory::Search,
            },
            
            // Navigation commands
            Command {
                name: "/cd".to_string(),
                description: "Change directory".to_string(),
                shortcut: None,
                category: CommandCategory::Navigation,
            },
            Command {
                name: "/goto".to_string(),
                description: "Go to specific path".to_string(),
                shortcut: None,
                category: CommandCategory::Navigation,
            },
            Command {
                name: "/back".to_string(),
                description: "Go back in history".to_string(),
                shortcut: Some("Alt+Left".to_string()),
                category: CommandCategory::Navigation,
            },
            Command {
                name: "/forward".to_string(),
                description: "Go forward in history".to_string(),
                shortcut: Some("Alt+Right".to_string()),
                category: CommandCategory::Navigation,
            },
            Command {
                name: "/home".to_string(),
                description: "Go to home directory".to_string(),
                shortcut: None,
                category: CommandCategory::Navigation,
            },
            Command {
                name: "/root".to_string(),
                description: "Go to root directory".to_string(),
                shortcut: None,
                category: CommandCategory::Navigation,
            },
            
            // Panel commands
            Command {
                name: "/split".to_string(),
                description: "Split panel horizontally".to_string(),
                shortcut: None,
                category: CommandCategory::View,
            },
            Command {
                name: "/swap".to_string(),
                description: "Swap left and right panels".to_string(),
                shortcut: Some("Ctrl+U".to_string()),
                category: CommandCategory::View,
            },
            Command {
                name: "/sync".to_string(),
                description: "Sync panels to same directory".to_string(),
                shortcut: None,
                category: CommandCategory::View,
            },
            
            // Settings
            Command {
                name: "/hidden".to_string(),
                description: "Toggle hidden files".to_string(),
                shortcut: Some("Ctrl+H".to_string()),
                category: CommandCategory::View,
            },
            Command {
                name: "/sort".to_string(),
                description: "Change sort mode".to_string(),
                shortcut: None,
                category: CommandCategory::View,
            },
            Command {
                name: "/api-key".to_string(),
                description: "Configure API keys".to_string(),
                shortcut: Some("Ctrl+K".to_string()),
                category: CommandCategory::System,
            },
            Command {
                name: "/config".to_string(),
                description: "Open configuration".to_string(),
                shortcut: None,
                category: CommandCategory::System,
            },
            Command {
                name: "/theme".to_string(),
                description: "Change color theme".to_string(),
                shortcut: None,
                category: CommandCategory::System,
            },
        ];
        
        let mut palette = Self {
            commands: commands.clone(),
            filtered_commands: commands,
            selected_index: 0,
        };
        
        palette.sort_commands();
        palette
    }
    
    fn sort_commands(&mut self) {
        self.filtered_commands.sort_by(|a, b| {
            a.category.as_str().cmp(b.category.as_str())
                .then_with(|| a.name.cmp(&b.name))
        });
    }
    
    pub fn filter(&mut self, query: &str) {
        if query.is_empty() || query == "/" {
            self.filtered_commands = self.commands.clone();
            self.sort_commands();
        } else {
            let search = if query.starts_with('/') {
                &query[1..]
            } else {
                query
            }.to_lowercase();
            
            self.filtered_commands = self.commands
                .iter()
                .filter(|cmd| {
                    let name_match = cmd.name[1..].to_lowercase().contains(&search);
                    let desc_match = cmd.description.to_lowercase().contains(&search);
                    let category_match = cmd.category.as_str().to_lowercase().contains(&search);
                    name_match || desc_match || category_match
                })
                .cloned()
                .collect();
            
            // Sort by relevance (name matches first, then description matches)
            self.filtered_commands.sort_by(|a, b| {
                let a_name_match = a.name[1..].to_lowercase().starts_with(&search);
                let b_name_match = b.name[1..].to_lowercase().starts_with(&search);
                
                match (a_name_match, b_name_match) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.category.as_str().cmp(b.category.as_str())
                        .then_with(|| a.name.cmp(&b.name))
                }
            });
        }
        
        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_commands.len() && !self.filtered_commands.is_empty() {
            self.selected_index = 0;
        }
    }
    
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.filtered_commands.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
    
    pub fn get_selected(&self) -> Option<&Command> {
        self.filtered_commands.get(self.selected_index)
    }
    
    pub fn get_filtered_commands(&self) -> &[Command] {
        &self.filtered_commands
    }
    
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }
    
    pub fn reset(&mut self) {
        self.filtered_commands = self.commands.clone();
        self.sort_commands();
        self.selected_index = 0;
    }
    
    // Group commands by category for display
    pub fn get_grouped_commands(&self) -> HashMap<CommandCategory, Vec<&Command>> {
        let mut grouped = HashMap::new();
        
        for cmd in &self.filtered_commands {
            grouped.entry(cmd.category.clone())
                .or_insert_with(Vec::new)
                .push(cmd);
        }
        
        grouped
    }
}