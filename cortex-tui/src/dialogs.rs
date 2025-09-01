use crate::ai_chat_dialog::AIChatDialog;
use crate::api_key_dialog::APIKeyDialog;
use crate::command_palette_dialog::CommandPaletteDialog;
use crate::config_dialog::ConfigDialog;
use crate::connection_dialog::ConnectionDialog;
use crate::editor_dialog::EditorDialog;
use crate::filter_dialog::FilterDialog;
use crate::plugin_dialog::PluginDialog;
use crate::search_dialog::SearchDialog;
use crate::viewer_dialog::ViewerDialog;
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
    Viewer(ViewerDialog),
    Editor(EditorDialog),
    Filter(FilterDialog),
    CommandPalette(CommandPaletteDialog),
    Search(SearchDialog),
    Connection(ConnectionDialog),
    Plugin(PluginDialog),
    Config(ConfigDialog),
    SaveConfirm(SaveConfirmDialog),
    ThemeSelection(ThemeSelectionDialog),
    Suggestions(SuggestionsDialog),
    AIChat(AIChatDialog),
    APIKey(APIKeyDialog),
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
pub struct SaveConfirmDialog {
    pub filename: String,
    pub selection: SaveChoice,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveChoice {
    Save,
    DontSave,
    Cancel,
}

impl SaveConfirmDialog {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            selection: SaveChoice::Save,
        }
    }

    pub fn next_choice(&mut self) {
        self.selection = match self.selection {
            SaveChoice::Save => SaveChoice::DontSave,
            SaveChoice::DontSave => SaveChoice::Cancel,
            SaveChoice::Cancel => SaveChoice::Save,
        };
    }

    pub fn prev_choice(&mut self) {
        self.selection = match self.selection {
            SaveChoice::Save => SaveChoice::Cancel,
            SaveChoice::DontSave => SaveChoice::Save,
            SaveChoice::Cancel => SaveChoice::DontSave,
        };
    }
}

#[derive(Debug, Clone)]
pub struct ThemeSelectionDialog {
    pub selected_index: usize,
    pub themes: Vec<(cortex_core::ThemeMode, String)>,
}

