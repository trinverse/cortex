use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct CommandPaletteDialog {
    pub input: String,
    pub cursor_position: usize,
    pub selected_index: usize,
    pub filtered_commands: Vec<CommandInfo>,
    pub all_commands: Vec<CommandInfo>,
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub shortcut: Option<String>,
    pub category: String,
}

impl Default for CommandPaletteDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPaletteDialog {
    pub fn new() -> Self {
        let all_commands = Self::get_all_commands();
        Self {
            input: "/".to_string(),
            cursor_position: 1,
            selected_index: 0,
            filtered_commands: all_commands.clone(),
            all_commands,
        }
    }

    fn get_all_commands() -> Vec<CommandInfo> {
        vec![
            // System
            CommandInfo {
                name: "/exit".to_string(),
                description: "Exit Cortex".to_string(),
                shortcut: Some("Ctrl+Q".to_string()),
                category: "System".to_string(),
            },
            CommandInfo {
                name: "/reload".to_string(),
                description: "Reload file panels".to_string(),
                shortcut: Some("Ctrl+R".to_string()),
                category: "System".to_string(),
            },
            CommandInfo {
                name: "/restart".to_string(),
                description: "Restart Cortex application".to_string(),
                shortcut: None,
                category: "System".to_string(),
            },
            CommandInfo {
                name: "/help".to_string(),
                description: "Show help dialog".to_string(),
                shortcut: Some("F1".to_string()),
                category: "Help".to_string(),
            },
            // File operations
            CommandInfo {
                name: "/copy".to_string(),
                description: "Copy selected files".to_string(),
                shortcut: Some("F5".to_string()),
                category: "Files".to_string(),
            },
            CommandInfo {
                name: "/move".to_string(),
                description: "Move selected files".to_string(),
                shortcut: Some("F6".to_string()),
                category: "Files".to_string(),
            },
            CommandInfo {
                name: "/delete".to_string(),
                description: "Delete selected files".to_string(),
                shortcut: Some("F8".to_string()),
                category: "Files".to_string(),
            },
            CommandInfo {
                name: "/mkdir".to_string(),
                description: "Create new directory".to_string(),
                shortcut: Some("F7".to_string()),
                category: "Files".to_string(),
            },
            CommandInfo {
                name: "/rename".to_string(),
                description: "Rename current file".to_string(),
                shortcut: None,
                category: "Files".to_string(),
            },
            // View
            CommandInfo {
                name: "/view".to_string(),
                description: "View current file".to_string(),
                shortcut: Some("F3".to_string()),
                category: "View".to_string(),
            },
            CommandInfo {
                name: "/edit".to_string(),
                description: "Edit current file".to_string(),
                shortcut: Some("F4".to_string()),
                category: "View".to_string(),
            },
            CommandInfo {
                name: "/filter".to_string(),
                description: "Quick filter current panel".to_string(),
                shortcut: Some("Ctrl+F".to_string()),
                category: "Search".to_string(),
            },
            CommandInfo {
                name: "/find".to_string(),
                description: "Find files by name".to_string(),
                shortcut: Some("Alt+F7".to_string()),
                category: "Search".to_string(),
            },
            // Navigation
            CommandInfo {
                name: "/cd".to_string(),
                description: "Change directory".to_string(),
                shortcut: None,
                category: "Navigation".to_string(),
            },
            CommandInfo {
                name: "/home".to_string(),
                description: "Go to home directory".to_string(),
                shortcut: None,
                category: "Navigation".to_string(),
            },
            CommandInfo {
                name: "/root".to_string(),
                description: "Go to root directory".to_string(),
                shortcut: None,
                category: "Navigation".to_string(),
            },
            // Settings
            CommandInfo {
                name: "/hidden".to_string(),
                description: "Toggle hidden files".to_string(),
                shortcut: Some("Ctrl+H".to_string()),
                category: "View".to_string(),
            },
            CommandInfo {
                name: "/sort".to_string(),
                description: "Change sort mode".to_string(),
                shortcut: None,
                category: "View".to_string(),
            },
            // Remote Connections
            CommandInfo {
                name: "/sftp".to_string(),
                description: "Connect to SFTP server".to_string(),
                shortcut: None,
                category: "Remote".to_string(),
            },
            CommandInfo {
                name: "/ftp".to_string(),
                description: "Connect to FTP server".to_string(),
                shortcut: None,
                category: "Remote".to_string(),
            },
            // Configuration
            CommandInfo {
                name: "/config".to_string(),
                description: "Open configuration settings".to_string(),
                shortcut: None,
                category: "Settings".to_string(),
            },
            CommandInfo {
                name: "/settings".to_string(),
                description: "Open settings dialog".to_string(),
                shortcut: None,
                category: "Settings".to_string(),
            },
            CommandInfo {
                name: "/theme".to_string(),
                description: "Change color theme".to_string(),
                shortcut: None,
                category: "Settings".to_string(),
            },
            // Plugin System
            CommandInfo {
                name: "/plugin".to_string(),
                description: "Manage plugins".to_string(),
                shortcut: None,
                category: "Plugins".to_string(),
            },
            CommandInfo {
                name: "/stats".to_string(),
                description: "Show file statistics".to_string(),
                shortcut: None,
                category: "Plugins".to_string(),
            },
            CommandInfo {
                name: "/analyze".to_string(),
                description: "Analyze directory contents".to_string(),
                shortcut: None,
                category: "Plugins".to_string(),
            },
            CommandInfo {
                name: "/backup".to_string(),
                description: "Backup current file".to_string(),
                shortcut: None,
                category: "Plugins".to_string(),
            },
            CommandInfo {
                name: "/duplicate".to_string(),
                description: "Duplicate current file".to_string(),
                shortcut: None,
                category: "Plugins".to_string(),
            },
            CommandInfo {
                name: "/compress".to_string(),
                description: "Compress selected files".to_string(),
                shortcut: None,
                category: "Plugins".to_string(),
            },
            // File Monitoring
            CommandInfo {
                name: "/monitor".to_string(),
                description: "Toggle file monitoring on/off".to_string(),
                shortcut: None,
                category: "Monitoring".to_string(),
            },
            CommandInfo {
                name: "/watch".to_string(),
                description: "Show watched directories".to_string(),
                shortcut: None,
                category: "Monitoring".to_string(),
            },
            CommandInfo {
                name: "/notifications".to_string(),
                description: "Toggle notifications visibility".to_string(),
                shortcut: None,
                category: "Monitoring".to_string(),
            },
        ]
    }

