use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Gauge},
    Frame,
};
use cortex_core::{SearchCriteria, SearchResult, SearchProgress, SearchType};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SearchDialog {
    pub criteria: SearchCriteria,
    pub state: SearchState,
    pub results: Vec<SearchResult>,
    pub selected_result: usize,
    pub search_progress: Option<SearchProgressInfo>,
    pub input_field: InputField,
    pub show_options: bool,
}

#[derive(Debug, Clone)]
pub enum SearchState {
    Setup,      // Setting up search criteria
    Searching,  // Search in progress
    Results,    // Viewing results
}

#[derive(Debug, Clone)]
pub enum InputField {
    Pattern,
    Extensions,
    Directory,
}

#[derive(Debug, Clone)]
pub struct SearchProgressInfo {
    pub current_path: PathBuf,
    pub searched: usize,
    pub total: usize,
    pub found: usize,
}

impl SearchDialog {
    pub fn new() -> Self {
        Self {
            criteria: SearchCriteria {
                pattern: String::new(),
                search_type: SearchType::Wildcard,
                case_sensitive: false,
                search_in_files: false,
                include_hidden: false,
                include_subdirs: true,
                max_depth: None,
                file_extensions: Vec::new(),
                size_filter: None,
                date_filter: None,
            },
            state: SearchState::Setup,
            results: Vec::new(),
            selected_result: 0,
            search_progress: None,
            input_field: InputField::Pattern,
            show_options: false,
        }
    }
    
    pub fn render(&self, frame: &mut Frame) {
        match self.state {
            SearchState::Setup => self.render_setup(frame),
            SearchState::Searching => self.render_searching(frame),
            SearchState::Results => self.render_results(frame),
        }
    }
    
    fn render_setup(&self, frame: &mut Frame) {
        let area = centered_rect(70, 60, frame.area());
        frame.render_widget(Clear, area);
        
        let block = Block::default()
            .title(" Find Files (Alt+F7) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Pattern input
                Constraint::Length(3),  // Directory
                Constraint::Length(3),  // Extensions
                Constraint::Min(5),     // Options
                Constraint::Length(2),  // Help
            ])
            .split(inner);
        
        // Pattern input
        let pattern_style = if matches!(self.input_field, InputField::Pattern) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        
        let pattern_block = Block::default()
            .title(" Search Pattern ")
            .borders(Borders::ALL)
            .border_style(pattern_style);
        let pattern_inner = pattern_block.inner(chunks[0]);
        frame.render_widget(pattern_block, chunks[0]);
        
