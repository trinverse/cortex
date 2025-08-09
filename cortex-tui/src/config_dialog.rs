use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};
use cortex_core::Config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigTab {
    General,
    Panels,
    Colors,
    Plugins,
    Network,
    Keybindings,
}

impl ConfigTab {
    pub fn all() -> Vec<Self> {
        vec![
            Self::General,
            Self::Panels, 
            Self::Colors,
            Self::Plugins,
            Self::Network,
            Self::Keybindings,
        ]
    }
    
    pub fn title(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Panels => "Panels",
            Self::Colors => "Colors",
            Self::Plugins => "Plugins",
            Self::Network => "Network",
            Self::Keybindings => "Keys",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigDialog {
    pub config: Config,
    pub current_tab: ConfigTab,
    pub selected_index: usize,
    pub editing: bool,
    pub edit_value: String,
    pub edit_cursor: usize,
}

impl ConfigDialog {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            current_tab: ConfigTab::General,
            selected_index: 0,
            editing: false,
            edit_value: String::new(),
            edit_cursor: 0,
        }
    }
    
    pub fn next_tab(&mut self) {
        let tabs = ConfigTab::all();
        let current_idx = tabs.iter().position(|&t| t == self.current_tab).unwrap_or(0);
        let next_idx = (current_idx + 1) % tabs.len();
        self.current_tab = tabs[next_idx];
        self.selected_index = 0;
        self.cancel_edit();
    }
    
    pub fn prev_tab(&mut self) {
        let tabs = ConfigTab::all();
        let current_idx = tabs.iter().position(|&t| t == self.current_tab).unwrap_or(0);
        let prev_idx = if current_idx == 0 { tabs.len() - 1 } else { current_idx - 1 };
        self.current_tab = tabs[prev_idx];
        self.selected_index = 0;
        self.cancel_edit();
    }
    
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    pub fn move_selection_down(&mut self) {
        let max_items = self.get_max_items();
        if self.selected_index < max_items.saturating_sub(1) {
            self.selected_index += 1;
        }
    }
    
    pub fn get_max_items(&self) -> usize {
        match self.current_tab {
            ConfigTab::General => 8,
            ConfigTab::Panels => 4,
            ConfigTab::Colors => 4,
            ConfigTab::Plugins => 4,
            ConfigTab::Network => 5,
            ConfigTab::Keybindings => 1,
        }
    }
    
    pub fn start_edit(&mut self) {
        if self.editing {
            return;
        }
        
        self.editing = true;
        self.edit_value = self.get_current_value();
        self.edit_cursor = self.edit_value.len();
    }
    
    pub fn cancel_edit(&mut self) {
        self.editing = false;
        self.edit_value.clear();
        self.edit_cursor = 0;
    }
    
    pub fn confirm_edit(&mut self) {
        if !self.editing {
            return;
        }
        
        self.set_current_value(&self.edit_value.clone());
        self.cancel_edit();
    }
    
    pub fn get_current_value(&self) -> String {
        match self.current_tab {
            ConfigTab::General => match self.selected_index {
                0 => self.config.general.show_hidden.to_string(),
                1 => self.config.general.confirm_delete.to_string(),
                2 => self.config.general.show_icons.to_string(),
                3 => self.config.general.auto_reload.to_string(),
                4 => self.config.general.confirm_operations.to_string(),
                5 => self.config.general.enable_sound.to_string(),
                6 => self.config.general.terminal.clone(),
                7 => self.config.general.editor.clone(),
                _ => String::new(),
            },
            ConfigTab::Panels => match self.selected_index {
                0 => self.config.panels.default_sort.clone(),
                1 => self.config.panels.show_size.to_string(),
                2 => self.config.panels.show_permissions.to_string(),
                3 => self.config.panels.show_modified.to_string(),
                _ => String::new(),
            },
            ConfigTab::Colors => match self.selected_index {
                0 => self.config.colors.selection_bg.clone(),
                1 => self.config.colors.directory_fg.clone(),
                2 => self.config.colors.executable_fg.clone(),
                3 => self.config.colors.symlink_fg.clone(),
                _ => String::new(),
            },
            ConfigTab::Plugins => match self.selected_index {
                0 => self.config.plugins.enable_plugins.to_string(),
                1 => self.config.plugins.auto_reload_plugins.to_string(),
                2 => self.config.plugins.allow_unsafe_plugins.to_string(),
                3 => self.config.general.plugin_directory.clone(),
                _ => String::new(),
            },
            ConfigTab::Network => match self.selected_index {
                0 => self.config.network.connection_timeout.to_string(),
                1 => self.config.network.save_credentials.to_string(),
                2 => self.config.network.verify_ssl.to_string(),
                3 => self.config.network.enable_compression.to_string(),
                4 => self.config.network.known_hosts.join(","),
                _ => String::new(),
            },
            ConfigTab::Keybindings => "Custom keybindings (not yet editable)".to_string(),
        }
    }
    