    pub fn filter_commands(&mut self) {
        let query = self.input.trim().to_lowercase();

        if query == "/" {
            // Show all commands
            self.filtered_commands = self.all_commands.clone();
        } else if let Some(search) = query.strip_prefix('/') {
            // Filter based on text after /
            self.filtered_commands = self
                .all_commands
                .iter()
                .filter(|cmd| {
                    cmd.name[1..].to_lowercase().contains(search)
                        || cmd.description.to_lowercase().contains(search)
                        || cmd.category.to_lowercase().contains(search)
                })
                .cloned()
                .collect();

            // Sort by relevance
            self.filtered_commands.sort_by(|a, b| {
                let a_starts = a.name[1..].to_lowercase().starts_with(search);
                let b_starts = b.name[1..].to_lowercase().starts_with(search);

                match (a_starts, b_starts) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a
                        .category
                        .cmp(&b.category)
                        .then_with(|| a.name.cmp(&b.name)),
                }
            });
        }

        // Reset selection if needed
        if self.selected_index >= self.filtered_commands.len() && !self.filtered_commands.is_empty()
        {
            self.selected_index = 0;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.filter_commands();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 1 {
            // Don't delete the initial /
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
            self.filter_commands();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 1 {
            // Don't go before /
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
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

    pub fn get_selected_command(&self) -> Option<String> {
        self.filtered_commands
            .get(self.selected_index)
            .map(|cmd| cmd.name.clone())
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Min(5),    // Command list
                Constraint::Length(2), // Help line
            ])
            .split(area);

        // Input area
        let input_block = Block::default()
            .title(" Command Palette ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let input_inner = input_block.inner(chunks[0]);
        frame.render_widget(input_block, chunks[0]);

        let input_text =
            Paragraph::new(self.input.as_str()).style(Style::default().fg(Color::White));
        frame.render_widget(input_text, input_inner);

        // Show cursor
        frame.set_cursor_position((input_inner.x + self.cursor_position as u16, input_inner.y));

        // Command list
        let list_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Yellow));
        let list_inner = list_block.inner(chunks[1]);
        frame.render_widget(list_block, chunks[1]);

        if self.filtered_commands.is_empty() {
            let no_results = Paragraph::new("No matching commands")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(no_results, list_inner);
        } else {
            // Group commands by category
            let mut items = Vec::new();
            let mut current_category = String::new();

            for (idx, cmd) in self.filtered_commands.iter().enumerate() {
                // Add category header if changed
                if cmd.category != current_category {
                    current_category = cmd.category.clone();
                    items.push(ListItem::new(Line::from(vec![Span::styled(
                        format!(" {} ", current_category),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )])));
                }

                // Create command item
                let is_selected = idx == self.selected_index;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                let mut spans = vec![
                    Span::styled("  ", style),
                    Span::styled(
                        format!("{:<15}", cmd.name),
                        style.fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" ", style),
                    Span::styled(format!("{:<40}", cmd.description), style.fg(Color::White)),
                ];

                if let Some(ref shortcut) = cmd.shortcut {
                    spans.push(Span::styled(
                        format!(" [{}]", shortcut),
                        style.fg(Color::DarkGray),
                    ));
                }

                items.push(ListItem::new(Line::from(spans)));
            }

            let list = List::new(items);
            frame.render_widget(list, list_inner);
        }

        // Help line
        let help_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Yellow));
        let help_inner = help_block.inner(chunks[2]);
        frame.render_widget(help_block, chunks[2]);

        let help_text = " ↑↓: Navigate | Enter: Execute | Tab: Autocomplete | ESC: Cancel ";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_inner);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
