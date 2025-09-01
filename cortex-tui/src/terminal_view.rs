use cortex_core::terminal::{TerminalManager, TerminalEvent, TerminalSize};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct TerminalTab {
    pub id: String,
    pub title: String,
    pub content: Vec<String>,
    pub cursor_pos: (u16, u16),
    pub is_active: bool,
}

pub struct TerminalView {
    pub manager: TerminalManager,
    pub tabs: Vec<TerminalTab>,
    pub active_tab: usize,
    pub event_receivers: HashMap<String, mpsc::UnboundedReceiver<TerminalEvent>>,
    pub split_mode: SplitMode,
    pub show_tabs: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitMode {
    Single,
    Horizontal,
    Vertical,
}

impl TerminalView {
    pub fn new() -> Self {
        Self {
            manager: TerminalManager::new(),
            tabs: Vec::new(),
            active_tab: 0,
            event_receivers: HashMap::new(),
            split_mode: SplitMode::Single,
            show_tabs: true,
        }
    }
    
    pub fn create_new_terminal(&mut self, title: Option<String>, working_dir: Option<std::path::PathBuf>) -> anyhow::Result<()> {
        let (rows, cols) = self.get_terminal_size();
        let size = TerminalSize { rows, cols };
        
        let (session_id, event_rx) = self.manager.create_session(
            title.clone(),
            working_dir,
            Some(size),
        )?;
        
        let tab = TerminalTab {
            id: session_id.clone(),
            title: title.unwrap_or_else(|| format!("Terminal {}", self.tabs.len() + 1)),
            content: Vec::new(),
            cursor_pos: (0, 0),
            is_active: false,
        };
        
        self.tabs.push(tab);
        self.event_receivers.insert(session_id, event_rx);
        self.active_tab = self.tabs.len() - 1;
        self.update_active_states();
        
        Ok(())
    }
    
    pub fn close_current_terminal(&mut self) -> anyhow::Result<()> {
        if self.tabs.is_empty() {
            return Ok(());
        }
        
        let tab = &self.tabs[self.active_tab];
        self.manager.close_session(&tab.id)?;
        self.event_receivers.remove(&tab.id);
        self.tabs.remove(self.active_tab);
        
        if self.active_tab >= self.tabs.len() && self.active_tab > 0 {
            self.active_tab -= 1;
        }
        
        self.update_active_states();
        Ok(())
    }
    
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            self.update_active_states();
        }
    }
    
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.update_active_states();
        }
    }
    
    fn update_active_states(&mut self) {
        for (i, tab) in self.tabs.iter_mut().enumerate() {
            tab.is_active = i == self.active_tab;
        }
    }
    
    pub fn send_input(&mut self, input: &str) -> anyhow::Result<()> {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            self.manager.write_to_session(&tab.id, input.as_bytes())?;
        }
        Ok(())
    }
    
    pub fn send_key(&mut self, key: char) -> anyhow::Result<()> {
        self.send_input(&key.to_string())
    }
    
    pub fn process_events(&mut self) {
        let mut updates = Vec::new();
        
        for (session_id, rx) in &mut self.event_receivers {
            while let Ok(event) = rx.try_recv() {
                updates.push((session_id.clone(), event));
            }
        }
        
        for (session_id, event) in updates {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == session_id) {
                match event {
                    TerminalEvent::Output(_) => {
                        // Update content from terminal
                        if let Ok(content) = self.manager.get_screen_content(&session_id) {
                            tab.content = content;
                        }
                        if let Ok(cursor) = self.manager.get_cursor_position(&session_id) {
                            tab.cursor_pos = cursor;
                        }
                    }
                    TerminalEvent::Resize(_) => {
                        // Handle resize if needed
                    }
                    TerminalEvent::Exit(_) => {
                        // Mark terminal as exited
                        tab.title.push_str(" [exited]");
                    }
                    TerminalEvent::Error(e) => {
                        tab.content.push(format!("Error: {}", e));
                    }
                }
            }
        }
    }
    
    pub fn resize(&mut self, rows: u16, cols: u16) -> anyhow::Result<()> {
        for tab in &self.tabs {
            self.manager.resize_session(&tab.id, rows, cols)?;
        }
        Ok(())
    }
    
    fn get_terminal_size(&self) -> (u16, u16) {
        // Default size, will be updated based on actual UI area
        (24, 80)
    }
    
    pub fn toggle_split_mode(&mut self) {
        self.split_mode = match self.split_mode {
            SplitMode::Single => SplitMode::Horizontal,
            SplitMode::Horizontal => SplitMode::Vertical,
            SplitMode::Vertical => SplitMode::Single,
        };
    }
}

pub fn draw_terminal_view(frame: &mut Frame, area: Rect, view: &TerminalView, theme: &cortex_core::Theme) {
    let chunks = if view.show_tabs && view.tabs.len() > 1 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area)
    } else {
        // Create a compatible layout result
        let mut result = Vec::new();
        result.push(Rect::default());
        result.push(area);
        result.into()
    };
    
    // Draw tabs if needed
    if view.show_tabs && view.tabs.len() > 1 {
        let tab_titles: Vec<String> = view.tabs.iter().map(|t| t.title.clone()).collect();
        let tabs = Tabs::new(tab_titles)
            .select(view.active_tab)
            .style(Style::default().fg(theme.normal_text))
            .highlight_style(Style::default().fg(theme.info).add_modifier(Modifier::BOLD));
        frame.render_widget(tabs, chunks[0]);
    }
    
    let terminal_area = if view.show_tabs && view.tabs.len() > 1 {
        chunks[1]
    } else {
        area
    };
    
    // Draw terminal content based on split mode
    match view.split_mode {
        SplitMode::Single => {
            if let Some(tab) = view.tabs.get(view.active_tab) {
                draw_terminal_content(frame, terminal_area, tab, theme);
            }
        }
        SplitMode::Horizontal => {
            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(terminal_area);
            
            if let Some(tab) = view.tabs.get(view.active_tab) {
                draw_terminal_content(frame, split[0], tab, theme);
            }
            
            if view.tabs.len() > 1 {
                let next_idx = (view.active_tab + 1) % view.tabs.len();
                if let Some(tab) = view.tabs.get(next_idx) {
                    draw_terminal_content(frame, split[1], tab, theme);
                }
            }
        }
        SplitMode::Vertical => {
            let split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(terminal_area);
            
            if let Some(tab) = view.tabs.get(view.active_tab) {
                draw_terminal_content(frame, split[0], tab, theme);
            }
            
            if view.tabs.len() > 1 {
                let next_idx = (view.active_tab + 1) % view.tabs.len();
                if let Some(tab) = view.tabs.get(next_idx) {
                    draw_terminal_content(frame, split[1], tab, theme);
                }
            }
        }
    }
}

fn draw_terminal_content(frame: &mut Frame, area: Rect, tab: &TerminalTab, theme: &cortex_core::Theme) {
    let lines: Vec<Line> = tab.content.iter()
        .map(|line| Line::from(Span::raw(line)))
        .collect();
    
    let terminal = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" {} ", tab.title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if tab.is_active { theme.active_border } else { theme.inactive_border }
                ))
        )
        .style(Style::default()
            .fg(theme.normal_text)
            .bg(Color::Black));
    
    frame.render_widget(terminal, area);
    
    // Set cursor position for active terminal
    if tab.is_active && area.height > 2 && area.width > 2 {
        let cursor_x = area.x + 1 + tab.cursor_pos.1.min(area.width - 2);
        let cursor_y = area.y + 1 + tab.cursor_pos.0.min(area.height - 2);
        frame.set_cursor(cursor_x, cursor_y);
    }
}