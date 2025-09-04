pub mod backend;
pub mod manager;
pub mod renderer;

pub use backend::{WindowBackend, WindowMode, detect_window_mode};
pub use manager::{WindowManager, WindowConfig, WindowEvent as WinEvent};
pub use renderer::TerminalRenderer;