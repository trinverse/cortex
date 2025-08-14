use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone)]
pub struct AIChatDialog {
    pub messages: Vec<Message>,
    pub input: String,
    pub cursor_position: usize,
    pub scroll_position: usize,
    pub streaming_response: Option<String>,
    pub is_processing: bool,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl AIChatDialog {
    pub fn new() -> Self {
        Self {
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: "AI Assistant ready. Type your file management commands or questions.".to_string(),
                    timestamp: chrono::Local::now(),
                }
            ],
            input: String::new(),
            cursor_position: 0,
            scroll_position: 0,
            streaming_response: None,
            is_processing: false,
        }
    }
    
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }
    
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }
    
    pub fn submit_message(&mut self) -> Option<String> {
        if !self.input.is_empty() && !self.is_processing {
            let message = self.input.clone();
            self.messages.push(Message {
                role: MessageRole::User,
                content: message.clone(),
                timestamp: chrono::Local::now(),
            });
            self.input.clear();
            self.cursor_position = 0;
            self.is_processing = true;
            self.scroll_to_bottom(); // Auto-scroll when user sends a message
            Some(message)
        } else {
            None
        }
    }
    
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content,
            timestamp: chrono::Local::now(),
        });
        self.is_processing = false;
        self.streaming_response = None;
        self.scroll_to_bottom();
    }
    
    pub fn update_streaming_response(&mut self, chunk: String) {
        if let Some(ref mut response) = self.streaming_response {
            response.push_str(&chunk);
        } else {
            self.streaming_response = Some(chunk);
        }
        self.scroll_to_bottom();
    }
    
    pub fn finalize_streaming_response(&mut self) {
        if let Some(response) = self.streaming_response.take() {
            self.add_assistant_message(response);
        }
    }
    
    pub fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
        }
    }
    
    pub fn scroll_down(&mut self) {
        self.scroll_position += 1;
    }
    
    pub fn scroll_to_bottom(&mut self) {
        // Calculate total lines needed for all messages
        let total_lines = self.calculate_total_lines();
        // We want to scroll to show the bottom of the content
        // Assuming viewport is about 20-30 lines, we leave a small margin
        self.scroll_position = total_lines.saturating_sub(5);
    }
    
    pub fn scroll_to_bottom_for_viewport(&mut self, viewport_height: usize) {
        // More accurate scrolling when we know the viewport height
        let total_lines = self.calculate_total_lines();
        if total_lines > viewport_height {
            self.scroll_position = total_lines - viewport_height + 2; // Small margin
        } else {
            self.scroll_position = 0;
        }
    }
    
    fn calculate_total_lines(&self) -> usize {
        let mut lines = 0;
        
        for msg in &self.messages {
            // Count actual lines in the message
            if msg.role != MessageRole::System {
                lines += 1; // For timestamp/prefix line
            }
            lines += msg.content.lines().count().max(1);
            lines += 1; // Spacing between messages
        }
        
        if let Some(ref response) = self.streaming_response {
            lines += 1; // Timestamp/prefix
            lines += response.lines().count().max(1);
            lines += 1; // Cursor line
        }
        
        lines
    }
}

pub fn draw_ai_chat_dialog(frame: &mut Frame, dialog: &mut AIChatDialog, theme: &cortex_core::Theme) {
    let size = frame.area();
    
    // Calculate dialog size (80% width, 70% height)
    let dialog_width = size.width * 4 / 5;
    let dialog_height = size.height * 7 / 10;
    
    let dialog_area = Rect::new(
        (size.width - dialog_width) / 2,
        (size.height - dialog_height) / 2,
        dialog_width,
        dialog_height,
    );
    
    // Clear the area and fill with panel background
    frame.render_widget(Clear, dialog_area);
    let background_block = Block::default()
        .style(Style::default().bg(theme.panel_background));
    frame.render_widget(background_block, dialog_area);
    
    // Create layout for messages and input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(dialog_area);
    
    // Build chat messages as Lines for Paragraph
    let mut lines: Vec<Line> = Vec::new();
    
    for msg in &dialog.messages {
        let (style, prefix) = match msg.role {
            MessageRole::User => (
                Style::default().fg(theme.info),
                "You: "
            ),
            MessageRole::Assistant => (
                Style::default().fg(theme.normal_text),
                "AI: "
            ),
            MessageRole::System => (
                Style::default().fg(theme.dim_text).add_modifier(Modifier::ITALIC),
                ""
            ),
        };
        
        // Add timestamp if not system message
        if msg.role != MessageRole::System {
            let timestamp = msg.timestamp.format("%H:%M:%S").to_string();
            lines.push(Line::from(vec![
                Span::styled(format!("[{}] ", timestamp), Style::default().fg(theme.dim_text)),
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
            ]));
        }
        
        // Split content into lines for better wrapping
        for line in msg.content.lines() {
            if line.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(Span::styled(line, style)));
            }
        }
        
        // Add spacing between messages
        lines.push(Line::from(""));
    }
    
    // Add streaming response if present
    if let Some(ref response) = dialog.streaming_response {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        lines.push(Line::from(vec![
            Span::styled(format!("[{}] ", timestamp), Style::default().fg(theme.dim_text)),
            Span::styled("AI: ", Style::default().fg(theme.normal_text).add_modifier(Modifier::BOLD)),
        ]));
        
        for line in response.lines() {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme.normal_text)
            )));
        }
        
        // Add blinking cursor to show it's still streaming
        lines.push(Line::from(Span::styled(
            "▊",
            Style::default().fg(theme.warning).add_modifier(Modifier::SLOW_BLINK)
        )));
    }
    
    // Create title with scroll indicators
    let total_lines = dialog.calculate_total_lines();
    let viewport_height = chunks[0].height.saturating_sub(2) as usize; // Subtract borders
    let can_scroll_up = dialog.scroll_position > 0;
    let can_scroll_down = dialog.scroll_position + viewport_height < total_lines;
    
    let title = if can_scroll_up && can_scroll_down {
        " AI Assistant Chat [↑↓ Scroll: Ctrl+Up/Down, PgUp/PgDn] "
    } else if can_scroll_up {
        " AI Assistant Chat [↑ Scroll up available] "
    } else if can_scroll_down {
        " AI Assistant Chat [↓ More messages below] "
    } else {
        " AI Assistant Chat "
    };
    
    // Create scrollable paragraph
    let messages_paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.active_border))
                .style(Style::default().bg(theme.panel_background))
        )
        .style(Style::default().bg(theme.panel_background))
        .wrap(Wrap { trim: false })
        .scroll((dialog.scroll_position as u16, 0));
    
    frame.render_widget(messages_paragraph, chunks[0]);
    
    // Draw input area
    let input_block = Block::default()
        .title(if dialog.is_processing { " Processing... " } else { " Type your command (Enter to send, Esc to close) " })
        .borders(Borders::ALL)
        .border_style(Style::default().fg(
            if dialog.is_processing { theme.warning } else { theme.active_border }
        ))
        .style(Style::default().bg(theme.panel_background));
    
    let input_paragraph = Paragraph::new(dialog.input.as_str())
        .block(input_block)
        .style(Style::default().fg(theme.normal_text).bg(theme.panel_background));
    
    frame.render_widget(input_paragraph, chunks[1]);
    
    // Set cursor position - always show cursor for input
    frame.set_cursor_position((
        chunks[1].x + 1 + dialog.cursor_position as u16,
        chunks[1].y + 1,
    ));
}