        let pattern_text = if self.criteria.pattern.is_empty() {
            "e.g., *.rs, file*.txt, or regex pattern"
        } else {
            &self.criteria.pattern
        };
        let pattern = Paragraph::new(pattern_text)
            .style(if self.criteria.pattern.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            });
        frame.render_widget(pattern, pattern_inner);
        
        // Directory
        let dir_style = if matches!(self.input_field, InputField::Directory) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        
        let dir_block = Block::default()
            .title(" Search In ")
            .borders(Borders::ALL)
            .border_style(dir_style);
        let dir_inner = dir_block.inner(chunks[1]);
        frame.render_widget(dir_block, chunks[1]);
        
        let dir_text = Paragraph::new("Current directory")
            .style(Style::default().fg(Color::White));
        frame.render_widget(dir_text, dir_inner);
        
        // Extensions filter
        let ext_style = if matches!(self.input_field, InputField::Extensions) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        
        let ext_block = Block::default()
            .title(" File Extensions (optional) ")
            .borders(Borders::ALL)
            .border_style(ext_style);
        let ext_inner = ext_block.inner(chunks[2]);
        frame.render_widget(ext_block, chunks[2]);
        
        let ext_text = if self.criteria.file_extensions.is_empty() {
            "e.g., rs,txt,md (comma-separated)"
        } else {
            &self.criteria.file_extensions.join(",")
        };
        let ext = Paragraph::new(ext_text)
            .style(if self.criteria.file_extensions.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            });
        frame.render_widget(ext, ext_inner);
        
        // Options
        let options_block = Block::default()
            .title(" Options ")
            .borders(Borders::ALL);
        let options_inner = options_block.inner(chunks[3]);
        frame.render_widget(options_block, chunks[3]);
        
        let mut options = vec![
            format!("[{}] Case sensitive", if self.criteria.case_sensitive { "×" } else { " " }),
            format!("[{}] Search in file contents", if self.criteria.search_in_files { "×" } else { " " }),
            format!("[{}] Include hidden files", if self.criteria.include_hidden { "×" } else { " " }),
            format!("[{}] Search subdirectories", if self.criteria.include_subdirs { "×" } else { " " }),
        ];
        
        match self.criteria.search_type {
            SearchType::Wildcard => options.push("Search type: Wildcard (*)".to_string()),
            SearchType::Regex => options.push("Search type: Regular Expression".to_string()),
            SearchType::Exact => options.push("Search type: Exact Match".to_string()),
            SearchType::Contains => options.push("Search type: Contains".to_string()),
        }
        
        let options_list = List::new(
            options.into_iter().map(|o| ListItem::new(o)).collect::<Vec<_>>()
        );
        frame.render_widget(options_list, options_inner);
        
        // Help
        let help_text = " Tab: Next field | Space: Toggle option | Enter: Start search | ESC: Cancel ";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[4]);
    }
    
    fn render_searching(&self, frame: &mut Frame) {
        let area = centered_rect(60, 30, frame.area());
        frame.render_widget(Clear, area);
        
        let block = Block::default()
            .title(" Searching... ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Current path
                Constraint::Length(2),  // Progress bar
                Constraint::Length(2),  // Stats
                Constraint::Min(1),     // Space
                Constraint::Length(1),  // Cancel hint
            ])
            .split(inner);
        
        if let Some(ref progress) = self.search_progress {
            // Current path
            let path_text = format!("Searching: {}", 
                progress.current_path.display());
            let path = Paragraph::new(path_text)
                .style(Style::default().fg(Color::White));
            frame.render_widget(path, chunks[0]);
            
            // Progress bar
            let percentage = if progress.total > 0 {
                (progress.searched as f64 / progress.total as f64 * 100.0) as u16
            } else {
                0
            };
            
            let progress_bar = Gauge::default()
                .gauge_style(Style::default().fg(Color::Green))
                .percent(percentage)
                .label(format!("{}%", percentage));
            frame.render_widget(progress_bar, chunks[1]);
            
            // Stats
            let stats = format!("Searched: {} / {} | Found: {} matches",
                progress.searched, progress.total, progress.found);
            let stats_text = Paragraph::new(stats)
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            frame.render_widget(stats_text, chunks[2]);
        }
        
        // Cancel hint
        let cancel = Paragraph::new("Press ESC to cancel")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(cancel, chunks[4]);
    }
    
    fn render_results(&self, frame: &mut Frame) {
        let area = centered_rect(90, 80, frame.area());
        frame.render_widget(Clear, area);
        
        let block = Block::default()
            .title(format!(" Search Results ({} found) ", self.results.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),    // Results list
                Constraint::Length(3),  // Preview
                Constraint::Length(1),  // Help
            ])
            .split(inner);
        
        // Results list
        let items: Vec<ListItem> = self.results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let is_selected = idx == self.selected_result;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let size = humansize::format_size(result.size, humansize::BINARY);
                let path = result.path.display().to_string();
                
                let content = if result.matches.is_empty() {
                    format!("{:<60} {:>10}", path, size)
                } else {
                    format!("{:<60} {:>10} ({} matches)", 
                        path, size, result.matches.len())
                };
                
                ListItem::new(Line::from(vec![
                    Span::styled(content, style)
                ]))
            })
            .collect();
        
        let list = List::new(items);
        frame.render_widget(list, chunks[0]);
        
        // Preview of selected result
        if let Some(result) = self.results.get(self.selected_result) {
            let preview_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            let preview_inner = preview_block.inner(chunks[1]);
            frame.render_widget(preview_block, chunks[1]);
            
            if !result.matches.is_empty() {
                if let Some(first_match) = result.matches.first() {
                    if let Some(ref content) = first_match.line_content {
                        let preview_text = format!("Line {}: {}", 
                            first_match.line_number.unwrap_or(0), content);
                        let preview = Paragraph::new(preview_text)
                            .style(Style::default().fg(Color::Yellow));
                        frame.render_widget(preview, preview_inner);
                    }
                }
            }
        }
        
        // Help
        let help = " ↑↓: Navigate | Enter: Go to file | F3: View | F4: Edit | ESC: Close | F7: New search ";
        let help_text = Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help_text, chunks[2]);
    }
    
    pub fn move_selection_up(&mut self) {
        if self.selected_result > 0 {
            self.selected_result -= 1;
        }
    }
    
    pub fn move_selection_down(&mut self) {
        if self.selected_result < self.results.len().saturating_sub(1) {
            self.selected_result += 1;
        }
    }
    
    pub fn get_selected_path(&self) -> Option<PathBuf> {
        self.results.get(self.selected_result)
            .map(|r| r.path.clone())
    }
    
    pub fn toggle_case_sensitive(&mut self) {
        self.criteria.case_sensitive = !self.criteria.case_sensitive;
    }
    
    pub fn toggle_search_in_files(&mut self) {
        self.criteria.search_in_files = !self.criteria.search_in_files;
    }
    
    pub fn toggle_include_hidden(&mut self) {
        self.criteria.include_hidden = !self.criteria.include_hidden;
    }
    
    pub fn toggle_include_subdirs(&mut self) {
        self.criteria.include_subdirs = !self.criteria.include_subdirs;
    }
    
    pub fn cycle_search_type(&mut self) {
        self.criteria.search_type = match self.criteria.search_type {
            SearchType::Wildcard => SearchType::Contains,
            SearchType::Contains => SearchType::Exact,
            SearchType::Exact => SearchType::Regex,
            SearchType::Regex => SearchType::Wildcard,
        };
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