    pub fn set_current_value(&mut self, value: &str) {
        match self.current_tab {
            ConfigTab::General => match self.selected_index {
                0 => self.config.general.show_hidden = value.parse().unwrap_or(false),
                1 => self.config.general.confirm_delete = value.parse().unwrap_or(true),
                2 => self.config.general.show_icons = value.parse().unwrap_or(false),
                3 => self.config.general.auto_reload = value.parse().unwrap_or(true),
                4 => self.config.general.confirm_operations = value.parse().unwrap_or(true),
                5 => self.config.general.enable_sound = value.parse().unwrap_or(false),
                6 => self.config.general.terminal = value.to_string(),
                7 => self.config.general.editor = value.to_string(),
                _ => {}
            },
            ConfigTab::Panels => match self.selected_index {
                0 => self.config.panels.default_sort = value.to_string(),
                1 => self.config.panels.show_size = value.parse().unwrap_or(true),
                2 => self.config.panels.show_permissions = value.parse().unwrap_or(true),
                3 => self.config.panels.show_modified = value.parse().unwrap_or(true),
                _ => {}
            },
            ConfigTab::Colors => match self.selected_index {
                0 => self.config.colors.selection_bg = value.to_string(),
                1 => self.config.colors.directory_fg = value.to_string(),
                2 => self.config.colors.executable_fg = value.to_string(),
                3 => self.config.colors.symlink_fg = value.to_string(),
                _ => {}
            },
            ConfigTab::Plugins => match self.selected_index {
                0 => self.config.plugins.enable_plugins = value.parse().unwrap_or(true),
                1 => self.config.plugins.auto_reload_plugins = value.parse().unwrap_or(false),
                2 => self.config.plugins.allow_unsafe_plugins = value.parse().unwrap_or(false),
                3 => self.config.general.plugin_directory = value.to_string(),
                _ => {}
            },
            ConfigTab::Network => match self.selected_index {
                0 => self.config.network.connection_timeout = value.parse().unwrap_or(30),
                1 => self.config.network.save_credentials = value.parse().unwrap_or(false),
                2 => self.config.network.verify_ssl = value.parse().unwrap_or(false),
                3 => self.config.network.enable_compression = value.parse().unwrap_or(false),
                4 => self.config.network.known_hosts = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                _ => {}
            },
            ConfigTab::Keybindings => {}
        }
    }
    
    pub fn insert_char(&mut self, c: char) {
        if self.editing {
            self.edit_value.insert(self.edit_cursor, c);
            self.edit_cursor += 1;
        }
    }
    
    pub fn delete_char(&mut self) {
        if self.editing && self.edit_cursor > 0 {
            self.edit_cursor -= 1;
            self.edit_value.remove(self.edit_cursor);
        }
    }
    
    pub fn move_cursor_left(&mut self) {
        if self.editing && self.edit_cursor > 0 {
            self.edit_cursor -= 1;
        }
    }
    
    pub fn move_cursor_right(&mut self) {
        if self.editing && self.edit_cursor < self.edit_value.len() {
            self.edit_cursor += 1;
        }
    }
    