impl ThemeSelectionDialog {
    pub fn new(current_mode: cortex_core::ThemeMode) -> Self {
        let themes = vec![
            (cortex_core::ThemeMode::Dark, "Dark".to_string()),
            (cortex_core::ThemeMode::Light, "Light".to_string()),
            (cortex_core::ThemeMode::Gruvbox, "Gruvbox".to_string()),
            (cortex_core::ThemeMode::Nord, "Nord".to_string()),
            (
                cortex_core::ThemeMode::Random,
                "Random (changes every 10 min)".to_string(),
            ),
        ];

        let selected_index = themes
            .iter()
            .position(|(mode, _)| *mode == current_mode)
            .unwrap_or(0);

        Self {
            selected_index,
            themes,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.themes.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn get_selected_theme(&self) -> cortex_core::ThemeMode {
        self.themes[self.selected_index].0
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

impl Default for HelpDialog {
    fn default() -> Self {
        Self::new()
    }
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
            ("F9".to_string(), "Config".to_string()),
            ("F10".to_string(), "Quit application".to_string()),
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
            ("Ctrl+K".to_string(), "API Key Configuration".to_string()),
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

pub fn render_dialog(frame: &mut Frame, dialog: &mut Dialog, theme: &cortex_core::Theme) {
    match dialog {
        Dialog::Confirm(d) => {
            let area = centered_rect(60, 20, frame.size());
            render_confirm_dialog(frame, area, d)
        }
        Dialog::Input(d) => {
            let area = centered_rect(60, 20, frame.size());
            render_input_dialog(frame, area, d)
        }
        Dialog::Progress(d) => {
            let area = centered_rect(60, 20, frame.size());
            render_progress_dialog(frame, area, d)
        }
        Dialog::Error(d) => {
            let area = centered_rect(60, 20, frame.size());
            render_error_dialog(frame, area, d)
        }
        Dialog::Help(d) => render_help_dialog(frame, d),
        Dialog::Viewer(d) => d.render(frame),
        Dialog::Editor(d) => d.render(frame),
        Dialog::Filter(d) => d.render(frame),
        Dialog::CommandPalette(d) => d.render(frame),
        Dialog::Search(d) => d.render(frame),
        Dialog::Connection(d) => d.render(frame),
        Dialog::Plugin(d) => d.render(frame),
        Dialog::Config(d) => d.render(frame, theme),
        Dialog::SaveConfirm(d) => {
            let area = centered_rect(60, 20, frame.size());
            render_save_confirm_dialog(frame, area, d)
        }
        Dialog::ThemeSelection(d) => {
            let area = centered_rect(50, 30, frame.size());
            render_theme_selection_dialog(frame, area, d)
        }
        Dialog::Suggestions(d) => {
            draw_suggestions_dialog(frame, d, theme)
        }
        Dialog::AIChat(d) => {
            crate::ai_chat_dialog::draw_ai_chat_dialog(frame, d, theme)
        }
        Dialog::APIKey(d) => {
            let area = frame.size();
            d.render(frame, area)
        }
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
        .constraints([Constraint::Min(3), Constraint::Length(3)])
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

    let buttons_paragraph = Paragraph::new(buttons).alignment(Alignment::Center);
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

    let input_block = Block::default().borders(Borders::ALL);
    let input_inner = input_block.inner(chunks[1]);
    frame.render_widget(input_block, chunks[1]);

    let input = Paragraph::new(dialog.value.as_str());
    frame.render_widget(input, input_inner);

    frame.set_cursor(input_inner.x + dialog.cursor_position as u16, input_inner.y);
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

    let status = Paragraph::new(format!("{} / {} bytes", dialog.current, dialog.total));
    frame.render_widget(status, chunks[2]);

    let message = Paragraph::new(dialog.message.as_str()).wrap(Wrap { trim: true });
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
            .constraints([Constraint::Min(3), Constraint::Length(1)])
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

fn render_save_confirm_dialog(frame: &mut Frame, area: Rect, dialog: &SaveConfirmDialog) {
    use crate::dialogs::SaveChoice;

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Save Changes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(inner);

    // File name
    let filename = format!("File: {}", dialog.filename);
    let filename_widget = Paragraph::new(filename)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(filename_widget, chunks[0]);

    // Message
    let message = "The file has been modified.\nDo you want to save changes before closing?";
    let message_widget = Paragraph::new(message)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(message_widget, chunks[1]);

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[3]);

    let save_style = if dialog.selection == SaveChoice::Save {
        Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let dont_save_style = if dialog.selection == SaveChoice::DontSave {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let cancel_style = if dialog.selection == SaveChoice::Cancel {
        Style::default()
            .bg(Color::Red)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };

    let save_button = Paragraph::new("[S]ave")
        .alignment(Alignment::Center)
        .style(save_style);
    frame.render_widget(save_button, button_chunks[0]);

    let dont_save_button = Paragraph::new("[D]on't Save")
        .alignment(Alignment::Center)
        .style(dont_save_style);
    frame.render_widget(dont_save_button, button_chunks[1]);

    let cancel_button = Paragraph::new("[C]ancel")
        .alignment(Alignment::Center)
        .style(cancel_style);
    frame.render_widget(cancel_button, button_chunks[2]);
}

fn render_help_dialog(frame: &mut Frame, dialog: &HelpDialog) {
    let area = centered_rect(70, 80, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = dialog
        .items
        .iter()
        .enumerate()
        .map(|(idx, (key, desc))| {
            if desc.is_empty() && !key.is_empty() {
                ListItem::new(Line::from(vec![Span::styled(
                    key,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]))
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
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_theme_selection_dialog(frame: &mut Frame, area: Rect, dialog: &ThemeSelectionDialog) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Theme ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(inner);

    // Instructions
    let instructions = Paragraph::new("Use ↑/↓ to select, Enter to apply, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[0]);

    // Theme list
    let items: Vec<ListItem> = dialog
        .themes
        .iter()
        .enumerate()
        .map(|(i, (_, name))| {
            let style = if i == dialog.selected_index {
                Style::default()
                    .bg(Color::Rgb(40, 44, 52))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let content = if i == dialog.selected_index {
                format!("  ▶ {}", name)
            } else {
                format!("    {}", name)
            };

            ListItem::new(Line::from(vec![Span::styled(content, style)]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[1]);

    // Status
    let status = Paragraph::new("Press Enter to apply the selected theme")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[2]);
}

#[derive(Debug, Clone)]
pub struct SuggestionsDialog {
    pub suggestions: Vec<(String, String)>, // (display_name, full_path)
    pub selected_index: usize,
}

impl SuggestionsDialog {
    pub fn new(suggestions: Vec<(String, String)>) -> Self {
        Self {
            suggestions,
            selected_index: 0,
        }
    }
    
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.suggestions.is_empty() {
            self.selected_index = self.suggestions.len() - 1;
        }
    }
    
    pub fn move_down(&mut self) {
        if self.selected_index < self.suggestions.len().saturating_sub(1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }
    
    pub fn get_selected_suggestion(&self) -> Option<&String> {
        self.suggestions.get(self.selected_index).map(|(_, path)| path)
    }
    
    pub fn get_selected_display_name(&self) -> Option<&String> {
        self.suggestions.get(self.selected_index).map(|(name, _)| name)
    }
}

fn draw_suggestions_dialog(frame: &mut Frame, dialog: &SuggestionsDialog, theme: &cortex_core::Theme) {
    // Position the dialog as a small overlay near the command line
    let area = frame.size();
    let suggestion_count = dialog.suggestions.len().min(5);
    let popup_area = Rect {
        x: area.x + 2,
        y: area.height.saturating_sub((suggestion_count as u16) + 6), // Just above command line
        width: area.width.saturating_sub(4).min(50), // Narrower width
        height: (suggestion_count as u16) + 2, // Exact height needed + borders
    };

    // Clear the area
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Directories (Enter=accept, ↑↓=navigate, Esc=close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.active_border));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Create list items from suggestions - show just the directory names
    let items: Vec<ListItem> = dialog.suggestions
        .iter()
        .take(5) // Show max 5 suggestions
        .enumerate()
        .map(|(idx, (display_name, _))| {
            let is_selected = idx == dialog.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(theme.selected_fg)
                    .bg(theme.selected_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.command_line_fg)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {}/", display_name), style) // Add trailing slash to indicate directory
            ]))
        })
        .collect();

    let suggestions_list = List::new(items);
    frame.render_widget(suggestions_list, inner_area);
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
