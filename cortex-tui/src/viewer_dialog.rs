use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::viewer::FileViewer;

#[derive(Debug, Clone)]
pub struct ViewerDialog {
    pub viewer: FileViewer,
    pub search_mode: bool,
    pub search_input: String,
}

impl ViewerDialog {
    pub fn new(viewer: FileViewer) -> Self {
        Self {
            viewer,
            search_mode: false,
            search_input: String::new(),
        }
    }
    
    pub fn render(&mut self, frame: &mut Frame) {
        let area = centered_rect(90, 90, frame.area());
        frame.render_widget(Clear, area);
        
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(5),     // Content
                Constraint::Length(2),  // Status
                Constraint::Length(2),  // Help
            ])
            .split(area);
        
        // Title
        let title = format!(" Viewing: {} ", self.viewer.title);
        let title_block = Block::default()
            .title(title)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(title_block, chunks[0]);
        
        // Content area
        let content_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan));
        
        let inner = content_block.inner(chunks[1]);
        frame.render_widget(content_block, chunks[1]);
        
        // Render file content
        let visible_lines: Vec<Line> = self.viewer.lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let mut style = Style::default();
                
                // Highlight search matches
                if let Some(ref term) = self.viewer.search_term {
                    if line.to_lowercase().contains(&term.to_lowercase()) {
                        style = style.bg(Color::Yellow).fg(Color::Black);
                    }
                }
                
                // Highlight selected line
                if i == self.viewer.selected_line {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                
                // Apply syntax highlighting for common patterns
                if !self.viewer.hex_mode {
                    Line::from(self.apply_syntax_highlighting(line))
                } else {
                    Line::from(vec![Span::styled(line.clone(), style)])
                }
            })
            .collect();
        
        let content = Paragraph::new(visible_lines)
            .wrap(Wrap { trim: false });
        frame.render_widget(content, inner);
        
        // Status bar
        let status = self.viewer.get_status();
        let status_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan));
        let status_inner = status_block.inner(chunks[2]);
        frame.render_widget(status_block, chunks[2]);
        let status_text = Paragraph::new(status)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(status_text, status_inner);
        
        // Help line or search input
        if self.search_mode {
            let search_block = Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Search: ");
            let search_inner = search_block.inner(chunks[3]);
            frame.render_widget(search_block, chunks[3]);
            let search_text = Paragraph::new(self.search_input.as_str())
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(search_text, search_inner);
            
            // Show cursor
            frame.set_cursor_position((
                search_inner.x + self.search_input.len() as u16,
                search_inner.y,
            ));
        } else {
            let help_text = if self.viewer.hex_mode {
                " ESC/F3: Exit | H: Toggle Hex | /: Search | F: Search Next | ↑↓: Scroll | PgUp/PgDn: Page "
            } else {
                " ESC/F3: Exit | W: Toggle Wrap | /: Search | F: Find Next | ↑↓: Scroll | PgUp/PgDn: Page "
            };
            
            let help_block = Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan));
            let help_inner = help_block.inner(chunks[3]);
            frame.render_widget(help_block, chunks[3]);
            let help = Paragraph::new(help_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(help, help_inner);
        }
    }
    
    fn apply_syntax_highlighting(&self, line: &str) -> Vec<Span> {
        let mut spans = Vec::new();
        
        // Simple syntax highlighting for common patterns
        if line.trim_start().starts_with("//") || line.trim_start().starts_with("#") {
            // Comments
            spans.push(Span::styled(line.to_string(), 
                Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)));
        } else if line.contains("fn ") || line.contains("def ") || 
                  line.contains("function ") || line.contains("class ") {
            // Function/class definitions
            spans.push(Span::styled(line.to_string(), 
                Style::default().fg(Color::Yellow)));
        } else if line.contains("use ") || line.contains("import ") || 
                  line.contains("include ") || line.contains("require ") {
            // Imports
            spans.push(Span::styled(line.to_string(), 
                Style::default().fg(Color::Magenta)));
        } else if line.contains("pub ") || line.contains("const ") || 
                  line.contains("let ") || line.contains("var ") {
            // Keywords
            spans.push(Span::styled(line.to_string(), 
                Style::default().fg(Color::Cyan)));
        } else {
            spans.push(Span::raw(line.to_string()));
        }
        
        spans
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