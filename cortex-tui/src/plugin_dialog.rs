use cortex_core::PluginInfo;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct PluginDialog {
    pub plugins: Vec<PluginInfo>,
    pub plugin_states: Vec<bool>, // Track enabled/disabled state
    pub selected_index: usize,
    pub show_details: bool,
}

impl PluginDialog {
    pub fn new(plugins: Vec<PluginInfo>) -> Self {
        let plugin_states = vec![true; plugins.len()]; // Default all enabled
        Self {
            plugins,
            plugin_states,
            selected_index: 0,
            show_details: false,
        }
    }

    pub fn with_states(plugins: Vec<PluginInfo>, states: Vec<bool>) -> Self {
        Self {
            plugins,
            plugin_states: states,
            selected_index: 0,
            show_details: false,
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.plugins.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    pub fn get_selected_plugin(&self) -> Option<&PluginInfo> {
        self.plugins.get(self.selected_index)
    }

    pub fn toggle_selected_plugin(&mut self) -> Option<bool> {
        if let Some(state) = self.plugin_states.get_mut(self.selected_index) {
            *state = !*state;
            Some(*state)
        } else {
            None
        }
    }

    pub fn is_selected_plugin_enabled(&self) -> bool {
        self.plugin_states
            .get(self.selected_index)
            .copied()
            .unwrap_or(true)
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = centered_rect(80, 70, frame.area());
        frame.render_widget(Clear, area);

        if self.show_details && !self.plugins.is_empty() {
            self.render_plugin_details(frame, area);
        } else {
            self.render_plugin_list(frame, area);
        }
    }

    fn render_plugin_list(&self, frame: &mut Frame, area: Rect) {
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Plugin list
                Constraint::Length(3), // Help
            ])
            .split(area);

        // Title
        let title_block = Block::default()
            .title(" Plugin Manager ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let title_inner = title_block.inner(chunks[0]);
        frame.render_widget(title_block, chunks[0]);

        let title_text = format!("Loaded Plugins: {}", self.plugins.len());
        let title_para = Paragraph::new(title_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(title_para, title_inner);

        // Plugin list
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let list_inner = list_block.inner(chunks[1]);
        frame.render_widget(list_block, chunks[1]);

        if self.plugins.is_empty() {
            let no_plugins = Paragraph::new("No plugins loaded")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(no_plugins, list_inner);
        } else {
            let items: Vec<ListItem> = self
                .plugins
                .iter()
                .enumerate()
                .map(|(idx, plugin)| {
                    let is_selected = idx == self.selected_index;
                    let style = if is_selected {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };

                    let is_enabled = self.plugin_states.get(idx).copied().unwrap_or(true);
                    let (status_indicator, status_color) = if is_enabled {
                        ("✓", Color::Green)
                    } else {
                        ("✗", Color::Red)
                    };

                    let spans = vec![
                        Span::styled("  ", style),
                        Span::styled(status_indicator, style.fg(status_color)),
                        Span::styled("  ", style),
                        Span::styled(
                            format!("{:<20}", plugin.name),
                            style.fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("  ", style),
                        Span::styled(format!("v{:<8}", plugin.version), style.fg(Color::Cyan)),
                        Span::styled("  ", style),
                        Span::styled(
                            format!("{:<30}", plugin.description),
                            style.fg(Color::White),
                        ),
                        Span::styled("  ", style),
                        Span::styled(plugin.author.clone(), style.fg(Color::DarkGray)),
                    ];

                    ListItem::new(Line::from(spans))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, list_inner);
        }

        // Help
        let help_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let help_inner = help_block.inner(chunks[2]);
        frame.render_widget(help_block, chunks[2]);

        let help_text = " ↑↓: Navigate | Enter: Details | R: Reload | Space: Toggle | ESC: Close ";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_inner);
    }

    fn render_plugin_details(&self, frame: &mut Frame, area: Rect) {
        if let Some(plugin) = self.get_selected_plugin() {
            // Main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(5),    // Details
                    Constraint::Length(3), // Help
                ])
                .split(area);

            // Title
            let title_block = Block::default()
                .title(format!(" Plugin Details: {} ", plugin.name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            frame.render_widget(title_block, chunks[0]);

            // Details
            let details_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            let details_inner = details_block.inner(chunks[1]);
            frame.render_widget(details_block, chunks[1]);

            let mut details_text = vec![
                Line::from(vec![
                    Span::styled(
                        "Name: ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(plugin.name.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Version: ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(plugin.version.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Author: ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(plugin.author.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Description: ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        plugin.description.clone(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Min Cortex Version: ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        plugin.min_cortex_version.clone(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
            ];

            if !plugin.commands.is_empty() {
                details_text.push(Line::from(vec![Span::styled(
                    "Commands: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]));
                for command in &plugin.commands {
                    details_text.push(Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                        Span::styled(command.clone(), Style::default().fg(Color::Green)),
                    ]));
                }
                details_text.push(Line::from(""));
            }

            if !plugin.event_hooks.is_empty() {
                details_text.push(Line::from(vec![Span::styled(
                    "Event Hooks: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]));
                for hook in &plugin.event_hooks {
                    details_text.push(Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                        Span::styled(hook.clone(), Style::default().fg(Color::Yellow)),
                    ]));
                }
            }

            let details_para = Paragraph::new(details_text)
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(details_para, details_inner);

            // Help
            let help_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            let help_inner = help_block.inner(chunks[2]);
            frame.render_widget(help_block, chunks[2]);

            let help_text = " ESC/Backspace: Back to list | Space: Toggle Plugin | R: Reload ";
            let help = Paragraph::new(help_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(help, help_inner);
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
