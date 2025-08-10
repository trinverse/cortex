use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::time::{Duration, Instant};

/// Mouse action types
#[derive(Debug, Clone, PartialEq)]
pub enum MouseAction {
    Click(Position),
    DoubleClick(Position),
    RightClick(Position),
    MiddleClick(Position),
    Drag { from: Position, to: Position },
    ScrollUp(Position),
    ScrollDown(Position),
    Hover(Position),
}

/// Position in the terminal
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Check if position is within a rectangle
    pub fn is_within(&self, rect: &Rect) -> bool {
        self.x >= rect.x
            && self.x < rect.x + rect.width
            && self.y >= rect.y
            && self.y < rect.y + rect.height
    }

    /// Get relative position within a rectangle
    pub fn relative_to(&self, rect: &Rect) -> Option<Position> {
        if self.is_within(rect) {
            Some(Position {
                x: self.x - rect.x,
                y: self.y - rect.y,
            })
        } else {
            None
        }
    }
}

/// Mouse handler for detecting mouse actions
pub struct MouseHandler {
    last_click: Option<(Position, Instant)>,
    double_click_threshold: Duration,
    drag_start: Option<Position>,
    is_dragging: bool,
}

impl Default for MouseHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            last_click: None,
            double_click_threshold: Duration::from_millis(500),
            drag_start: None,
            is_dragging: false,
        }
    }

    /// Process a mouse event and return the corresponding action
    pub fn process_event(&mut self, event: MouseEvent) -> Option<MouseAction> {
        let pos = Position::new(event.column, event.row);

        match event.kind {
            MouseEventKind::Down(button) => self.handle_mouse_down(button, pos),
            MouseEventKind::Up(button) => self.handle_mouse_up(button, pos),
            MouseEventKind::Drag(button) => self.handle_mouse_drag(button, pos),
            MouseEventKind::Moved => Some(MouseAction::Hover(pos)),
            MouseEventKind::ScrollDown => Some(MouseAction::ScrollDown(pos)),
            MouseEventKind::ScrollUp => Some(MouseAction::ScrollUp(pos)),
            _ => None,
        }
    }

    fn handle_mouse_down(&mut self, button: MouseButton, pos: Position) -> Option<MouseAction> {
        match button {
            MouseButton::Left => {
                self.drag_start = Some(pos);
                None // Don't emit action on down, wait for up
            }
            MouseButton::Right => {
                None // Wait for up to emit right click
            }
            MouseButton::Middle => {
                None // Wait for up to emit middle click
            }
        }
    }

    fn handle_mouse_up(&mut self, button: MouseButton, pos: Position) -> Option<MouseAction> {
        match button {
            MouseButton::Left => {
                if self.is_dragging {
                    // End of drag
                    self.is_dragging = false;
                    if let Some(start) = self.drag_start {
                        self.drag_start = None;
                        return Some(MouseAction::Drag {
                            from: start,
                            to: pos,
                        });
                    }
                }

                // Check for double click
                if let Some((last_pos, last_time)) = self.last_click {
                    if last_pos == pos && last_time.elapsed() < self.double_click_threshold {
                        self.last_click = None;
                        return Some(MouseAction::DoubleClick(pos));
                    }
                }

                // Single click
                self.last_click = Some((pos, Instant::now()));
                self.drag_start = None;
                Some(MouseAction::Click(pos))
            }
            MouseButton::Right => Some(MouseAction::RightClick(pos)),
            MouseButton::Middle => Some(MouseAction::MiddleClick(pos)),
        }
    }

    fn handle_mouse_drag(&mut self, _button: MouseButton, pos: Position) -> Option<MouseAction> {
        if let Some(start) = self.drag_start {
            if !self.is_dragging && (pos.x != start.x || pos.y != start.y) {
                self.is_dragging = true;
            }

            if self.is_dragging {
                return Some(MouseAction::Drag {
                    from: start,
                    to: pos,
                });
            }
        }
        None
    }

    /// Reset the handler state
    pub fn reset(&mut self) {
        self.last_click = None;
        self.drag_start = None;
        self.is_dragging = false;
    }
}

/// Context menu that appears on right-click
#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub position: Position,
    pub items: Vec<ContextMenuItem>,
    pub selected_index: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub action: ContextMenuAction,
    pub enabled: bool,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuAction {
    Copy,
    Cut,
    Paste,
    Delete,
    Rename,
    Properties,
    NewFile,
    NewFolder,
    Open,
    OpenWith,
    ViewFile,
    EditFile,
    Refresh,
    SelectAll,
    InvertSelection,
}

