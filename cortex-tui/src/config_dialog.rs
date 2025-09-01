use cortex_core::Config;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigTab {
    General,
    Panels,
    Colors,
    Themes,
    AI,
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
            Self::Themes,
            Self::AI,
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
            Self::Themes => "Themes",
            Self::AI => "AI",
            Self::Plugins => "Plugins",
            Self::Network => "Network",
            Self::Keybindings => "Keys",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigDialog {
    pub config: Config,
    pub dirty: bool,
    pub current_tab: ConfigTab,
    pub selected_index: usize,
    pub editing: bool,
    pub edit_value: String,
    pub edit_cursor: usize,
    pub available_themes: Vec<(String, String)>, // (id, name) pairs
    pub current_theme: String,
    pub ai_api_key: String,
    pub ai_provider: String,
    pub available_providers: Vec<String>,
}

impl ConfigDialog {
    pub fn new(config: Config, theme_manager: &cortex_core::ThemeManager) -> Self {
        let ai_provider = config.ai.default_provider.clone();
        let available_themes = theme_manager
            .available_themes()
            .iter()
            .map(|t| (t.mode.name().to_lowercase(), t.mode.name().to_string()))
            .collect();
        let current_theme = theme_manager.get_current_theme().mode.name().to_lowercase();

        Self {
            config,
            dirty: false,
            current_tab: ConfigTab::General,
            selected_index: 0,
            editing: false,
            edit_value: String::new(),
            edit_cursor: 0,
            available_themes,
            current_theme,
            ai_api_key: String::new(),
            ai_provider,
            available_providers: vec![
                "groq".to_string(),
                "openai".to_string(),
                "claude".to_string(),
                "ollama".to_string(),
            ],
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = ConfigTab::all();
        let current_idx = tabs
            .iter()
            .position(|&t| t == self.current_tab)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % tabs.len();
        self.current_tab = tabs[next_idx];
        self.selected_index = 0;
        self.cancel_edit();
    }

    pub fn prev_tab(&mut self) {
        let tabs = ConfigTab::all();
        let current_idx = tabs
            .iter()
            .position(|&t| t == self.current_tab)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            tabs.len() - 1
        } else {
            current_idx - 1
        };
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
            ConfigTab::Themes => 1,
            ConfigTab::AI => 2,
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
                5 => self.config.general.terminal.clone(),
                6 => self.config.general.editor.clone(),
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
            ConfigTab::Themes => match self.selected_index {
                0 => self.current_theme.clone(),
                _ => String::new(),
            },
            ConfigTab::AI => match self.selected_index {
                0 => self.ai_provider.clone(),
                1 => self.ai_api_key.clone(),
                _ => String::new(),
            },
            ConfigTab::Keybindings => "Custom keybindings (not yet editable)".to_string(),
        }
    }

    pub fn set_current_value(&mut self, value: &str) {
        self.dirty = true;
        match self.current_tab {
            ConfigTab::General => match self.selected_index {
                0 => self.config.general.show_hidden = value.parse().unwrap_or(false),
                1 => self.config.general.confirm_delete = value.parse().unwrap_or(true),
                2 => self.config.general.show_icons = value.parse().unwrap_or(false),
                3 => self.config.general.auto_reload = value.parse().unwrap_or(true),
                4 => self.config.general.confirm_operations = value.parse().unwrap_or(true),
                5 => self.config.general.terminal = value.to_string(),
                6 => self.config.general.editor = value.to_string(),
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
                4 => {
                    self.config.network.known_hosts = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                _ => {}
            },
            ConfigTab::Themes => match self.selected_index {
                0 => {
                    self.current_theme = value.to_string();
                    self.config.general.theme = value.to_string();
                }
                _ => {}
            },
            ConfigTab::AI => match self.selected_index {
                0 => self.ai_provider = value.to_string(),
                1 => self.ai_api_key = value.to_string(),
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

    pub fn render(&self, frame: &mut Frame, theme: &cortex_core::Theme) {
        let area = centered_rect(85, 80, frame.size());
        frame.render_widget(Clear, area);

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(5),    // Content
                Constraint::Length(3), // Help
            ])
            .split(area);

        // Tabs
        let tabs_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.active_border));
        let tabs_inner = tabs_block.inner(chunks[0]);
        frame.render_widget(tabs_block, chunks[0]);

        let tab_titles: Vec<_> = ConfigTab::all().iter().map(|t| t.title()).collect();
        
        // Apply same explicit color fix for tabs as we did for list items
        let tab_highlight_style = if theme.mode == cortex_core::ThemeMode::Light {
            Style::default().bg(ratatui::style::Color::Rgb(210, 227, 252)).fg(ratatui::style::Color::Rgb(24, 28, 33))
        } else {
            Style::default().bg(theme.selected_bg).fg(theme.selected_fg)
        };
        
        let tabs = Tabs::new(tab_titles)
            .block(Block::default())
            .highlight_style(tab_highlight_style)
            .select(
                ConfigTab::all()
                    .iter()
                    .position(|&t| t == self.current_tab)
                    .unwrap_or(0),
            );
        frame.render_widget(tabs, tabs_inner);

        // Content
        let content_block = Block::default()
            .title(format!(" {} Settings ", self.current_tab.title()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.active_border));
        let content_inner = content_block.inner(chunks[1]);
        frame.render_widget(content_block, chunks[1]);

        self.render_tab_content(frame, content_inner, theme);

        // Help
        let help_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.active_border));
        let help_inner = help_block.inner(chunks[2]);
        frame.render_widget(help_block, chunks[2]);

        let help_text = if self.editing {
            " Enter: Save | ESC: Cancel | ←→: Move cursor | Backspace: Delete "
        } else {
            " ↑↓: Navigate | ←→: Switch tabs | Enter: Edit | S: Save config | ESC: Close "
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme.dim_text))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_inner);
    }

    fn render_tab_content(&self, frame: &mut Frame, area: Rect, theme: &cortex_core::Theme) {
        let items = self.get_tab_items();

        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(idx, (name, value))| {
                let is_selected = idx == self.selected_index;

                let display_value = if is_selected && self.editing {
                    &self.edit_value
                } else {
                    value
                };

                // Force explicit colors to avoid any inheritance issues
                use ratatui::style::Color;
                
                let (bg_color, fg_color) = if is_selected {
                    // Force light theme colors if selected
                    if theme.mode == cortex_core::ThemeMode::Light {
                        (Color::Rgb(210, 227, 252), Color::Rgb(24, 28, 33)) // Light blue bg, dark text
                    } else {
                        (theme.selected_bg, theme.selected_fg)
                    }
                } else {
                    (Color::Reset, theme.normal_text)
                };

                let base_style = if is_selected {
                    Style::default().bg(bg_color).fg(fg_color)
                } else {
                    Style::default().fg(theme.normal_text)
                };

                let name_style = if is_selected {
                    Style::default().bg(bg_color).fg(fg_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.highlight_text).add_modifier(Modifier::BOLD)
                };

                let value_style = if is_selected && self.editing {
                    Style::default().bg(bg_color).fg(theme.warning)
                } else if is_selected {
                    Style::default().bg(bg_color).fg(fg_color)
                } else {
                    Style::default().fg(theme.normal_text)
                };

                let spans = vec![
                    Span::styled("  ", base_style),
                    Span::styled(format!("{:<25}", name), name_style),
                    Span::styled("  ", base_style),
                    Span::styled(display_value.clone(), value_style),
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
                frame.set_cursor(cursor_x, cursor_y);
            }
        }
    }

    fn get_tab_items(&self) -> Vec<(String, String)> {
        match self.current_tab {
            ConfigTab::General => vec![
                (
                    "Show Hidden Files".to_string(),
                    self.config.general.show_hidden.to_string(),
                ),
                (
                    "Confirm Delete".to_string(),
                    self.config.general.confirm_delete.to_string(),
                ),
                (
                    "Show Icons".to_string(),
                    self.config.general.show_icons.to_string(),
                ),
                (
                    "Auto Reload".to_string(),
                    self.config.general.auto_reload.to_string(),
                ),
                (
                    "Confirm Operations".to_string(),
                    self.config.general.confirm_operations.to_string(),
                ),
                (
                    "Default Terminal".to_string(),
                    self.config.general.terminal.clone(),
                ),
                (
                    "Default Editor".to_string(),
                    self.config.general.editor.clone(),
                ),
            ],
            ConfigTab::Panels => vec![
                (
                    "Default Sort Order".to_string(),
                    self.config.panels.default_sort.clone(),
                ),
                (
                    "Show File Size".to_string(),
                    self.config.panels.show_size.to_string(),
                ),
                (
                    "Show Permissions".to_string(),
                    self.config.panels.show_permissions.to_string(),
                ),
                (
                    "Show Modified Date".to_string(),
                    self.config.panels.show_modified.to_string(),
                ),
            ],
            ConfigTab::Colors => vec![
                (
                    "Selection Background".to_string(),
                    self.config.colors.selection_bg.clone(),
                ),
                (
                    "Directory Color".to_string(),
                    self.config.colors.directory_fg.clone(),
                ),
                (
                    "Executable Color".to_string(),
                    self.config.colors.executable_fg.clone(),
                ),
                (
                    "Symlink Color".to_string(),
                    self.config.colors.symlink_fg.clone(),
                ),
            ],
            ConfigTab::Plugins => vec![
                (
                    "Enable Plugins".to_string(),
                    self.config.plugins.enable_plugins.to_string(),
                ),
                (
                    "Auto Reload Plugins".to_string(),
                    self.config.plugins.auto_reload_plugins.to_string(),
                ),
                (
                    "Allow Unsafe Plugins".to_string(),
                    self.config.plugins.allow_unsafe_plugins.to_string(),
                ),
                (
                    "Plugin Directory".to_string(),
                    self.config.general.plugin_directory.clone(),
                ),
            ],
            ConfigTab::Network => vec![
                (
                    "Connection Timeout (s)".to_string(),
                    self.config.network.connection_timeout.to_string(),
                ),
                (
                    "Save Credentials".to_string(),
                    self.config.network.save_credentials.to_string(),
                ),
                (
                    "Verify SSL".to_string(),
                    self.config.network.verify_ssl.to_string(),
                ),
                (
                    "Enable Compression".to_string(),
                    self.config.network.enable_compression.to_string(),
                ),
                (
                    "Known Hosts".to_string(),
                    self.config.network.known_hosts.join(", "),
                ),
            ],
            ConfigTab::Themes => vec![
                (
                    "Current Theme".to_string(),
                    self.current_theme.clone(),
                ),
            ],
            ConfigTab::AI => vec![
                (
                    "AI Provider".to_string(),
                    self.ai_provider.clone(),
                ),
                (
                    "API Key".to_string(),
                    if self.ai_api_key.is_empty() { 
                        "Not set".to_string() 
                    } else { 
                        "*".repeat(self.ai_api_key.len().min(8)) 
                    },
                ),
            ],
            ConfigTab::Keybindings => vec![(
                "Custom Keybindings".to_string(),
                format!("{} defined", self.config.keybindings.custom.len()),
            )],
        }
    }

    pub fn cycle_theme_forward(&mut self) {
        self.dirty = true;
        let current_idx = self.available_themes
            .iter()
            .position(|(id, _)| id == &self.current_theme)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.available_themes.len();
        self.current_theme = self.available_themes[next_idx].0.clone();
        self.config.general.theme = self.current_theme.clone();
        self.edit_value = self.current_theme.clone();
    }

    pub fn cycle_theme_backward(&mut self) {
        self.dirty = true;
        let current_idx = self.available_themes
            .iter()
            .position(|(id, _)| id == &self.current_theme)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.available_themes.len() - 1
        } else {
            current_idx - 1
        };
        self.current_theme = self.available_themes[prev_idx].0.clone();
        self.config.general.theme = self.current_theme.clone();
        self.edit_value = self.current_theme.clone();
    }

    pub fn cycle_provider_forward(&mut self) {
        self.dirty = true;
        let current_idx = self.available_providers
            .iter()
            .position(|p| p == &self.ai_provider)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.available_providers.len();
        self.ai_provider = self.available_providers[next_idx].clone();
        self.config.ai.default_provider = self.ai_provider.clone();
        self.edit_value = self.ai_provider.clone();
    }

    pub fn cycle_provider_backward(&mut self) {
        self.dirty = true;
        let current_idx = self.available_providers
            .iter()
            .position(|p| p == &self.ai_provider)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.available_providers.len() - 1
        } else {
            current_idx - 1
        };
        self.ai_provider = self.available_providers[prev_idx].clone();
        self.config.ai.default_provider = self.ai_provider.clone();
        self.edit_value = self.ai_provider.clone();
    }

    pub fn is_dropdown_field(&self) -> bool {
        match self.current_tab {
            ConfigTab::Themes => self.selected_index == 0,
            ConfigTab::AI => self.selected_index == 0,
            _ => false,
        }
    }

    pub fn is_boolean_field(&self) -> bool {
        match self.current_tab {
            ConfigTab::General => matches!(self.selected_index, 0 | 1 | 2 | 3 | 4),
            ConfigTab::Panels => matches!(self.selected_index, 1 | 2 | 3),
            ConfigTab::Plugins => matches!(self.selected_index, 0 | 1 | 2),
            ConfigTab::Network => matches!(self.selected_index, 1 | 2 | 3),
            _ => false,
        }
    }

    pub fn toggle_current_boolean_value(&mut self) {
        let current_value_str = self.get_current_value();
        let current_value = current_value_str.parse::<bool>().unwrap_or(false);
        self.set_current_value(&(!current_value).to_string());
        self.edit_value = (!current_value).to_string();
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
