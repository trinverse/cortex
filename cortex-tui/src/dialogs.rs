use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone)]
pub enum Dialog {
    Confirm(ConfirmDialog),
    Input(InputDialog),
    Progress(ProgressDialog),
    Error(ErrorDialog),
    Help(HelpDialog),
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub selected: bool,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            selected: true,
        }
    }

    pub fn toggle_selection(&mut self) {
        self.selected = !self.selected;
    }
}

#[derive(Debug, Clone)]
pub struct InputDialog {
    pub title: String,
    pub prompt: String,
    pub value: String,
    pub cursor_position: usize,
}

impl InputDialog {
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            value: String::new(),
            cursor_position: 0,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgressDialog {
    pub title: String,
    pub operation: String,
    pub current: u64,
    pub total: u64,
    pub message: String,
    pub can_cancel: bool,
}

impl ProgressDialog {
    pub fn new(title: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            operation: operation.into(),
            current: 0,
            total: 0,
            message: String::new(),
            can_cancel: true,
        }
    }

    pub fn update(&mut self, current: u64, total: u64, message: impl Into<String>) {
        self.current = current;
        self.total = total;
        self.message = message.into();
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorDialog {
    pub title: String,
    pub message: String,
    pub details: Option<String>,
}

impl ErrorDialog {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            title: "Error".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct HelpDialog {
    pub items: Vec<(String, String)>,
    pub selected_index: usize,
}

impl HelpDialog {
    pub fn new() -> Self {
        let items = vec![
            ("Navigation".to_string(), "".to_string()),
            ("↑/↓".to_string(), "Move selection".to_string()),
            ("←/→".to_string(), "Navigate directories".to_string()),
            ("Enter".to_string(), "Enter directory".to_string()),
            ("Tab".to_string(), "Switch panels".to_string()),
            ("Home/End".to_string(), "Jump to first/last".to_string()),
            ("PgUp/PgDn".to_string(), "Page up/down".to_string()),
            ("".to_string(), "".to_string()),
            ("File Operations".to_string(), "".to_string()),
            ("F5".to_string(), "Copy files".to_string()),
            ("F6".to_string(), "Move/rename files".to_string()),
            ("F7".to_string(), "Create directory".to_string()),
            ("F8".to_string(), "Delete files".to_string()),
            ("Space".to_string(), "Mark/unmark file".to_string()),
            ("Ctrl+A".to_string(), "Mark all".to_string()),
            ("Ctrl+U".to_string(), "Unmark all".to_string()),
            ("".to_string(), "".to_string()),
            ("View Options".to_string(), "".to_string()),
            ("Ctrl+H".to_string(), "Toggle hidden files".to_string()),
            ("Alt+1".to_string(), "Sort by name".to_string()),
            ("Alt+2".to_string(), "Sort by size".to_string()),
            ("Alt+3".to_string(), "Sort by date".to_string()),
            ("Alt+4".to_string(), "Sort by extension".to_string()),
            ("".to_string(), "".to_string()),
            ("Other".to_string(), "".to_string()),
            ("F1".to_string(), "This help".to_string()),
            ("Ctrl+O / :".to_string(), "Command mode".to_string()),
            ("Ctrl+R".to_string(), "Refresh panels".to_string()),
            ("Ctrl+Q".to_string(), "Quit".to_string()),
            ("".to_string(), "".to_string()),
            ("Command Mode".to_string(), "".to_string()),
            ("%f".to_string(), "Current file".to_string()),
            ("%F".to_string(), "Marked files".to_string()),
            ("%d".to_string(), "Current directory".to_string()),
            ("%D".to_string(), "Other panel directory".to_string()),
            ("Tab".to_string(), "Insert current file".to_string()),
            ("↑/↓".to_string(), "Command history".to_string()),
            ("Ctrl+U".to_string(), "Clear line".to_string()),
            ("Ctrl+W".to_string(), "Delete word".to_string()),
            ("ESC".to_string(), "Exit command mode".to_string()),
        ];

        Self {
            items,
            selected_index: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
        }
    }
}

pub fn render_dialog(frame: &mut Frame, dialog: &Dialog) {
    let area = centered_rect(60, 20, frame.area());
    
    match dialog {
        Dialog::Confirm(d) => render_confirm_dialog(frame, area, d),
        Dialog::Input(d) => render_input_dialog(frame, area, d),
        Dialog::Progress(d) => render_progress_dialog(frame, area, d),
        Dialog::Error(d) => render_error_dialog(frame, area, d),
        Dialog::Help(d) => render_help_dialog(frame, d),
    }
}

fn render_confirm_dialog(frame: &mut Frame, area: Rect, dialog: &ConfirmDialog) {
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title(dialog.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(inner);
    
    let message = Paragraph::new(dialog.message.as_str())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(message, chunks[0]);
    
    let buttons = if dialog.selected {
        Line::from(vec![
            Span::styled(" [Yes] ", Style::default().bg(Color::Blue).fg(Color::White)),
            Span::raw("  "),
            Span::raw(" [No] "),
        ])
    } else {
        Line::from(vec![
            Span::raw(" [Yes] "),
            Span::raw("  "),
            Span::styled(" [No] ", Style::default().bg(Color::Blue).fg(Color::White)),
        ])
    };
    
    let buttons_paragraph = Paragraph::new(buttons)
        .alignment(Alignment::Center);
    frame.render_widget(buttons_paragraph, chunks[1]);
}

fn render_input_dialog(frame: &mut Frame, area: Rect, dialog: &InputDialog) {
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title(dialog.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);
    
    let prompt = Paragraph::new(dialog.prompt.as_str());
    frame.render_widget(prompt, chunks[0]);
    
    let input_block = Block::default()
        .borders(Borders::ALL);
    let input_inner = input_block.inner(chunks[1]);
    frame.render_widget(input_block, chunks[1]);
    
    let input = Paragraph::new(dialog.value.as_str());
    frame.render_widget(input, input_inner);
    
    frame.set_cursor_position((
        input_inner.x + dialog.cursor_position as u16,
        input_inner.y,
    ));
}

fn render_progress_dialog(frame: &mut Frame, area: Rect, dialog: &ProgressDialog) {
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title(dialog.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);
    
    let operation = Paragraph::new(dialog.operation.as_str());
    frame.render_widget(operation, chunks[0]);
    
    let progress = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Green))
        .percent(dialog.percentage() as u16)
        .label(format!("{}%", dialog.percentage() as u16));
    frame.render_widget(progress, chunks[1]);
    
    let status = Paragraph::new(format!(
        "{} / {} bytes",
        dialog.current, dialog.total
    ));
    frame.render_widget(status, chunks[2]);
    
    let message = Paragraph::new(dialog.message.as_str())
        .wrap(Wrap { trim: true });
    frame.render_widget(message, chunks[3]);
    
    if dialog.can_cancel {
        let cancel_hint = Paragraph::new("Press ESC to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(cancel_hint, chunks[4]);
    }
}

fn render_error_dialog(frame: &mut Frame, area: Rect, dialog: &ErrorDialog) {
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title(dialog.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = if dialog.details.is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner)
    };
    
    let message = Paragraph::new(dialog.message.as_str())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(message, chunks[0]);
    
    if let Some(details) = &dialog.details {
        let details_widget = Paragraph::new(details.as_str())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(details_widget, chunks[2]);
    }
    
    let ok_button = Paragraph::new("[OK]")
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Red).fg(Color::White));
    frame.render_widget(ok_button, chunks[chunks.len() - 1]);
}

fn render_help_dialog(frame: &mut Frame, dialog: &HelpDialog) {
    let area = centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let items: Vec<ListItem> = dialog.items.iter().enumerate().map(|(idx, (key, desc))| {
        if desc.is_empty() && !key.is_empty() {
            ListItem::new(Line::from(vec![
                Span::styled(key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]))
        } else if key.is_empty() {
            ListItem::new(Line::from(vec![Span::raw("")]))
        } else {
            let style = if idx == dialog.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {:12}", key), Style::default().fg(Color::Cyan)),
                Span::styled(desc, style),
            ]))
        }
    }).collect();
    
    let list = List::new(items);
    frame.render_widget(list, inner);
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