impl ContextMenu {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            items: Vec::new(),
            selected_index: 0,
            visible: false,
        }
    }

    /// Create a file context menu
    pub fn file_menu(position: Position, has_selection: bool) -> Self {
        let mut menu = Self::new(position);

        menu.items = vec![
            ContextMenuItem {
                label: "Open".to_string(),
                action: ContextMenuAction::Open,
                enabled: true,
                shortcut: Some("Enter".to_string()),
            },
            ContextMenuItem {
                label: "Open With...".to_string(),
                action: ContextMenuAction::OpenWith,
                enabled: true,
                shortcut: None,
            },
            ContextMenuItem {
                label: "View".to_string(),
                action: ContextMenuAction::ViewFile,
                enabled: true,
                shortcut: Some("F3".to_string()),
            },
            ContextMenuItem {
                label: "Edit".to_string(),
                action: ContextMenuAction::EditFile,
                enabled: true,
                shortcut: Some("F4".to_string()),
            },
            ContextMenuItem {
                label: "─────────".to_string(),
                action: ContextMenuAction::Copy, // Dummy action for separator
                enabled: false,
                shortcut: None,
            },
            ContextMenuItem {
                label: "Copy".to_string(),
                action: ContextMenuAction::Copy,
                enabled: true,
                shortcut: Some("F5".to_string()),
            },
            ContextMenuItem {
                label: "Cut".to_string(),
                action: ContextMenuAction::Cut,
                enabled: true,
                shortcut: Some("F6".to_string()),
            },
            ContextMenuItem {
                label: "Delete".to_string(),
                action: ContextMenuAction::Delete,
                enabled: true,
                shortcut: Some("F8".to_string()),
            },
            ContextMenuItem {
                label: "Rename".to_string(),
                action: ContextMenuAction::Rename,
                enabled: !has_selection,
                shortcut: Some("Shift+F6".to_string()),
            },
            ContextMenuItem {
                label: "─────────".to_string(),
                action: ContextMenuAction::Copy, // Dummy action for separator
                enabled: false,
                shortcut: None,
            },
            ContextMenuItem {
                label: "Properties".to_string(),
                action: ContextMenuAction::Properties,
                enabled: true,
                shortcut: Some("Alt+Enter".to_string()),
            },
        ];

        menu.visible = true;
        menu
    }

    /// Create a panel context menu (right-click on empty space)
    pub fn panel_menu(position: Position) -> Self {
        let mut menu = Self::new(position);

        menu.items = vec![
            ContextMenuItem {
                label: "New File".to_string(),
                action: ContextMenuAction::NewFile,
                enabled: true,
                shortcut: Some("Shift+F4".to_string()),
            },
            ContextMenuItem {
                label: "New Folder".to_string(),
                action: ContextMenuAction::NewFolder,
                enabled: true,
                shortcut: Some("F7".to_string()),
            },
            ContextMenuItem {
                label: "─────────".to_string(),
                action: ContextMenuAction::Copy, // Dummy action for separator
                enabled: false,
                shortcut: None,
            },
            ContextMenuItem {
                label: "Paste".to_string(),
                action: ContextMenuAction::Paste,
                enabled: true, // Should check clipboard
                shortcut: Some("Ctrl+V".to_string()),
            },
            ContextMenuItem {
                label: "─────────".to_string(),
                action: ContextMenuAction::Copy, // Dummy action for separator
                enabled: false,
                shortcut: None,
            },
            ContextMenuItem {
                label: "Select All".to_string(),
                action: ContextMenuAction::SelectAll,
                enabled: true,
                shortcut: Some("Ctrl+A".to_string()),
            },
            ContextMenuItem {
                label: "Invert Selection".to_string(),
                action: ContextMenuAction::InvertSelection,
                enabled: true,
                shortcut: Some("*".to_string()),
            },
            ContextMenuItem {
                label: "─────────".to_string(),
                action: ContextMenuAction::Copy, // Dummy action for separator
                enabled: false,
                shortcut: None,
            },
            ContextMenuItem {
                label: "Refresh".to_string(),
                action: ContextMenuAction::Refresh,
                enabled: true,
                shortcut: Some("Ctrl+R".to_string()),
            },
        ];

        menu.visible = true;
        menu
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            // Skip separators
            while self.selected_index > 0 && !self.items[self.selected_index].enabled {
                self.selected_index -= 1;
            }
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
            // Skip separators
            while self.selected_index < self.items.len() - 1
                && !self.items[self.selected_index].enabled
            {
                self.selected_index += 1;
            }
        }
    }

    /// Get the selected action
    pub fn get_selected_action(&self) -> Option<ContextMenuAction> {
        if self.items[self.selected_index].enabled {
            Some(self.items[self.selected_index].action.clone())
        } else {
            None
        }
    }

    /// Hide the menu
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Calculate the required size for the menu
    pub fn calculate_size(&self) -> (u16, u16) {
        let width = self
            .items
            .iter()
            .map(|item| item.label.len() + item.shortcut.as_ref().map(|s| s.len() + 2).unwrap_or(0))
            .max()
            .unwrap_or(0) as u16
            + 4; // Add padding

        let height = self.items.len() as u16 + 2; // Add border

        (width, height)
    }
}

/// Mouse region for detecting which UI element was clicked
#[derive(Debug, Clone)]
pub struct MouseRegion {
    pub rect: Rect,
    pub region_type: MouseRegionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseRegionType {
    LeftPanel,
    RightPanel,
    CommandLine,
    StatusBar,
    MenuBar,
    Dialog,
}

impl MouseRegion {
    pub fn new(rect: Rect, region_type: MouseRegionType) -> Self {
        Self { rect, region_type }
    }

    /// Check if a position is within this region
    pub fn contains(&self, pos: &Position) -> bool {
        pos.is_within(&self.rect)
    }
}

/// Mouse region manager for tracking UI regions
pub struct MouseRegionManager {
    regions: Vec<MouseRegion>,
}

impl Default for MouseRegionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseRegionManager {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Clear all regions
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Register a new region
    pub fn register(&mut self, region: MouseRegion) {
        self.regions.push(region);
    }

    /// Find which region contains a position
    pub fn find_region(&self, pos: &Position) -> Option<&MouseRegion> {
        self.regions.iter().find(|r| r.contains(pos))
    }

    /// Get the region type at a position
    pub fn get_region_type(&self, pos: &Position) -> Option<MouseRegionType> {
        self.find_region(pos).map(|r| r.region_type.clone())
    }
}
