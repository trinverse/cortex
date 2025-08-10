use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
    Gruvbox,
    Nord,
    Random,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    // Panel colors
    pub active_border: Color,
    pub inactive_border: Color,
    
    // File type colors
    pub directory: Color,
    pub symlink: Color,
    pub executable: Color,
    pub regular_file: Color,
    pub source_code: Color,
    pub document: Color,
    pub image: Color,
    pub archive: Color,
    pub hidden: Color,
    
    // Selection colors
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub inactive_selected_bg: Color,
    pub marked: Color,
    
    // UI colors
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub command_line_bg: Color,
    pub command_line_fg: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    
    // Text colors
    pub normal_text: Color,
    pub dim_text: Color,
    pub highlight_text: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            active_border: Color::Cyan,
            inactive_border: Color::Gray,
            
            directory: Color::LightBlue,
            symlink: Color::Cyan,
            executable: Color::LightGreen,
            regular_file: Color::White,
            source_code: Color::Green,
            document: Color::Yellow,
            image: Color::Magenta,
            archive: Color::Red,
            hidden: Color::DarkGray,
            
            selected_bg: Color::Rgb(40, 44, 52),
            selected_fg: Color::White,
            inactive_selected_bg: Color::Rgb(30, 30, 30),
            marked: Color::Yellow,
            
            status_bar_bg: Color::Rgb(33, 33, 33),
            status_bar_fg: Color::White,
            command_line_bg: Color::Black,
            command_line_fg: Color::White,
            
            error: Color::Red,
            warning: Color::Yellow,
            success: Color::Green,
            info: Color::Blue,
            
            normal_text: Color::White,
            dim_text: Color::Gray,
            highlight_text: Color::Cyan,
        }
    }
    
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            active_border: Color::Blue,
            inactive_border: Color::Gray,
            
            directory: Color::Blue,
            symlink: Color::Cyan,
            executable: Color::Green,
            regular_file: Color::Black,
            source_code: Color::Rgb(0, 128, 0),
            document: Color::Rgb(184, 134, 11),
            image: Color::Magenta,
            archive: Color::Red,
            hidden: Color::Gray,
            
            selected_bg: Color::Rgb(200, 200, 255),
            selected_fg: Color::Black,
            inactive_selected_bg: Color::Rgb(230, 230, 230),
            marked: Color::Rgb(255, 200, 0),
            
            status_bar_bg: Color::Rgb(240, 240, 240),
            status_bar_fg: Color::Black,
            command_line_bg: Color::White,
            command_line_fg: Color::Black,
            
            error: Color::Red,
            warning: Color::Rgb(255, 140, 0),
            success: Color::Green,
            info: Color::Blue,
            
            normal_text: Color::Black,
            dim_text: Color::Gray,
            highlight_text: Color::Blue,
        }
    }
    
    pub fn gruvbox() -> Self {
        Self {
            mode: ThemeMode::Gruvbox,
            active_border: Color::Rgb(251, 184, 108),
            inactive_border: Color::Rgb(124, 111, 100),
            
            directory: Color::Rgb(131, 165, 152),
            symlink: Color::Rgb(142, 192, 124),
            executable: Color::Rgb(184, 187, 38),
            regular_file: Color::Rgb(235, 219, 178),
            source_code: Color::Rgb(142, 192, 124),
            document: Color::Rgb(251, 184, 108),
            image: Color::Rgb(211, 134, 155),
            archive: Color::Rgb(204, 36, 29),
            hidden: Color::Rgb(124, 111, 100),
            
            selected_bg: Color::Rgb(60, 56, 54),
            selected_fg: Color::Rgb(251, 241, 199),
            inactive_selected_bg: Color::Rgb(50, 48, 47),
            marked: Color::Rgb(250, 189, 47),
            
            status_bar_bg: Color::Rgb(40, 40, 40),
            status_bar_fg: Color::Rgb(235, 219, 178),
            command_line_bg: Color::Rgb(29, 32, 33),
            command_line_fg: Color::Rgb(235, 219, 178),
            
            error: Color::Rgb(204, 36, 29),
            warning: Color::Rgb(250, 189, 47),
            success: Color::Rgb(152, 151, 26),
            info: Color::Rgb(69, 133, 136),
            
            normal_text: Color::Rgb(235, 219, 178),
            dim_text: Color::Rgb(124, 111, 100),
            highlight_text: Color::Rgb(251, 184, 108),
        }
    }
    
    pub fn nord() -> Self {
        Self {
            mode: ThemeMode::Nord,
            active_border: Color::Rgb(136, 192, 208),
            inactive_border: Color::Rgb(76, 86, 106),
            
            directory: Color::Rgb(136, 192, 208),
            symlink: Color::Rgb(143, 188, 187),
            executable: Color::Rgb(163, 190, 140),
            regular_file: Color::Rgb(216, 222, 233),
            source_code: Color::Rgb(163, 190, 140),
            document: Color::Rgb(235, 203, 139),
            image: Color::Rgb(180, 142, 173),
            archive: Color::Rgb(191, 97, 106),
            hidden: Color::Rgb(76, 86, 106),
            
            selected_bg: Color::Rgb(67, 76, 94),
            selected_fg: Color::Rgb(236, 239, 244),
            inactive_selected_bg: Color::Rgb(59, 66, 82),
            marked: Color::Rgb(235, 203, 139),
            
            status_bar_bg: Color::Rgb(46, 52, 64),
            status_bar_fg: Color::Rgb(216, 222, 233),
            command_line_bg: Color::Rgb(46, 52, 64),
            command_line_fg: Color::Rgb(216, 222, 233),
            
            error: Color::Rgb(191, 97, 106),
            warning: Color::Rgb(235, 203, 139),
            success: Color::Rgb(163, 190, 140),
            info: Color::Rgb(129, 161, 193),
            
            normal_text: Color::Rgb(216, 222, 233),
            dim_text: Color::Rgb(76, 86, 106),
            highlight_text: Color::Rgb(136, 192, 208),
        }
    }
    
    pub fn get_file_style(&self, file_type: &crate::FileType, extension: Option<&String>) -> Style {
        let color = match file_type {
            crate::FileType::Directory => self.directory,
            crate::FileType::Symlink => self.symlink,
            crate::FileType::File => {
                if let Some(ext) = extension {
                    match ext.as_str() {
                        "rs" | "go" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "hpp" |
                        "cs" | "swift" | "kt" | "scala" | "rb" | "php" | "lua" | "sh" | "bash" => {
                            self.source_code
                        }
                        "md" | "txt" | "doc" | "docx" | "pdf" | "odt" | "rtf" => self.document,
                        "jpg" | "jpeg" | "png" | "gif" | "svg" | "bmp" | "ico" | "webp" => self.image,
                        "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" | "deb" | "rpm" => self.archive,
                        _ => self.regular_file,
                    }
                } else {
                    self.regular_file
                }
            }
            crate::FileType::Other => self.dim_text,
        };
        
        Style::default().fg(color)
    }
    
    pub fn get_selected_style(&self, is_active: bool) -> Style {
        if is_active {
            Style::default()
                .bg(self.selected_bg)
                .fg(self.selected_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(self.inactive_selected_bg)
                .fg(self.selected_fg)
        }
    }
    
    pub fn get_marked_style(&self) -> Style {
        Style::default()
            .fg(self.marked)
            .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
    }
    
    pub fn get_border_style(&self, is_active: bool) -> Style {
        if is_active {
            Style::default()
                .fg(self.active_border)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.inactive_border)
        }
    }
}

