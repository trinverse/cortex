use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AIProvider {
    Groq,
    Gemini,
    Anthropic,
    OpenAI,
}

impl AIProvider {
    pub fn as_str(&self) -> &str {
        match self {
            AIProvider::Groq => "Groq",
            AIProvider::Gemini => "Gemini",
            AIProvider::Anthropic => "Anthropic",
            AIProvider::OpenAI => "OpenAI",
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => AIProvider::Groq,
            1 => AIProvider::Gemini,
            2 => AIProvider::Anthropic,
            3 => AIProvider::OpenAI,
            _ => AIProvider::Groq,
        }
    }

    pub fn all() -> Vec<AIProvider> {
        vec![
            AIProvider::Groq,
            AIProvider::Gemini,
            AIProvider::Anthropic,
            AIProvider::OpenAI,
        ]
    }

    pub fn config_key(&self) -> &str {
        match self {
            AIProvider::Groq => "groq_api_key",
            AIProvider::Gemini => "gemini_api_key",
            AIProvider::Anthropic => "anthropic_api_key",
            AIProvider::OpenAI => "openai_api_key",
        }
    }

    pub fn env_var(&self) -> &str {
        match self {
            AIProvider::Groq => "GROQ_API_KEY",
            AIProvider::Gemini => "GEMINI_API_KEY",
            AIProvider::Anthropic => "ANTHROPIC_API_KEY",
            AIProvider::OpenAI => "OPENAI_API_KEY",
        }
    }
}

#[derive(Debug, Clone)]
pub struct APIKeyDialog {
    pub selected_provider: AIProvider,
    pub api_key: String,
    pub cursor_position: usize,
    pub input_mode: bool,
    pub provider_dropdown_open: bool,
    pub selected_provider_index: usize,
    pub show_key: bool,
    pub saved_keys: HashMap<String, bool>, // provider -> has_key
}

impl APIKeyDialog {
    pub fn new() -> Self {
        let mut saved_keys = HashMap::new();
        
        // Check which providers already have keys configured
        for provider in AIProvider::all() {
            let has_key = std::env::var(provider.env_var()).is_ok();
            saved_keys.insert(provider.as_str().to_string(), has_key);
        }

        Self {
            selected_provider: AIProvider::Groq,
            api_key: String::new(),
            cursor_position: 0,
            input_mode: false,
            provider_dropdown_open: false,
            selected_provider_index: 0,
            show_key: false,
            saved_keys,
        }
    }

    pub fn toggle_dropdown(&mut self) {
        self.provider_dropdown_open = !self.provider_dropdown_open;
        if !self.provider_dropdown_open {
            self.selected_provider = AIProvider::from_index(self.selected_provider_index);
        }
    }

    pub fn next_provider(&mut self) {
        if self.provider_dropdown_open {
            self.selected_provider_index = (self.selected_provider_index + 1) % 4;
        }
    }

    pub fn prev_provider(&mut self) {
        if self.provider_dropdown_open {
            if self.selected_provider_index == 0 {
                self.selected_provider_index = 3;
            } else {
                self.selected_provider_index -= 1;
            }
        }
    }

    pub fn toggle_input_mode(&mut self) {
        self.input_mode = !self.input_mode;
        if self.input_mode {
            self.cursor_position = self.api_key.len();
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.input_mode {
            self.api_key.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if self.input_mode && self.cursor_position > 0 {
            self.api_key.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.input_mode && self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.input_mode && self.cursor_position < self.api_key.len() {
            self.cursor_position += 1;
        }
    }

    pub fn toggle_show_key(&mut self) {
        self.show_key = !self.show_key;
    }

    pub fn get_masked_key(&self) -> String {
        if self.show_key || self.api_key.is_empty() {
            self.api_key.clone()
        } else {
            "*".repeat(self.api_key.len())
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Clear the background
        f.render_widget(Clear, area);

        // Calculate dialog size
        let dialog_width = 60;
        let dialog_height = 14;
        
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Main dialog block
        let block = Block::default()
            .title(" API Key Configuration ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        
        f.render_widget(block, dialog_area);

        // Inner layout
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2),  // Info text
                Constraint::Length(1),  // Spacing
                Constraint::Length(3),  // Provider selection
                Constraint::Length(3),  // API Key input
                Constraint::Length(2),  // Help text
                Constraint::Min(0),     // Remaining space
            ])
            .split(dialog_area);

        // Info text
        let info_text = vec![
            Line::from("Configure your AI provider API key"),
            Line::from(vec![
                Span::styled("Warning: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Never commit API keys to repositories"),
            ]),
        ];
        let info = Paragraph::new(info_text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(info, inner[0]);

        // Provider selection
        let provider_label = if self.provider_dropdown_open {
            format!("Provider: [{}] ▼", AIProvider::from_index(self.selected_provider_index).as_str())
        } else {
            format!("Provider: [{}] ▶", self.selected_provider.as_str())
        };
        
        let provider_style = if self.provider_dropdown_open {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        
        let provider_widget = Paragraph::new(provider_label)
            .style(provider_style)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(provider_widget, inner[2]);

        // Show dropdown if open
        if self.provider_dropdown_open {
            let dropdown_area = Rect {
                x: inner[2].x + 10,
                y: inner[2].y + 1,
                width: 20,
                height: 5,
            };
            
            f.render_widget(Clear, dropdown_area);
            
            let dropdown_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray));
            
            let dropdown_inner = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(dropdown_area);
            
            f.render_widget(dropdown_block, dropdown_area);
            
            for (i, provider) in AIProvider::all().iter().enumerate() {
                let style = if i == self.selected_provider_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let has_key = self.saved_keys.get(provider.as_str()).unwrap_or(&false);
                let indicator = if *has_key { " ✓" } else { "" };
                let text = format!(" {}{}", provider.as_str(), indicator);
                
                let item = Paragraph::new(text).style(style);
                if i < 4 {
                    f.render_widget(item, dropdown_inner[i]);
                }
            }
        }

        // API Key input
        let key_display = self.get_masked_key();
        let key_label = format!("API Key: {}", if self.input_mode { 
            format!("{}│", &key_display[..self.cursor_position.min(key_display.len())])
        } else {
            key_display
        });
        
        let key_style = if self.input_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        
        let key_widget = Paragraph::new(key_label)
            .style(key_style)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(if self.input_mode { 
                    Style::default().fg(Color::Yellow) 
                } else { 
                    Style::default().fg(Color::Gray) 
                }));
        f.render_widget(key_widget, inner[3]);

        // Help text
        let help_lines = if self.input_mode {
            vec![
                Line::from(vec![
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Save  "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel  "),
                    Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
                    Span::raw(" Show/Hide"),
                ]),
            ]
        } else if self.provider_dropdown_open {
            vec![
                Line::from(vec![
                    Span::styled("[↑↓]", Style::default().fg(Color::Green)),
                    Span::raw(" Navigate  "),
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Select  "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Close"),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled("[Tab]", Style::default().fg(Color::Green)),
                    Span::raw(" Provider  "),
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Edit Key  "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ]),
            ]
        };
        
        let help = Paragraph::new(help_lines)
            .alignment(Alignment::Center);
        f.render_widget(help, inner[4]);
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}