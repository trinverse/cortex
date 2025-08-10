use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Sftp,
    Ftp,
}

#[derive(Debug, Clone)]
pub struct ConnectionDialog {
    pub connection_type: ConnectionType,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub private_key_path: String,
    pub use_private_key: bool,
    pub selected_field: usize,
    pub cursor_position: usize,
}

impl ConnectionDialog {
    pub fn new() -> Self {
        Self {
            connection_type: ConnectionType::Sftp,
            host: String::new(),
            port: "22".to_string(),
            username: String::new(),
            password: String::new(),
            private_key_path: String::new(),
            use_private_key: false,
            selected_field: 0,
            cursor_position: 0,
        }
    }

    pub fn with_type(mut self, connection_type: ConnectionType) -> Self {
        match connection_type {
            ConnectionType::Sftp => self.port = "22".to_string(),
            ConnectionType::Ftp => self.port = "21".to_string(),
        }
        self.connection_type = connection_type;
        self
    }

    pub fn next_field(&mut self) {
        let max_fields = if self.use_private_key { 5 } else { 4 };
        self.selected_field = (self.selected_field + 1) % max_fields;
        self.cursor_position = self.get_current_field_content().len();
    }

    pub fn prev_field(&mut self) {
        let max_fields = if self.use_private_key { 5 } else { 4 };
        if self.selected_field == 0 {
            self.selected_field = max_fields - 1;
        } else {
            self.selected_field -= 1;
        }
        self.cursor_position = self.get_current_field_content().len();
    }

    pub fn get_current_field_content(&self) -> &String {
        match self.selected_field {
            0 => &self.host,
            1 => &self.port,
            2 => &self.username,
            3 => {
                if self.use_private_key {
                    &self.private_key_path
                } else {
                    &self.password
                }
            }
            4 => &self.password, // Only when use_private_key is true
            _ => &self.host,
        }
    }

    pub fn get_current_field_content_mut(&mut self) -> &mut String {
        match self.selected_field {
            0 => &mut self.host,
            1 => &mut self.port,
            2 => &mut self.username,
            3 => {
                if self.use_private_key {
                    &mut self.private_key_path
                } else {
                    &mut self.password
                }
            }
            4 => &mut self.password, // Only when use_private_key is true
            _ => &mut self.host,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let cursor_pos = self.cursor_position;
        let field = self.get_current_field_content_mut();
        field.insert(cursor_pos, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let cursor_pos = self.cursor_position - 1;
            let field = self.get_current_field_content_mut();
            if !field.is_empty() {
                field.remove(cursor_pos);
                self.cursor_position = cursor_pos;
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let field = self.get_current_field_content();
        if self.cursor_position < field.len() {
            self.cursor_position += 1;
        }
    }

    pub fn toggle_auth_method(&mut self) {
        self.use_private_key = !self.use_private_key;
        if self.selected_field >= 3 {
            self.selected_field = 3;
        }
        self.cursor_position = self.get_current_field_content().len();
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = self.centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);

        let connection_name = match self.connection_type {
            ConnectionType::Sftp => "SFTP",
            ConnectionType::Ftp => "FTP",
        };

        let block = Block::default()
            .title(format!(" {} Connection ", connection_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Instructions
                Constraint::Length(3), // Host
                Constraint::Length(3), // Port
                Constraint::Length(3), // Username
                Constraint::Length(3), // Auth method toggle
                Constraint::Length(3), // Password/Key field
                if self.use_private_key {
                    Constraint::Length(3)
                } else {
                    Constraint::Length(0)
                }, // Password for key
                Constraint::Min(2),    // Connect/Cancel buttons
            ])
            .split(inner);

        // Instructions
        let instructions = Paragraph::new(
            "Tab/Shift+Tab: navigate fields, Ctrl+T: toggle auth, Enter: connect, Esc: cancel",
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
        frame.render_widget(instructions, chunks[0]);

        // Host field
        self.render_field(frame, chunks[1], "Host", &self.host, 0);

        // Port field
        self.render_field(frame, chunks[2], "Port", &self.port, 1);

        // Username field
        self.render_field(frame, chunks[3], "Username", &self.username, 2);

        // Auth method toggle
        let auth_text = if self.use_private_key {
            "Auth: Private Key (Ctrl+T to use password)"
        } else {
            "Auth: Password (Ctrl+T to use private key)"
        };
        let auth_para = Paragraph::new(auth_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(auth_para, chunks[4]);

        // Password or Private Key field
        if self.use_private_key {
            self.render_field(
                frame,
                chunks[5],
                "Private Key Path",
                &self.private_key_path,
                3,
            );
            if chunks.len() > 6 {
                self.render_field(frame, chunks[6], "Passphrase", &self.password, 4);
            }
        } else {
            self.render_field(frame, chunks[5], "Password", &self.password, 3);
        }

        // Connect/Cancel buttons
        let button_chunk = chunks[if self.use_private_key && chunks.len() > 6 {
            7
        } else {
            6
        }];
        let buttons = Paragraph::new("[Enter] Connect    [Esc] Cancel")
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(buttons, button_chunk);
    }

    fn render_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        label: &str,
        value: &str,
        field_index: usize,
    ) {
        let is_selected = self.selected_field == field_index;
        let is_password = label == "Password" || label == "Passphrase";

        let display_value = if is_password && !value.is_empty() {
            "*".repeat(value.len())
        } else {
            value.to_string()
        };

        let field_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(area);

        // Label
        let label_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let label_para = Paragraph::new(format!("{}:", label)).style(label_style);
        frame.render_widget(label_para, field_chunks[0]);

        // Input field
        let input_style = if is_selected {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });

        let input_inner = input_block.inner(field_chunks[1]);
        frame.render_widget(input_block, field_chunks[1]);

        let input_para = Paragraph::new(display_value).style(input_style);
        frame.render_widget(input_para, input_inner);

        // Cursor
        if is_selected {
            let cursor_x = input_inner.x + self.cursor_position.min(value.len()) as u16;
            frame.set_cursor_position((cursor_x, input_inner.y));
        }
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
