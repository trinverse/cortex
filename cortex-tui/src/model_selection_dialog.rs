use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Tabs},
    Frame,
};
use cortex_core::ai::embedded::{ModelInfo, ModelTier};

#[derive(Debug, Clone)]
pub struct ModelSelectionDialog {
    pub models: Vec<ModelInfo>,
    pub selected_index: usize,
    pub tab_index: usize,  // 0: Free/Embedded, 1: Premium/Cloud
    pub downloaded_models: Vec<String>,
    pub downloading: Option<(String, f64)>,  // (model_id, progress)
}

impl ModelSelectionDialog {
    pub fn new(models: Vec<ModelInfo>, downloaded: Vec<String>) -> Self {
        Self {
            models,
            selected_index: 0,
            tab_index: 0,
            downloaded_models: downloaded,
            downloading: None,
        }
    }
    
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    pub fn move_down(&mut self) {
        let filtered = self.get_filtered_models();
        if self.selected_index < filtered.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
    
    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % 2;
        self.selected_index = 0;
    }
    
    pub fn prev_tab(&mut self) {
        if self.tab_index == 0 {
            self.tab_index = 1;
        } else {
            self.tab_index = 0;
        }
        self.selected_index = 0;
    }
    
    pub fn get_filtered_models(&self) -> Vec<&ModelInfo> {
        self.models
            .iter()
            .filter(|m| {
                if self.tab_index == 0 {
                    m.tier == ModelTier::Free
                } else {
                    m.tier == ModelTier::Premium
                }
            })
            .collect()
    }
    
    pub fn get_selected_model(&self) -> Option<&ModelInfo> {
        self.get_filtered_models().get(self.selected_index).copied()
    }
    
    pub fn is_model_downloaded(&self, model_id: &str) -> bool {
        self.downloaded_models.contains(&model_id.to_string())
    }
    
    pub fn set_downloading(&mut self, model_id: String, progress: f64) {
        self.downloading = Some((model_id, progress));
    }
    
    pub fn clear_downloading(&mut self) {
        self.downloading = None;
    }
}

pub fn draw_model_selection_dialog(frame: &mut Frame, dialog: &ModelSelectionDialog, theme: &cortex_core::Theme) {
    let size = frame.area();
    
    // Calculate dialog size (90% width, 80% height for more space)
    let dialog_width = size.width * 9 / 10;
    let dialog_height = size.height * 4 / 5;
    
    let dialog_area = Rect::new(
        (size.width - dialog_width) / 2,
        (size.height - dialog_height) / 2,
        dialog_width,
        dialog_height,
    );
    
    // Clear the area
    frame.render_widget(Clear, dialog_area);
    
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(5),     // Model list
            Constraint::Length(6),  // Model details
        ])
        .split(dialog_area);
    
    // Draw tabs
    let tab_titles = vec!["Free Models (Embedded)", "Premium Models (Cloud)"];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" AI Model Selection "))
        .style(Style::default().fg(theme.normal_text))
        .highlight_style(Style::default().fg(theme.selected_fg).add_modifier(Modifier::BOLD))
        .select(dialog.tab_index);
    frame.render_widget(tabs, chunks[0]);
    
    // Draw model list
    let filtered_models = dialog.get_filtered_models();
    let items: Vec<ListItem> = filtered_models
        .iter()
        .map(|model| {
            let status = if dialog.is_model_downloaded(&model.id) {
                " [Downloaded]"
            } else if let Some((downloading_id, _)) = &dialog.downloading {
                if downloading_id == &model.id {
                    " [Downloading...]"
                } else {
                    ""
                }
            } else {
                ""
            };
            
            let size_str = if model.size_bytes > 0 {
                format!(" ({})", humansize::format_size(model.size_bytes, humansize::BINARY))
            } else {
                String::new()
            };
            
            let line = format!("{}{}{}", model.name, size_str, status);
            
            let style = if dialog.is_model_downloaded(&model.id) {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.normal_text)
            };
            
            ListItem::new(line).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Available Models (↑↓ navigate, Enter to select, Tab to switch) ")
        )
        .highlight_style(
            Style::default()
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("> ");
    
    frame.render_stateful_widget(list, chunks[1], &mut dialog.selected_index.clone().into());
    
    // Draw model details
    if let Some(model) = dialog.get_selected_model() {
        let mut details = vec![
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(theme.dim_text)),
                Span::raw(&model.description),
            ]),
            Line::from(vec![
                Span::styled("Capabilities: ", Style::default().fg(theme.dim_text)),
                Span::raw(model.capabilities.join(", ")),
            ]),
        ];
        
        if model.tier == ModelTier::Free {
            details.push(Line::from(vec![
                Span::styled("RAM Required: ", Style::default().fg(theme.dim_text)),
                Span::raw(humansize::format_size(model.ram_required, humansize::BINARY)),
            ]));
            
            details.push(Line::from(vec![
                Span::styled("Quantization: ", Style::default().fg(theme.dim_text)),
                Span::raw(&model.quantization),
            ]));
        } else {
            details.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.dim_text)),
                Span::styled("Requires API Key", Style::default().fg(theme.warning)),
            ]));
        }
        
        // Show download progress if downloading
        if let Some((downloading_id, progress)) = &dialog.downloading {
            if downloading_id == &model.id {
                let gauge = Gauge::default()
                    .block(Block::default().title("Download Progress"))
                    .gauge_style(Style::default().fg(theme.info))
                    .percent((*progress * 100.0) as u16);
                
                // Split details area for progress
                let detail_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(3),
                        Constraint::Length(3),
                    ])
                    .split(chunks[2]);
                
                let paragraph = Paragraph::new(details)
                    .block(Block::default().borders(Borders::ALL).title(" Model Details "));
                frame.render_widget(paragraph, detail_chunks[0]);
                frame.render_widget(gauge, detail_chunks[1]);
                
                return;
            }
        }
        
        let paragraph = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Model Details "));
        frame.render_widget(paragraph, chunks[2]);
    }
}