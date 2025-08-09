use cortex_core::{ActivePanel, AppState, FileEntry, FileType, PanelState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub struct UI;

impl UI {
    pub fn draw(frame: &mut Frame, app: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        Self::draw_panel(frame, panels[0], &app.left_panel, app.active_panel == ActivePanel::Left && !app.command_mode);
        Self::draw_panel(frame, panels[1], &app.right_panel, app.active_panel == ActivePanel::Right && !app.command_mode);
        Self::draw_command_line(frame, chunks[1], app);
        Self::draw_status_bar(frame, chunks[2], app);
    }

    fn draw_panel(frame: &mut Frame, area: Rect, panel: &PanelState, is_active: bool) {
        let border_style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = format!(" {} ", panel.current_dir.display());
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let visible_height = inner_area.height as usize;
        let start_idx = panel.view_offset;
        let end_idx = (start_idx + visible_height).min(panel.entries.len());

        let items: Vec<ListItem> = panel.entries[start_idx..end_idx]
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let absolute_idx = start_idx + idx;
                let is_selected = absolute_idx == panel.selected_index;
                let is_marked = panel.is_marked(&entry.path);

                let style = Self::get_entry_style(entry, is_selected, is_marked, is_active);
                let content = Self::format_entry(entry, inner_area.width as usize);
                
                ListItem::new(Line::from(vec![Span::styled(content, style)]))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner_area);
    }

    fn get_entry_style(entry: &FileEntry, is_selected: bool, is_marked: bool, panel_active: bool) -> Style {
        let mut style = Style::default();

        style = match entry.file_type {
            FileType::Directory => style.fg(Color::Blue).add_modifier(Modifier::BOLD),
            FileType::Symlink => style.fg(Color::Cyan),
            FileType::File => {
                if let Some(ext) = &entry.extension {
                    match ext.as_str() {
                        "rs" | "go" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" => {
                            style.fg(Color::Green)
                        }
                        "md" | "txt" | "doc" | "pdf" => style.fg(Color::Yellow),
                        "jpg" | "png" | "gif" | "svg" => style.fg(Color::Magenta),
                        "zip" | "tar" | "gz" | "rar" | "7z" => style.fg(Color::Red),
                        _ => style.fg(Color::White),
                    }
                } else {
                    style.fg(Color::White)
                }
            }
            FileType::Other => style.fg(Color::Gray),
        };

        if is_marked {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        if is_selected {
            if panel_active {
                style = style.bg(Color::Blue).add_modifier(Modifier::BOLD);
            } else {
                style = style.bg(Color::DarkGray);
            }
        }

        style
    }

    fn format_entry(entry: &FileEntry, width: usize) -> String {
        let type_indicator = match entry.file_type {
            FileType::Directory => "/",
            FileType::Symlink => "@",
            _ => "",
        };

        let name_with_indicator = format!("{}{}", entry.name, type_indicator);
        let size_str = &entry.size_display;
        let size_width = size_str.width();

        let available_width = width.saturating_sub(size_width + 2);
        let name_width = name_with_indicator.width();

        if name_width <= available_width {
            let padding = available_width - name_width;
            format!("{}{:padding$} {}", name_with_indicator, "", size_str, padding = padding)
        } else {
            let truncated = Self::truncate_string(&name_with_indicator, available_width - 3);
            format!("{}... {}", truncated, size_str)
        }
    }

    fn truncate_string(s: &str, max_width: usize) -> String {
        let mut width = 0;
        let mut result = String::new();

        for ch in s.chars() {
            let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
            if width + ch_width > max_width {
                break;
            }
            width += ch_width;
            result.push(ch);
        }

        result
    }

    fn draw_command_line(frame: &mut Frame, area: Rect, app: &AppState) {
        let (border_color, title) = if app.command_mode {
            (Color::Cyan, " Command Mode (ESC to exit) ")
        } else {
            (Color::Gray, " Press Ctrl+O or : to enter command mode ")
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let prompt = if app.command_mode {
            "$ "
        } else {
            "$ (Ctrl+O to activate) "
        };
        
        let text = format!("{}{}", prompt, app.command_line);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(if app.command_mode { Color::White } else { Color::DarkGray }));
        
        frame.render_widget(paragraph, inner_area);
        
        // Set cursor position when in command mode
        if app.command_mode {
            let actual_prompt = "$ ";
            frame.set_cursor_position((
                inner_area.x + actual_prompt.len() as u16 + app.command_cursor as u16,
                inner_area.y,
            ));
        }
    }

    fn draw_status_bar(frame: &mut Frame, area: Rect, app: &AppState) {
        let active_panel = app.active_panel();
        let current_entry = active_panel.current_entry();
        
        let left_text = if let Some(entry) = current_entry {
            format!(
                " {} | {} | {}",
                entry.name,
                entry.size_display,
                entry.permissions
            )
        } else {
            " No selection".to_string()
        };

        let (file_count, total_size) = if let Ok((count, size)) = 
            cortex_core::FileSystem::get_directory_info(&active_panel.current_dir) {
            (count, humansize::format_size(size, humansize::BINARY))
        } else {
            (0, "0 B".to_string())
        };

        let right_text = format!(
            "{} items | {} | F1 Help ",
            file_count,
            total_size
        );

        let left_width = left_text.width();
        let right_width = right_text.width();
        let padding = area.width.saturating_sub((left_width + right_width) as u16) as usize;
        
        let status_line = Line::from(vec![
            Span::styled(left_text, Style::default().fg(Color::White)),
            Span::raw(" ".repeat(padding)),
            Span::styled(right_text, Style::default().fg(Color::White)),
        ]);

        let paragraph = Paragraph::new(status_line)
            .style(Style::default().bg(Color::DarkGray))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}