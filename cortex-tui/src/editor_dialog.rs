use crate::editor::TextEditor;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct EditorDialog {
    pub editor: TextEditor,
    pub search_mode: bool,
    pub search_input: String,
    pub replace_mode: bool,
    pub replace_input: String,
}

impl EditorDialog {
    pub fn new(editor: TextEditor) -> Self {
        Self {
            editor,
            search_mode: false,
            search_input: String::new(),
            replace_mode: false,
            replace_input: String::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = centered_rect(95, 95, frame.size());
        frame.render_widget(Clear, area);

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Content
                Constraint::Length(2), // Status
                Constraint::Length(2), // Help/Input
            ])
            .split(area);

        // Title bar
        let title = format!(
            " Editing: {} {} ",
            self.editor.title,
            if self.editor.modified {
                "[Modified]"
            } else {
                ""
            }
        );
        let title_block = Block::default()
            .title(title)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Green));
        frame.render_widget(title_block, chunks[0]);

        // Content area
        let content_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Green));

        let inner = content_block.inner(chunks[1]);
        frame.render_widget(content_block, chunks[1]);

        // Calculate visible area
        let height = inner.height as usize;
        let width = inner.width as usize;
        self.editor.update_view_offset(height, width);

        // Render visible lines with line numbers
        let mut lines_to_render = Vec::new();
        let line_number_width = self.editor.lines.len().to_string().len() + 1;

        for i in 0..height {
            let line_idx = self.editor.offset_row + i;
            if line_idx >= self.editor.lines.len() {
                break;
            }

            let line = &self.editor.lines[line_idx];
            let line_number = format!("{:>width$} ", line_idx + 1, width = line_number_width - 1);

            // Apply syntax highlighting
            let mut spans = vec![Span::styled(
                line_number,
                Style::default().fg(Color::DarkGray),
            )];

            // Check for search highlights
            if let Some(ref term) = self.editor.search_term {
                if let Some(pos) = line.to_lowercase().find(&term.to_lowercase()) {
                    // Split line into before match, match, and after match
                    let before = &line[..pos];
                    let matched = &line[pos..pos + term.len()];
                    let after = &line[pos + term.len()..];

                    spans.push(Span::raw(before.to_string()));
                    spans.push(Span::styled(
                        matched,
                        Style::default().bg(Color::Yellow).fg(Color::Black),
                    ));
                    spans.push(Span::raw(after.to_string()));
                } else {
                    spans.extend(self.apply_syntax_highlighting(line));
                }
            } else {
                spans.extend(self.apply_syntax_highlighting(line));
            }

            lines_to_render.push(Line::from(spans));
        }

        let content = Paragraph::new(lines_to_render);
        frame.render_widget(content, inner);

        // Render cursor
        if !self.search_mode && !self.replace_mode {
            let cursor_x = inner.x
                + line_number_width as u16
                + (self.editor.cursor_col - self.editor.offset_col) as u16;
            let cursor_y = inner.y + (self.editor.cursor_row - self.editor.offset_row) as u16;

            if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }

        // Status bar
        let status = self.editor.get_status();
        let status_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Green));
        let status_inner = status_block.inner(chunks[2]);
        frame.render_widget(status_block, chunks[2]);
        let status_text = Paragraph::new(status)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(status_text, status_inner);

        // Help line or search/replace input
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
            frame.set_cursor(
                search_inner.x + self.search_input.len() as u16,
                search_inner.y,
            );
        } else if self.replace_mode {
            let replace_block = Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Replace with: ");
            let replace_inner = replace_block.inner(chunks[3]);
            frame.render_widget(replace_block, chunks[3]);
            let replace_text = Paragraph::new(self.replace_input.as_str())
                .style(Style::default().fg(Color::Magenta));
            frame.render_widget(replace_text, replace_inner);

            // Show cursor
            frame.set_cursor(
                replace_inner.x + self.replace_input.len() as u16,
                replace_inner.y,
            );
        } else {
            let help_text = " ESC/F4: Exit | Ctrl+S: Save | Ctrl+F: Find | Ctrl+R: Replace | Ctrl+Z: Undo | Ctrl+Y: Redo ";

            let help_block = Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Green));
            let help_inner = help_block.inner(chunks[3]);
            frame.render_widget(help_block, chunks[3]);
            let help = Paragraph::new(help_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(help, help_inner);
        }

        // Show status message if any
        if let Some(ref msg) = self.editor.status_message {
            let msg_area = Rect {
                x: area.x + area.width / 2 - 20,
                y: area.y + area.height / 2,
                width: 40.min(area.width),
                height: 3,
            };
            frame.render_widget(Clear, msg_area);
            let msg_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let msg_inner = msg_block.inner(msg_area);
            frame.render_widget(msg_block, msg_area);
            let msg_text = Paragraph::new(msg.as_str()).alignment(Alignment::Center);
            frame.render_widget(msg_text, msg_inner);
        }
    }

    fn apply_syntax_highlighting(&self, line: &str) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Simple syntax highlighting based on file extension
        let ext = self
            .editor
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "rs" => {
                // Rust syntax highlighting
                if line.trim_start().starts_with("//") {
                    spans.push(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::ITALIC),
                    ));
                } else if line.contains("fn ")
                    || line.contains("impl ")
                    || line.contains("struct ")
                    || line.contains("enum ")
                {
                    spans.push(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Yellow),
                    ));
                } else if line.contains("use ") || line.contains("mod ") {
                    spans.push(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Magenta),
                    ));
                } else if line.contains("pub ")
                    || line.contains("let ")
                    || line.contains("const ")
                    || line.contains("mut ")
                {
                    spans.push(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Cyan),
                    ));
                } else {
                    spans.push(Span::raw(line.to_string()));
                }
            }
            "py" => {
                // Python syntax highlighting
                if line.trim_start().starts_with("#") {
                    spans.push(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::ITALIC),
                    ));
                } else if line.contains("def ") || line.contains("class ") {
                    spans.push(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Yellow),
                    ));
                } else if line.contains("import ") || line.contains("from ") {
                    spans.push(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Magenta),
                    ));
                } else {
                    spans.push(Span::raw(line.to_string()));
                }
            }
            _ => {
                // Default - no highlighting
                spans.push(Span::raw(line.to_string()));
            }
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
