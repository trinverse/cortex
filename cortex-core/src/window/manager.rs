use crate::window::WindowMode;
use anyhow::Result;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent as WinitWindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub mode: WindowMode,
    pub resizable: bool,
    pub decorations: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Cortex File Manager".to_string(),
            width: 1280,
            height: 800,
            mode: WindowMode::Windowed,
            resizable: true,
            decorations: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    Resize(u32, u32),
    Close,
    KeyPress(char),
    KeyDown(winit::event::VirtualKeyCode),
    MouseMove(f64, f64),
    MouseClick(f64, f64, MouseButton),
    MouseScroll(f64, f64),
    Redraw,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub struct WindowManager {
    config: WindowConfig,
    event_tx: Option<mpsc::UnboundedSender<WindowEvent>>,
    event_rx: Option<mpsc::UnboundedReceiver<WindowEvent>>,
    window_handle: Option<Arc<Window>>,
    event_loop: Option<EventLoop<()>>,
}

impl WindowManager {
    pub fn new(config: WindowConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            config,
            event_tx: Some(event_tx),
            event_rx: Some(event_rx),
            window_handle: None,
            event_loop: None,
        }
    }
    
    pub fn create_window(&mut self) -> Result<Arc<Window>> {
        if self.config.mode == WindowMode::Terminal {
            return Err(anyhow::anyhow!("Cannot create window in terminal mode"));
        }
        
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(self.config.resizable)
            .with_decorations(self.config.decorations)
            .build(&event_loop)?;
        
        let window_arc = Arc::new(window);
        self.window_handle = Some(window_arc.clone());
        self.event_loop = Some(event_loop);
        
        Ok(window_arc)
    }
    
    pub fn run_event_loop(mut self) -> Result<()> {
        let event_loop = self.event_loop
            .take()
            .ok_or_else(|| anyhow::anyhow!("No event loop created"))?;
        
        let window = self.window_handle
            .ok_or_else(|| anyhow::anyhow!("No window created"))?;
        
        let event_tx = self.event_tx.clone()
            .ok_or_else(|| anyhow::anyhow!("No event sender"))?;
        
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            
            match event {
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WinitWindowEvent::CloseRequested => {
                            let _ = event_tx.send(WindowEvent::Close);
                            *control_flow = ControlFlow::Exit;
                        }
                        WinitWindowEvent::Resized(size) => {
                            let _ = event_tx.send(WindowEvent::Resize(size.width, size.height));
                        }
                        WinitWindowEvent::KeyboardInput { input, .. } => {
                            if let Some(keycode) = input.virtual_keycode {
                                let _ = event_tx.send(WindowEvent::KeyDown(keycode));
                            }
                        }
                        WinitWindowEvent::ReceivedCharacter(c) => {
                            if !c.is_control() {
                                let _ = event_tx.send(WindowEvent::KeyPress(c));
                            }
                        }
                        WinitWindowEvent::CursorMoved { position, .. } => {
                            let _ = event_tx.send(WindowEvent::MouseMove(position.x, position.y));
                        }
                        WinitWindowEvent::MouseInput { state, button, .. } => {
                            if state == winit::event::ElementState::Pressed {
                                let mb = match button {
                                    winit::event::MouseButton::Left => MouseButton::Left,
                                    winit::event::MouseButton::Right => MouseButton::Right,
                                    winit::event::MouseButton::Middle => MouseButton::Middle,
                                    _ => return,
                                };
                                // We'd need to track cursor position for click coordinates
                                let _ = event_tx.send(WindowEvent::MouseClick(0.0, 0.0, mb));
                            }
                        }
                        WinitWindowEvent::MouseWheel { delta, .. } => {
                            if let winit::event::MouseScrollDelta::LineDelta(x, y) = delta {
                                let _ = event_tx.send(WindowEvent::MouseScroll(x as f64, y as f64));
                            }
                        }
                        _ => {}
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let _ = event_tx.send(WindowEvent::Redraw);
                }
                _ => {}
            }
        });
    }
    
    pub fn spawn_window_thread(config: WindowConfig) -> Result<mpsc::UnboundedReceiver<WindowEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        thread::spawn(move || {
            let mut manager = WindowManager::new(config);
            if let Ok(_window) = manager.create_window() {
                // Connect event channels
                if let Some(event_tx) = manager.event_tx.take() {
                    // Forward events to the main thread
                    if let Some(mut event_rx) = manager.event_rx.take() {
                        thread::spawn(move || {
                            while let Some(event) = event_rx.blocking_recv() {
                                let _ = tx.send(event);
                            }
                        });
                    }
                    
                    // Replace with forwarding sender
                    manager.event_tx = Some(event_tx);
                }
                
                let _ = manager.run_event_loop();
            }
        });
        
        Ok(rx)
    }
    
    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<WindowEvent>> {
        self.event_rx.take()
    }
}