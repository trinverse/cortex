use cortex_core::{
    state::{AppState, PanelState, ViewMode},
    ActivePanel, FileEntry, FileType, VfsEntry, VfsEntryType,
};
use humansize;
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
        let theme = app.theme_manager.get_current_theme();

        // FIRST: Fill entire terminal with theme background
        Self::draw_background(frame, frame.size(), theme);

        // Calculate command line height based on text width
        let terminal_width = frame.size().width as usize;
        let prompt = "$ ";
        let prompt_width = prompt.width();
        let border_width = 2; // Left and right borders
        let available_width = terminal_width.saturating_sub(border_width);

        let command_line_height = if available_width > prompt_width {
            let total_width = prompt_width + app.command_line.width();
            let lines_needed = total_width.div_ceil(available_width);
            (lines_needed as u16).clamp(1, 5) + 2 // +2 for borders, max 5 lines of text
        } else {
            3 // Default minimum height with borders
        };

        // Adjust layout based on whether command output is visible
        let constraints = if app.command_output_visible {
            vec![
                Constraint::Min(3),
                Constraint::Length(app.command_output_height),
                Constraint::Length(command_line_height),
                Constraint::Length(2), // Increased to 2 for function key bar
            ]
        } else {
            vec![
                Constraint::Min(3),
                Constraint::Length(command_line_height),
                Constraint::Length(2), // Increased to 2 for function key bar
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(frame.size());

        let (panels_area, command_output_area, command_area, status_area) =
            if app.command_output_visible {
                (chunks[0], Some(chunks[1]), chunks[2], chunks[3])
            } else {
                (chunks[0], None, chunks[1], chunks[2])
            };

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(panels_area);
        Self::draw_panel(
            frame,
            panels[0],
            &app.left_panel,
            app.active_panel == ActivePanel::Left,
            app,
        );
        Self::draw_panel(
            frame,
            panels[1],
            &app.right_panel,
            app.active_panel == ActivePanel::Right,
            app,
        );

        // Draw command output area if visible
        if let Some(output_area) = command_output_area {
            Self::draw_command_output(frame, output_area, app, theme);
        }

        Self::draw_command_line(frame, command_area, app, theme);
        Self::draw_status_bar(frame, status_area, app, theme);
    }

    fn draw_panel(
        frame: &mut Frame,
        area: Rect,
        panel: &PanelState,
        is_active: bool,
        app: &AppState,
    ) {
        let theme = app.theme_manager.get_current_theme();
        let border_style = theme.get_border_style(is_active);

        let title = if let Some(ref filter) = panel.filter {
            if panel.is_using_vfs() {
                if let Some(ref vfs_path) = panel.current_vfs_path {
                    match vfs_path {
                        cortex_core::vfs::VfsPath::Archive { .. } => {
                            format!(" [Archive] [Filter: {}] ", filter)
                        }
                        cortex_core::vfs::VfsPath::Sftp { host, username, .. } => {
                            format!(" [SFTP: {}@{}] [Filter: {}] ", username, host, filter)
                        }
                        cortex_core::vfs::VfsPath::Ftp { host, username, .. } => {
                            format!(" [FTP: {}@{}] [Filter: {}] ", username, host, filter)
                        }
                        _ => format!(" [Remote] [Filter: {}] ", filter),
                    }
                } else {
                    format!(" [Archive] [Filter: {}] ", filter)
                }
            } else {
                format!(" {} [Filter: {}] ", panel.current_dir.display(), filter)
            }
        } else if panel.is_using_vfs() {
            if let Some(ref vfs_path) = panel.current_vfs_path {
                match vfs_path {
                    cortex_core::vfs::VfsPath::Archive { .. } => " [Archive] ".to_string(),
                    cortex_core::vfs::VfsPath::Sftp {
                        host,
                        username,
                        path,
                        ..
                    } => {
                        format!(" [SFTP: {}@{}:{}] ", username, host, path)
                    }
                    cortex_core::vfs::VfsPath::Ftp {
                        host,
                        username,
                        path,
                        ..
                    } => {
                        format!(" [FTP: {}@{}:{}] ", username, host, path)
                    }
                    _ => " [Remote] ".to_string(),
                }
            } else {
                " [Archive] ".to_string()
            }
        } else {
            format!(" {} ", panel.current_dir.display())
        };
        // First fill panel area with panel background color
        let panel_bg = Block::default().style(Style::default().bg(theme.panel_background));
        frame.render_widget(panel_bg, area);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(theme.panel_background));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let visible_height = inner_area.height as usize;
        let start_idx = panel.view_offset;

        let items: Vec<ListItem> = if panel.is_using_vfs() {
            // Render VFS entries
            let vfs_entries = panel.get_visible_vfs_entries();
            let end_idx = (start_idx + visible_height).min(vfs_entries.len());

            vfs_entries[start_idx..end_idx]
                .iter()
                .enumerate()
                .map(|(idx, entry)| {
                    let absolute_idx = start_idx + idx;
                    let is_selected = absolute_idx == panel.selected_index;

                    let style = Self::get_vfs_entry_style(entry, is_selected, is_active, theme);
                    let content = Self::format_vfs_entry(entry, inner_area.width as usize);

                    ListItem::new(Line::from(vec![Span::styled(content, style)]))
                })
                .collect()
        } else {
            // Render regular entries
            let entries = panel.get_visible_entries();
            let end_idx = (start_idx + visible_height).min(entries.len());

            entries[start_idx..end_idx]
                .iter()
                .enumerate()
                .map(move |(idx, entry)| {
                    let absolute_idx = start_idx + idx;
                    let is_selected = absolute_idx == panel.selected_index;
                    let is_marked = panel.is_marked(&entry.path);

                    let style =
                        Self::get_entry_style(entry, is_selected, is_marked, is_active, theme);
                    let content = Self::format_entry(entry, inner_area.width as usize, panel, &app.config_manager.get());

                    ListItem::new(Line::from(vec![Span::styled(content, style)]))
                })
                .collect()
        };

        let list = List::new(items);
        frame.render_widget(list, inner_area);
    }

    fn get_entry_style(
        entry: &FileEntry,
        is_selected: bool,
        is_marked: bool,
        panel_active: bool,
        theme: &cortex_core::Theme,
    ) -> Style {
        let mut style = theme.get_file_style(&entry.file_type, entry.extension.as_ref());

        if is_marked {
            style = style
                .fg(theme.marked)
                .add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
        }

        if is_selected {
            let selected_style = theme.get_selected_style(panel_active);
            // Preserve the foreground color but use the selected background
            style = style.bg(selected_style.bg.unwrap_or(Color::Reset));
            if panel_active {
                style = style.add_modifier(Modifier::BOLD);
            }
        }

        style
    }

    fn format_entry(entry: &FileEntry, width: usize, panel: &PanelState, config: &cortex_core::Config) -> String {
        let icon = if config.general.show_icons {
            match entry.file_type {
                FileType::Directory => " ",
                FileType::Symlink => " ",
                FileType::File => " ",
                _ => "  ",
            }
        } else {
            ""
        };

        let type_indicator = match entry.file_type {
            FileType::Directory => "/",
            FileType::Symlink => "@",
            _ => "",
        };

        let name_with_indicator = format!("{}{}{}", icon, entry.name, type_indicator);
        
        // Build the info section based on view mode
        let mut info_parts = Vec::new();
        
        match panel.view_mode {
            ViewMode::Brief => {
                // No info parts
            }
            ViewMode::Full => {
                info_parts.push(entry.size_display.clone());
                if let Some(modified) = entry.modified {
                    info_parts.push(modified.format("%m-%d %H:%M").to_string());
                }
            }
            ViewMode::Wide => {
                info_parts.push(entry.size_display.clone());
                info_parts.push(entry.permissions.clone());
                if let Some(modified) = entry.modified {
                    info_parts.push(modified.format("%m-%d %H:%M").to_string());
                }
            }
        }
        
        let info_str = if info_parts.is_empty() {
            String::new()
        } else {
            info_parts.join(" | ")
        };
        
        let info_width = info_str.width();
        let padding_needed = if info_width > 0 { 2 } else { 0 }; // Space for separator
        let available_width = width.saturating_sub(info_width + padding_needed);
        let name_width = name_with_indicator.width();

        if name_width <= available_width {
            let padding = available_width - name_width;
            if info_width > 0 {
                format!(
                    "{}{:padding$} {}",
                    name_with_indicator,
                    "",
                    info_str,
                    padding = padding
                )
            } else {
                name_with_indicator
            }
        } else {
            let truncated = Self::truncate_string(&name_with_indicator, available_width - 3);
            if info_width > 0 {
                format!("{}... {}", truncated, info_str)
            } else {
                format!("{}...", truncated)
            }
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

    fn get_vfs_entry_style(
        entry: &VfsEntry,
        is_selected: bool,
        panel_active: bool,
        theme: &cortex_core::Theme,
    ) -> Style {
        let mut style = Style::default();

        style = match entry.entry_type {
            VfsEntryType::Directory => style.fg(theme.directory).add_modifier(Modifier::BOLD),
            VfsEntryType::Archive => style.fg(theme.archive).add_modifier(Modifier::BOLD),
            VfsEntryType::Symlink => style.fg(theme.symlink),
            VfsEntryType::File => {
                // Try to infer type from extension
                let extension = entry.name.rsplit('.').next();
                if let Some(ext) = extension {
                    match ext.to_lowercase().as_str() {
                        "rs" | "go" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" => {
                            style.fg(theme.source_code)
                        }
                        "md" | "txt" | "doc" | "pdf" => style.fg(theme.document),
                        "jpg" | "png" | "gif" | "svg" => style.fg(theme.image),
                        "zip" | "tar" | "gz" | "rar" | "7z" => style.fg(theme.archive),
                        _ => style.fg(theme.regular_file),
                    }
                } else {
                    style.fg(theme.regular_file)
                }
            }
        };

        if is_selected {
            let selected_style = theme.get_selected_style(panel_active);
            style = style.bg(selected_style.bg.unwrap_or(Color::Reset));
            if panel_active {
                style = style.add_modifier(Modifier::BOLD);
            }
        }

        style
    }

    fn format_vfs_entry(entry: &VfsEntry, width: usize) -> String {
        let type_indicator = match entry.entry_type {
            VfsEntryType::Directory => "/",
            VfsEntryType::Archive => "@",
            VfsEntryType::Symlink => "@",
            VfsEntryType::File => "",
        };

        let name_with_indicator = format!("{}{}", entry.name, type_indicator);
        let size_str = if let Some(compressed) = entry.compressed_size {
            format!(
                "{} ({})",
                humansize::format_size(entry.size, humansize::BINARY),
                humansize::format_size(compressed, humansize::BINARY)
            )
        } else {
            humansize::format_size(entry.size, humansize::BINARY)
        };
        let size_width = size_str.width();

        let available_width = width.saturating_sub(size_width + 2);
        let name_width = name_with_indicator.width();

        if name_width <= available_width {
            let padding = available_width - name_width;
            format!(
                "{}{:padding$} {}",
                name_with_indicator,
                "",
                size_str,
                padding = padding
            )
        } else {
            let truncated = Self::truncate_string(&name_with_indicator, available_width - 3);
            format!("{}... {}", truncated, size_str)
        }
    }

    fn draw_command_line(
        frame: &mut Frame,
        area: Rect,
        app: &AppState,
        theme: &cortex_core::Theme,
    ) {
        let title = if app.command_line.starts_with('/') {
            " Special Commands (/ for menu) "
        } else {
            " Command Line (type to execute, / for special) "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.active_border));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let prompt = "$ ";
        let text = format!("{}{}", prompt, app.command_line);

        // Use Paragraph with wrap for multi-line support
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(theme.command_line_fg))
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(paragraph, inner_area);

        // Calculate cursor position considering line wrapping
        let prompt_and_cursor = prompt.len() + app.command_cursor;
        let width = inner_area.width as usize;
        let cursor_line = prompt_and_cursor / width;
        let cursor_col = prompt_and_cursor % width;

        // Only show cursor if it's within the visible area
        if cursor_line < inner_area.height as usize {
            frame.set_cursor(
                inner_area.x + cursor_col as u16,
                inner_area.y + cursor_line as u16,
);
        }
    }

    fn draw_status_bar(frame: &mut Frame, area: Rect, app: &AppState, theme: &cortex_core::Theme) {
        // Split status area into two lines
        let status_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(area);

        // Draw the info line (top)
        Self::draw_info_line(frame, status_chunks[0], app, theme);

        // Draw the function key bar (bottom)
        Self::draw_function_key_bar(frame, status_chunks[1], theme);
    }

    fn draw_info_line(frame: &mut Frame, area: Rect, app: &AppState, theme: &cortex_core::Theme) {
        let active_panel = app.active_panel();
        let current_entry = active_panel.current_entry();

        let left_text = if let Some(entry) = current_entry {
            format!(
                " {} | {} | {}",
                entry.name, entry.size_display, entry.permissions
            )
        } else {
            " No selection".to_string()
        };

        let (file_count, total_size) = if let Ok((count, size)) =
            cortex_core::FileSystem::get_directory_info(&active_panel.current_dir)
        {
            (count, humansize::format_size(size, humansize::BINARY))
        } else {
            (0, "0 B".to_string())
        };

        // Build the middle section with git info
        let middle_text = if let Some(ref git_info) = active_panel.git_info {
            let branch = &git_info.branch;
            let dirty_indicator = if git_info.is_dirty { "*" } else { "" };
            let ahead_behind = if git_info.ahead > 0 || git_info.behind > 0 {
                format!(" ↑{} ↓{}", git_info.ahead, git_info.behind)
            } else {
                String::new()
            };
            format!(" 🔀 {}{}{}", branch, dirty_indicator, ahead_behind)
        } else {
            String::new()
        };

        let theme_name = format!("{:?}", theme.mode);
        let right_text = format!("{} items | {} | {}", file_count, total_size, theme_name);

        // Calculate spacing
        let left_width = left_text.width();
        let middle_width = middle_text.width();
        let right_width = right_text.width();
        let total_width = area.width as usize;

        // Create the status line with proper spacing
        let mut spans = vec![Span::styled(
            left_text,
            Style::default().fg(theme.status_bar_fg),
        )];

        if !middle_text.is_empty() {
            // Add padding before git info
            let padding_before =
                ((total_width.saturating_sub(left_width + middle_width + right_width)) / 2).max(1);
            spans.push(Span::raw(" ".repeat(padding_before)));

            // Add git info with color based on status
            let git_style = if active_panel.git_info.as_ref().unwrap().is_dirty {
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled(middle_text, git_style));

            // Add padding after git info
            let padding_after = total_width
                .saturating_sub(left_width + padding_before + middle_width + right_width);
            spans.push(Span::raw(" ".repeat(padding_after)));
        } else {
            // No git info, just add padding
            let padding = total_width.saturating_sub(left_width + right_width);
            spans.push(Span::raw(" ".repeat(padding)));
        }

        spans.push(Span::styled(
            right_text,
            Style::default().fg(theme.status_bar_fg),
        ));

        let status_line = Line::from(spans);

        let paragraph = Paragraph::new(status_line)
            .style(
                Style::default()
                    .bg(theme.status_bar_bg)
                    .fg(theme.status_bar_fg),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    fn draw_function_key_bar(frame: &mut Frame, area: Rect, theme: &cortex_core::Theme) {
        // Define function key actions - similar to Far Manager
        let function_keys = vec![
            ("F1", "Help"),
            ("F2", "Menu"),
            ("F3", "View"),
            ("F4", "Edit"),
            ("F5", "Copy"),
            ("F6", "Move"),
            ("F7", "MkDir"),
            ("F8", "Delete"),
            ("F9", "Config"),
            ("F10", "Quit"),
        ];

        let total_width = area.width as usize;
        let key_width = total_width / function_keys.len();

        let mut spans = Vec::new();

        for (i, (key, action)) in function_keys.iter().enumerate() {
            // Create styled key number (inverted colors for emphasis)
            let key_span = Span::styled(
                format!("{}", key),
                Style::default()
                    .bg(theme.active_border)  // Use active border color for better visibility
                    .fg(theme.background)      // Use background color for contrast
                    .add_modifier(Modifier::BOLD),
            );

            // Create action text with a space before it
            let action_span = Span::styled(
                format!(" {}", action),  // Add space before action text
                Style::default()
                    .fg(theme.status_bar_fg)
                    .bg(theme.panel_background), // Use panel background for subtle separation
            );

            spans.push(key_span);
            spans.push(action_span);

            // Add padding to distribute evenly
            if i < function_keys.len() - 1 {
                let padding_needed = key_width.saturating_sub(key.len() + action.len() + 1); // +1 for the added space
                spans.push(Span::styled(
                    " ".repeat(padding_needed),
                    Style::default().bg(theme.panel_background), // Continue panel background
                ));
            }
        }

        let function_key_line = Line::from(spans);

        let paragraph = Paragraph::new(function_key_line)
            .style(
                Style::default()
                    .bg(theme.panel_background)  // Use panel background instead of status bar bg
                    .fg(theme.status_bar_fg),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    fn draw_command_output(
        frame: &mut Frame,
        area: Rect,
        app: &AppState,
        theme: &cortex_core::Theme,
    ) {
        let title = if app.command_running {
            " Command Output (Running...) [Ctrl+C to cancel] ".to_string()
        } else {
            format!(
                " Command Output ({} lines) [O to toggle] ",
                app.command_output.len()
            )
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.active_border));

        // Convert command output to list items, showing most recent at bottom
        let available_height = area.height.saturating_sub(2) as usize; // Account for borders
        let total_lines = app.command_output.len();

        // Show only the lines that fit, starting from the most recent that fit
        let start_index = total_lines.saturating_sub(available_height);

        let output_lines: Vec<ListItem> = app
            .command_output
            .iter()
            .skip(start_index)
            .map(|line| {
                // Color-code different types of messages
                let style = if line.starts_with("[ERROR]") {
                    Style::default().fg(theme.error)
                } else if line.starts_with("[STARTED]") || line.starts_with("[COMPLETED]") {
                    Style::default().fg(theme.info).add_modifier(Modifier::BOLD)
                } else if line.starts_with("[WORKING DIR]") {
                    Style::default().fg(theme.dim_text)
                } else {
                    Style::default().fg(theme.normal_text)
                };

                ListItem::new(Line::from(vec![Span::raw(line.clone())])).style(style)
            })
            .collect();

        let list = List::new(output_lines).block(block);

        frame.render_widget(list, area);
    }

    fn draw_background(frame: &mut Frame, area: Rect, theme: &cortex_core::Theme) {
        // Fill the entire terminal with the theme's background color
        let background = Block::default().style(Style::default().bg(theme.background));
        frame.render_widget(background, area);
    }
}