    pub fn render(&self, frame: &mut Frame) {
        let area = centered_rect(85, 80, frame.area());
        frame.render_widget(Clear, area);
        
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(5),     // Content
                Constraint::Length(3),  // Help
            ])
            .split(area);
        
        // Tabs
        let tabs_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let tabs_inner = tabs_block.inner(chunks[0]);
        frame.render_widget(tabs_block, chunks[0]);
        
        let tab_titles: Vec<_> = ConfigTab::all().iter().map(|t| t.title()).collect();
        let tabs = Tabs::new(tab_titles)
            .block(Block::default())
            .highlight_style(Style::default().bg(Color::Green).fg(Color::Black))
            .select(ConfigTab::all().iter().position(|&t| t == self.current_tab).unwrap_or(0));
        frame.render_widget(tabs, tabs_inner);
        
        // Content
        let content_block = Block::default()
            .title(format!(" {} Settings ", self.current_tab.title()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let content_inner = content_block.inner(chunks[1]);
        frame.render_widget(content_block, chunks[1]);
        
        self.render_tab_content(frame, content_inner);
        
        // Help
        let help_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let help_inner = help_block.inner(chunks[2]);
        frame.render_widget(help_block, chunks[2]);
        
        let help_text = if self.editing {
            " Enter: Save | ESC: Cancel | ←→: Move cursor | Backspace: Delete "
        } else {
            " ↑↓: Navigate | ←→: Switch tabs | Enter: Edit | S: Save config | ESC: Close "
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_inner);
    }
    
    fn render_tab_content(&self, frame: &mut Frame, area: Rect) {
        let items = self.get_tab_items();
        
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(idx, (name, value))| {
                let is_selected = idx == self.selected_index;
                
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let display_value = if is_selected && self.editing {
                    &self.edit_value
                } else {
                    value
                };
                
                let spans = vec![
                    Span::styled("  ", style),
                    Span::styled(
                        format!("{:<25}", name),
                        style.fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    ),
                    Span::styled("  ", style),
                    Span::styled(
                        display_value.clone(),
                        style.fg(if is_selected && self.editing { Color::Yellow } else { Color::White })
                    ),
                ];
                
                ListItem::new(Line::from(spans))
            })
            .collect();
        
        let list = List::new(list_items);
        frame.render_widget(list, area);
        
        // Show cursor if editing
        if self.editing {
            let cursor_x = area.x + 27 + self.edit_cursor as u16;
            let cursor_y = area.y + self.selected_index as u16;
            if cursor_y < area.y + area.height {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
    
    fn get_tab_items(&self) -> Vec<(String, String)> {
        match self.current_tab {
            ConfigTab::General => vec![
                ("Show Hidden Files".to_string(), self.config.general.show_hidden.to_string()),
                ("Confirm Delete".to_string(), self.config.general.confirm_delete.to_string()),
                ("Show Icons".to_string(), self.config.general.show_icons.to_string()),
                ("Auto Reload".to_string(), self.config.general.auto_reload.to_string()),
                ("Confirm Operations".to_string(), self.config.general.confirm_operations.to_string()),
                ("Enable Sound".to_string(), self.config.general.enable_sound.to_string()),
                ("Default Terminal".to_string(), self.config.general.terminal.clone()),
                ("Default Editor".to_string(), self.config.general.editor.clone()),
            ],
            ConfigTab::Panels => vec![
                ("Default Sort Order".to_string(), self.config.panels.default_sort.clone()),
                ("Show File Size".to_string(), self.config.panels.show_size.to_string()),
                ("Show Permissions".to_string(), self.config.panels.show_permissions.to_string()),
                ("Show Modified Date".to_string(), self.config.panels.show_modified.to_string()),
            ],
            ConfigTab::Colors => vec![
                ("Selection Background".to_string(), self.config.colors.selection_bg.clone()),
                ("Directory Color".to_string(), self.config.colors.directory_fg.clone()),
                ("Executable Color".to_string(), self.config.colors.executable_fg.clone()),
                ("Symlink Color".to_string(), self.config.colors.symlink_fg.clone()),
            ],
            ConfigTab::Plugins => vec![
                ("Enable Plugins".to_string(), self.config.plugins.enable_plugins.to_string()),
                ("Auto Reload Plugins".to_string(), self.config.plugins.auto_reload_plugins.to_string()),
                ("Allow Unsafe Plugins".to_string(), self.config.plugins.allow_unsafe_plugins.to_string()),
                ("Plugin Directory".to_string(), self.config.general.plugin_directory.clone()),
            ],
            ConfigTab::Network => vec![
                ("Connection Timeout (s)".to_string(), self.config.network.connection_timeout.to_string()),
                ("Save Credentials".to_string(), self.config.network.save_credentials.to_string()),
                ("Verify SSL".to_string(), self.config.network.verify_ssl.to_string()),
                ("Enable Compression".to_string(), self.config.network.enable_compression.to_string()),
                ("Known Hosts".to_string(), self.config.network.known_hosts.join(", ")),
            ],
            ConfigTab::Keybindings => vec![
                ("Custom Keybindings".to_string(), format!("{} defined", self.config.keybindings.custom.len())),
            ],
        }
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