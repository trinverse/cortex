use anyhow::Result;
use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

pub struct EventHandler {
    _sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    shutdown_flag: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_clone = sender.clone();
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let shutdown_flag_clone = shutdown_flag.clone();

        tokio::spawn(async move {
            loop {
                // Check for shutdown signal
                if shutdown_flag_clone.load(Ordering::Relaxed) {
                    break;
                }

                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(event) = event::read() {
                        match event {
                            CrosstermEvent::Key(key) => {
                                if sender_clone.send(Event::Key(key)).is_err() {
                                    break; // Receiver dropped, exit loop
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                if sender_clone.send(Event::Mouse(mouse)).is_err() {
                                    break; // Receiver dropped, exit loop
                                }
                            }
                            CrosstermEvent::Resize(width, height) => {
                                if sender_clone.send(Event::Resize(width, height)).is_err() {
                                    break; // Receiver dropped, exit loop
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    if sender_clone.send(Event::Tick).is_err() {
                        break; // Receiver dropped, exit loop
                    }
                }
            }
        });

        Self {
            _sender: sender,
            receiver,
            shutdown_flag,
        }
    }

    /// Shutdown the background event handling task
    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::Relaxed);
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyBinding {
    Quit,
    Help,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Copy,
    Move,
    Delete,
    MakeDir,
    Rename,
    Search,
    ToggleHidden,
    ToggleMark,
    MarkAll,
    UnmarkAll,
    Refresh,
    ViewFile,
    EditFile,
    SortByName,
    SortBySize,
    SortByDate,
    SortByExt,
    CommandMode,
    ExecuteCommand,
    CancelCommand,
}

impl KeyBinding {
    pub fn from_key_event(key: KeyEvent) -> Option<Self> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Self::Quit),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Self::Quit),
            (KeyCode::Char('?'), _) => Some(Self::Help),

            (KeyCode::Up, _) => Some(Self::Up),
            (KeyCode::Down, _) => Some(Self::Down),
            (KeyCode::Left, _) => Some(Self::Left),
            (KeyCode::Right, _) => Some(Self::Right),
            (KeyCode::Enter, _) => Some(Self::Enter),
            (KeyCode::Backspace, _) => Some(Self::Back),
            (KeyCode::Home, _) => Some(Self::Home),
            (KeyCode::End, _) => Some(Self::End),
            (KeyCode::PageUp, _) => Some(Self::PageUp),
            (KeyCode::PageDown, _) => Some(Self::PageDown),
            (KeyCode::Tab, _) => Some(Self::Tab),

            (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Self::Rename),

            (KeyCode::Char('/'), _) => Some(Self::Search),
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => Some(Self::ToggleHidden),
            (KeyCode::Char(' '), _) => Some(Self::ToggleMark),
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => Some(Self::MarkAll),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(Self::UnmarkAll),
            (KeyCode::Char('r'), KeyModifiers::ALT) => Some(Self::Refresh),


            (KeyCode::Char('1'), KeyModifiers::ALT) => Some(Self::SortByName),
            (KeyCode::Char('2'), KeyModifiers::ALT) => Some(Self::SortBySize),
            (KeyCode::Char('3'), KeyModifiers::ALT) => Some(Self::SortByDate),
            (KeyCode::Char('4'), KeyModifiers::ALT) => Some(Self::SortByExt),

            (KeyCode::Char(':'), _) => Some(Self::CommandMode),
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => Some(Self::CommandMode),
            (KeyCode::Esc, _) => Some(Self::CancelCommand),

            _ => None,
        }
    }
}