pub struct ThemeManager {
    current_theme: Theme,
    themes: Vec<Theme>,
    last_rotation: Instant,
    rotation_interval: Duration,
    current_index: usize,
}

impl ThemeManager {
    pub fn new(mode: ThemeMode) -> Self {
        let themes = vec![
            Theme::dark(),
            Theme::light(),
            Theme::gruvbox(),
            Theme::nord(),
        ];
        
        let current_theme = match mode {
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::Light => Theme::light(),
            ThemeMode::Gruvbox => Theme::gruvbox(),
            ThemeMode::Nord => Theme::nord(),
            ThemeMode::Random => themes[0].clone(),
        };
        
        Self {
            current_theme,
            themes,
            last_rotation: Instant::now(),
            rotation_interval: Duration::from_secs(600), // 10 minutes
            current_index: 0,
        }
    }
    
    pub fn get_current_theme(&self) -> &Theme {
        &self.current_theme
    }
    
    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.current_theme = match mode {
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::Light => Theme::light(),
            ThemeMode::Gruvbox => Theme::gruvbox(),
            ThemeMode::Nord => Theme::nord(),
            ThemeMode::Random => {
                self.last_rotation = Instant::now();
                self.themes[self.current_index].clone()
            }
        };
    }
    
    pub fn update(&mut self) {
        if self.current_theme.mode == ThemeMode::Random && self.last_rotation.elapsed() >= self.rotation_interval {
            self.current_index = (self.current_index + 1) % self.themes.len();
            self.current_theme = self.themes[self.current_index].clone();
            self.last_rotation = Instant::now();
        }
    }
    
    pub fn next_theme(&mut self) {
        self.current_index = (self.current_index + 1) % self.themes.len();
        self.current_theme = self.themes[self.current_index].clone();
    }
    
    pub fn previous_theme(&mut self) {
        if self.current_index == 0 {
            self.current_index = self.themes.len() - 1;
        } else {
            self.current_index -= 1;
        }
        self.current_theme = self.themes[self.current_index].clone();
    }
}