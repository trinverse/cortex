use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
    Gruvbox,
    Nord,
    Modern,
    Random,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    // Background colors
    pub background: Color,       // Main terminal background
    pub panel_background: Color, // Panel area background

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
            background: Color::Rgb(29, 32, 33), // Very dark gray background
            panel_background: Color::Rgb(33, 36, 37), // Slightly lighter for panels
            active_border: Color::Rgb(139, 233, 253), // Bright cyan
            inactive_border: Color::Rgb(68, 71, 90), // Muted gray

            directory: Color::Rgb(189, 147, 249), // Soft purple (instead of light blue)
            symlink: Color::Rgb(139, 233, 253),   // Cyan
            executable: Color::Rgb(80, 250, 123), // Mint green
            regular_file: Color::Rgb(248, 248, 242), // Off-white
            source_code: Color::Rgb(139, 233, 253), // Cyan for code files
            document: Color::Rgb(241, 250, 140),  // Soft yellow
            image: Color::Rgb(255, 121, 198),     // Pink
            archive: Color::Rgb(255, 184, 108),   // Soft orange (instead of red)
            hidden: Color::Rgb(98, 114, 164),     // Muted blue-gray

            selected_bg: Color::Rgb(68, 71, 90), // Dracula-style selection
            selected_fg: Color::Rgb(248, 248, 242), // Off-white
            inactive_selected_bg: Color::Rgb(44, 47, 62),
            marked: Color::Rgb(241, 250, 140), // Soft yellow

            status_bar_bg: Color::Rgb(40, 42, 54), // Dark purple-gray
            status_bar_fg: Color::Rgb(248, 248, 242), // Off-white
            command_line_bg: Color::Rgb(33, 34, 44), // Slightly lighter than black
            command_line_fg: Color::Rgb(248, 248, 242),

            error: Color::Rgb(255, 85, 85),     // Soft red
            warning: Color::Rgb(255, 184, 108), // Orange
            success: Color::Rgb(80, 250, 123),  // Mint green
            info: Color::Rgb(139, 233, 253),    // Cyan

            normal_text: Color::Rgb(248, 248, 242), // Off-white
            dim_text: Color::Rgb(98, 114, 164),     // Muted blue-gray
            highlight_text: Color::Rgb(139, 233, 253), // Cyan
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            background: Color::Rgb(255, 255, 255), // Pure white background
            panel_background: Color::Rgb(250, 251, 252), // Very light gray for panels
            active_border: Color::Rgb(36, 41, 47), // Dark border for active
            inactive_border: Color::Rgb(208, 215, 222), // Light gray for inactive

            directory: Color::Rgb(7, 54, 110), // Dark blue for directories
            symlink: Color::Rgb(0, 91, 93),    // Dark teal for symlinks
            executable: Color::Rgb(22, 98, 22), // Dark green
            regular_file: Color::Rgb(36, 41, 47), // Dark gray
            source_code: Color::Rgb(0, 48, 110), // Dark blue for code
            document: Color::Rgb(133, 65, 0),  // Dark orange for documents
            image: Color::Rgb(102, 31, 117),   // Dark purple for images
            archive: Color::Rgb(157, 42, 53),  // Dark red for archives
            hidden: Color::Rgb(88, 96, 105),   // Medium gray for hidden

            selected_bg: Color::Rgb(210, 227, 252), // Light blue background
            selected_fg: Color::Rgb(24, 28, 33),    // Very dark text on selection
            inactive_selected_bg: Color::Rgb(234, 238, 242), // Light gray background
            marked: Color::Rgb(255, 235, 180),      // Light yellow background for marked

            status_bar_bg: Color::Rgb(246, 248, 250), // Light gray background
            status_bar_fg: Color::Rgb(36, 41, 47),    // Dark text
            command_line_bg: Color::Rgb(255, 255, 255), // White background
            command_line_fg: Color::Rgb(24, 28, 33),  // Very dark text

            error: Color::Rgb(157, 42, 53),   // Dark red
            warning: Color::Rgb(133, 77, 14), // Dark amber
            success: Color::Rgb(28, 117, 48), // Dark green
            info: Color::Rgb(0, 71, 122),     // Dark blue

            normal_text: Color::Rgb(24, 28, 33), // Very dark gray (almost black)
            dim_text: Color::Rgb(88, 96, 105),   // Medium gray
            highlight_text: Color::Rgb(0, 48, 110), // Dark blue for highlights
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            mode: ThemeMode::Gruvbox,
            background: Color::Rgb(29, 32, 33), // Gruvbox dark background
            panel_background: Color::Rgb(40, 40, 40), // Gruvbox dark gray
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
            background: Color::Rgb(46, 52, 64), // Nord Polar Night background
            panel_background: Color::Rgb(59, 66, 82), // Nord Polar Night lighter
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

    pub fn modern() -> Self {
        Self {
            mode: ThemeMode::Modern,
            background: Color::Rgb(24, 26, 33), // Modern dark blue background
            panel_background: Color::Rgb(30, 33, 41), // Slightly lighter
            active_border: Color::Rgb(100, 255, 218), // Mint accent
            inactive_border: Color::Rgb(61, 68, 85), // Subtle gray

            directory: Color::Rgb(130, 170, 255),    // Soft blue
            symlink: Color::Rgb(100, 255, 218),      // Mint
            executable: Color::Rgb(134, 239, 172),   // Lime green
            regular_file: Color::Rgb(199, 210, 254), // Light lavender
            source_code: Color::Rgb(129, 140, 248),  // Indigo
            document: Color::Rgb(253, 224, 71),      // Yellow
            image: Color::Rgb(244, 114, 182),        // Pink
            archive: Color::Rgb(251, 146, 60),       // Orange
            hidden: Color::Rgb(100, 116, 139),       // Slate

            selected_bg: Color::Rgb(56, 58, 89),    // Deep indigo
            selected_fg: Color::Rgb(241, 245, 249), // Near white
            inactive_selected_bg: Color::Rgb(39, 46, 58),
            marked: Color::Rgb(253, 224, 71), // Yellow

            status_bar_bg: Color::Rgb(30, 33, 41), // Very dark blue-gray
            status_bar_fg: Color::Rgb(199, 210, 254), // Light lavender
            command_line_bg: Color::Rgb(24, 26, 33), // Almost black with blue tint
            command_line_fg: Color::Rgb(199, 210, 254),

            error: Color::Rgb(248, 113, 113),   // Soft red
            warning: Color::Rgb(251, 191, 36),  // Amber
            success: Color::Rgb(134, 239, 172), // Lime
            info: Color::Rgb(100, 255, 218),    // Mint

            normal_text: Color::Rgb(199, 210, 254), // Light lavender
            dim_text: Color::Rgb(100, 116, 139),    // Slate
            highlight_text: Color::Rgb(100, 255, 218), // Mint
        }
    }

    pub fn get_file_style(&self, file_type: &crate::FileType, extension: Option<&String>) -> Style {
        let color = match file_type {
            crate::FileType::Directory => self.directory,
            crate::FileType::Symlink => self.symlink,
            crate::FileType::File => {
                if let Some(ext) = extension {
                    match ext.as_str() {
                        "rs" | "go" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "hpp"
                        | "cs" | "swift" | "kt" | "scala" | "rb" | "php" | "lua" | "sh"
                        | "bash" => self.source_code,
                        "md" | "txt" | "doc" | "docx" | "pdf" | "odt" | "rtf" => self.document,
                        "jpg" | "jpeg" | "png" | "gif" | "svg" | "bmp" | "ico" | "webp" => {
                            self.image
                        }
                        "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" | "deb" | "rpm" => {
                            self.archive
                        }
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
            Theme::modern(),
        ];

        let current_theme = match mode {
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::Light => Theme::light(),
            ThemeMode::Gruvbox => Theme::gruvbox(),
            ThemeMode::Nord => Theme::nord(),
            ThemeMode::Modern => Theme::modern(),
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
            ThemeMode::Modern => Theme::modern(),
            ThemeMode::Random => {
                self.last_rotation = Instant::now();
                self.themes[self.current_index].clone()
            }
        };
    }

    pub fn update(&mut self) {
        if self.current_theme.mode == ThemeMode::Random
            && self.last_rotation.elapsed() >= self.rotation_interval
        {
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
