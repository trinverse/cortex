use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum NotificationType {
    FileCreated,
    FileModified,
    FileDeleted,
    FileRenamed,
    Info,
    Warning,
    Error,
}

impl NotificationType {
    pub fn color(&self) -> Color {
        match self {
            NotificationType::FileCreated => Color::Green,
            NotificationType::FileModified => Color::Yellow,
            NotificationType::FileDeleted => Color::Red,
            NotificationType::FileRenamed => Color::Cyan,
            NotificationType::Info => Color::Blue,
            NotificationType::Warning => Color::Yellow,
            NotificationType::Error => Color::Red,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NotificationType::FileCreated => "+ ",
            NotificationType::FileModified => "~ ",
            NotificationType::FileDeleted => "- ",
            NotificationType::FileRenamed => "→ ",
            NotificationType::Info => "ⓘ ",
            NotificationType::Warning => "⚠ ",
            NotificationType::Error => "✗ ",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Duration,
    pub dismissed: bool,
}

impl Notification {
    pub fn new(
        id: u64,
        title: impl Into<String>,
        message: impl Into<String>,
        notification_type: NotificationType,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            message: message.into(),
            notification_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3), // Default 3 second display
            dismissed: false,
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn file_created(id: u64, path: &PathBuf) -> Self {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        Self::new(
            id,
            "File Created",
            format!("{} was created", filename),
            NotificationType::FileCreated,
        )
    }

    pub fn file_modified(id: u64, path: &PathBuf) -> Self {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        Self::new(
            id,
            "File Modified",
            format!("{} was modified", filename),
            NotificationType::FileModified,
        )
    }

    pub fn file_deleted(id: u64, path: &PathBuf) -> Self {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        Self::new(
            id,
            "File Deleted",
            format!("{} was deleted", filename),
            NotificationType::FileDeleted,
        )
    }

    pub fn file_renamed(id: u64, from: &PathBuf, to: &PathBuf) -> Self {
        let from_name = from
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        let to_name = to.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown");

        Self::new(
            id,
            "File Renamed",
            format!("{} → {}", from_name, to_name),
            NotificationType::FileRenamed,
        )
    }

    pub fn is_expired(&self) -> bool {
        !self.dismissed && self.created_at.elapsed() > self.duration
    }

    pub fn dismiss(&mut self) {
        self.dismissed = true;
    }

    pub fn remaining_time(&self) -> Duration {
        self.duration.saturating_sub(self.created_at.elapsed())
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        (elapsed / total).min(1.0)
    }
}

pub struct NotificationManager {
    notifications: VecDeque<Notification>,
    next_id: u64,
    max_notifications: usize,
    show_notifications: bool,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::new(),
            next_id: 1,
            max_notifications: 5,
            show_notifications: true,
        }
    }

    pub fn add(&mut self, notification: Notification) {
        // Remove oldest notification if at max capacity
        while self.notifications.len() >= self.max_notifications {
            self.notifications.pop_front();
        }

        self.notifications.push_back(notification);
    }

    pub fn add_notification(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        notification_type: NotificationType,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let notification = Notification::new(id, title, message, notification_type);
        self.add(notification);

        id
    }

    pub fn add_file_change(&mut self, path: &PathBuf, change_type: NotificationType) {
        let notification = match change_type {
            NotificationType::FileCreated => Notification::file_created(self.next_id, path),
            NotificationType::FileModified => Notification::file_modified(self.next_id, path),
            NotificationType::FileDeleted => Notification::file_deleted(self.next_id, path),
            _ => return, // Only handle file change types
        };

        self.next_id += 1;
        self.add(notification);
    }

    pub fn add_file_rename(&mut self, from: &PathBuf, to: &PathBuf) {
        let notification = Notification::file_renamed(self.next_id, from, to);
        self.next_id += 1;
        self.add(notification);
    }

    pub fn dismiss(&mut self, id: u64) {
        if let Some(notification) = self.notifications.iter_mut().find(|n| n.id == id) {
            notification.dismiss();
        }
    }

    pub fn clear_expired(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    pub fn clear_all(&mut self) {
        self.notifications.clear();
    }

    pub fn toggle_visibility(&mut self) {
        self.show_notifications = !self.show_notifications;
    }

    pub fn is_visible(&self) -> bool {
        self.show_notifications
    }

    pub fn count(&self) -> usize {
        self.notifications.len()
    }

    pub fn has_notifications(&self) -> bool {
        !self.notifications.is_empty()
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if !self.show_notifications {
            return;
        }

        // Clean up expired notifications first
        self.clear_expired();

        if self.notifications.is_empty() {
            return;
        }

        let area = frame.area();
        let notification_width = 40;
        let notification_height = 4;
        let margin = 1;

        // Start from top-right corner
        let start_x = area.width.saturating_sub(notification_width + margin);
        let mut start_y = margin;

        for notification in self.notifications.iter().rev() {
            if notification.dismissed {
                continue;
            }

            // Calculate notification area
            let notification_area = Rect {
                x: start_x,
                y: start_y,
                width: notification_width,
                height: notification_height,
            };

            // Skip if notification would go off screen
            if start_y + notification_height > area.height {
                break;
            }

            self.render_notification(frame, notification, notification_area);
            start_y += notification_height + 1;
        }
    }

    fn render_notification(&self, frame: &mut Frame, notification: &Notification, area: Rect) {
        // Clear the area
        frame.render_widget(Clear, area);

        // Create border with color based on notification type
        let border_style = Style::default()
            .fg(notification.notification_type.color())
            .add_modifier(Modifier::BOLD);

        let block = Block::default()
            .title(format!(" {} ", notification.notification_type.icon()))
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split inner area for title and message
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Min(1),    // Message
            ])
            .split(inner);

        // Render title
        let title = Paragraph::new(notification.title.as_str()).style(
            Style::default()
                .fg(notification.notification_type.color())
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(title, chunks[0]);

        // Render message with wrapping
        let message = Paragraph::new(notification.message.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
        frame.render_widget(message, chunks[1]);

        // Show progress bar at bottom (optional)
        if notification.duration > Duration::from_secs(1) {
            let progress = notification.progress();
            let progress_width = (inner.width as f32 * progress) as u16;

            if progress_width > 0 {
                let progress_area = Rect {
                    x: inner.x,
                    y: inner.y + inner.height - 1,
                    width: progress_width,
                    height: 1,
                };

                let progress_line = Line::from(vec![Span::styled(
                    " ".repeat(progress_width as usize),
                    Style::default().bg(notification.notification_type.color().into()),
                )]);
                let progress_widget = Paragraph::new(progress_line);
                frame.render_widget(progress_widget, progress_area);
            }
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
