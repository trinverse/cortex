pub mod backend;
pub mod manager;
pub mod renderer;

pub use backend::{WindowBackend, WindowMode};
pub use manager::{WindowManager, WindowConfig, WindowEvent as WinEvent};
pub use renderer::TerminalRenderer;