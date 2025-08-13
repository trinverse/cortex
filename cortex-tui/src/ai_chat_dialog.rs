use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
    }
    
    pub fn update_streaming_response(&mut self, chunk: String) {
        if let Some(ref mut response) = self.streaming_response {
            response.push_str(&chunk);
        } else {
            self.streaming_response = Some(chunk);
        }
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
}

pub fn draw_ai_chat_dialog(frame: &mut Frame, dialog: &AIChatDialog, theme: &cortex_core::Theme) {
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
    
    // Draw chat messages
    let mut messages: Vec<ListItem> = Vec::new();
    for msg in &dialog.messages {
        let style = match msg.role {
            MessageRole::User => Style::default().fg(theme.info),
            MessageRole::Assistant => Style::default().fg(theme.normal_text),
            MessageRole::System => Style::default().fg(theme.dim_text).add_modifier(Modifier::ITALIC),
        };
        
        let prefix = match msg.role {
            MessageRole::User => "You: ",
            MessageRole::Assistant => "AI: ",
            MessageRole::System => "",
        };
        
        let content = format!("{}{}", prefix, msg.content);
        messages.push(ListItem::new(content).style(style));
    }
    
    // Add streaming response if present
    if let Some(ref response) = dialog.streaming_response {
        messages.push(
            ListItem::new(format!("AI: {}", response))
                .style(Style::default().fg(theme.normal_text))
        );
    }
    
    let messages_list = List::new(messages)
        .block(
            Block::default()
                .title(" AI Assistant Chat ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.active_border))
                .style(Style::default().bg(theme.panel_background))
        )
        .style(Style::default().bg(theme.panel_background));
    
    frame.render_widget(messages_list, chunks[0]);
    
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