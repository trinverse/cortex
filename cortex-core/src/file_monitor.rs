use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub enum FileMonitorEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
}

#[derive(Debug, Clone)]
pub struct ChangeNotification {
    pub path: PathBuf,
    pub event: FileMonitorEvent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub type EventCallback = Arc<dyn Fn(ChangeNotification) + Send + Sync>;

pub struct FileMonitor {
    watcher: Option<RecommendedWatcher>,
    watched_paths: Arc<RwLock<HashMap<PathBuf, bool>>>, // path -> recursive
    event_sender: mpsc::UnboundedSender<FileMonitorEvent>,
    callbacks: Arc<RwLock<Vec<EventCallback>>>,
}

impl FileMonitor {
    pub fn new() -> Result<(Self, mpsc::UnboundedReceiver<FileMonitorEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let watched_paths = Arc::new(RwLock::new(HashMap::new()));
        let callbacks = Arc::new(RwLock::new(Vec::new()));

        let monitor = Self {
            watcher: None,
            watched_paths,
            event_sender,
            callbacks,
        };

        Ok((monitor, event_receiver))
    }

    pub async fn start(&mut self) -> Result<()> {
        let sender = self.event_sender.clone();
        let callbacks = self.callbacks.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Some(monitor_event) = Self::convert_event(event) {
                        let notification = ChangeNotification {
                            path: match &monitor_event {
                                FileMonitorEvent::Created(p) | 
                                FileMonitorEvent::Modified(p) | 
                                FileMonitorEvent::Deleted(p) => p.clone(),
                                FileMonitorEvent::Renamed { to, .. } => to.clone(),
                            },
                            event: monitor_event.clone(),
                            timestamp: chrono::Utc::now(),
                        };

                        // Send to main event loop
                        let _ = sender.send(monitor_event);
                        
                        // Call registered callbacks
                        let callbacks_clone = callbacks.clone();
                        tokio::spawn(async move {
                            let callbacks = callbacks_clone.read().await;
                            for callback in callbacks.iter() {
                                callback(notification.clone());
                            }
                        });
                    }
                }
                Err(e) => {
                    log::error!("File watcher error: {:?}", e);
                }
            }
        })?;

        // Configure watcher
        watcher.configure(Config::default().with_poll_interval(Duration::from_millis(100)))?;

        self.watcher = Some(watcher);
        Ok(())
    }

    pub async fn watch_path(&mut self, path: &Path, recursive: bool) -> Result<()> {
        if let Some(ref mut watcher) = self.watcher {
            let mode = if recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            
            watcher.watch(path, mode)?;
            
            let mut paths = self.watched_paths.write().await;
            paths.insert(path.to_path_buf(), recursive);
        }
        Ok(())
    }

    pub async fn unwatch_path(&mut self, path: &Path) -> Result<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher.unwatch(path)?;
            
            let mut paths = self.watched_paths.write().await;
            paths.remove(path);
        }
        Ok(())
    }

    pub async fn is_watching(&self, path: &Path) -> bool {
        let paths = self.watched_paths.read().await;
        paths.contains_key(path)
    }

    pub async fn get_watched_paths(&self) -> Vec<PathBuf> {
        let paths = self.watched_paths.read().await;
        paths.keys().cloned().collect()
    }

    pub async fn register_callback(&self, callback: EventCallback) {
        let mut callbacks = self.callbacks.write().await;
        callbacks.push(callback);
    }

    pub async fn clear_callbacks(&self) {
        let mut callbacks = self.callbacks.write().await;
        callbacks.clear();
    }

    fn convert_event(event: Event) -> Option<FileMonitorEvent> {
        match event.kind {
            EventKind::Create(_) => {
                if let Some(path) = event.paths.first() {
                    Some(FileMonitorEvent::Created(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Modify(_) => {
                if let Some(path) = event.paths.first() {
                    Some(FileMonitorEvent::Modified(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Remove(_) => {
                if let Some(path) = event.paths.first() {
                    Some(FileMonitorEvent::Deleted(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Other => {
                // Handle rename events
                if event.paths.len() == 2 {
                    Some(FileMonitorEvent::Renamed {
                        from: event.paths[0].clone(),
                        to: event.paths[1].clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(watcher) = self.watcher.take() {
            drop(watcher);
        }
        
        let mut paths = self.watched_paths.write().await;
        paths.clear();
        
        Ok(())
    }
}

impl Drop for FileMonitor {
    fn drop(&mut self) {
        if let Some(watcher) = self.watcher.take() {
            drop(watcher);
        }
    }
}

// Utility functions for managing file monitoring
pub struct FileMonitorManager {
    monitor: Arc<RwLock<FileMonitor>>,
    event_receiver: Arc<RwLock<mpsc::UnboundedReceiver<FileMonitorEvent>>>,
    is_running: Arc<RwLock<bool>>,
}

impl FileMonitorManager {
    pub async fn new() -> Result<Self> {
        let (monitor, event_receiver) = FileMonitor::new()?;
        
        Ok(Self {
            monitor: Arc::new(RwLock::new(monitor)),
            event_receiver: Arc::new(RwLock::new(event_receiver)),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        {
            let mut monitor = self.monitor.write().await;
            monitor.start().await?;
        }
        
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }

        // Start event processing loop
        let _monitor_clone = self.monitor.clone();
        let receiver_clone = self.event_receiver.clone();
        let running_clone = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut receiver = receiver_clone.write().await;
            
            while *running_clone.read().await {
                match receiver.recv().await {
                    Some(event) => {
                        log::debug!("File monitor event: {:?}", event);
                        // Events are already processed by callbacks in the watcher
                    }
                    None => {
                        log::warn!("File monitor event receiver closed");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }
        
        {
            let mut monitor = self.monitor.write().await;
            monitor.stop().await?;
        }
        
        Ok(())
    }

    pub async fn watch_directory(&self, path: &Path, recursive: bool) -> Result<()> {
        let mut monitor = self.monitor.write().await;
        monitor.watch_path(path, recursive).await
    }

    pub async fn unwatch_directory(&self, path: &Path) -> Result<()> {
        let mut monitor = self.monitor.write().await;
        monitor.unwatch_path(path).await
    }

    pub async fn register_change_callback(&self, callback: EventCallback) {
        let monitor = self.monitor.read().await;
        monitor.register_callback(callback).await;
    }

    pub async fn is_watching(&self, path: &Path) -> bool {
        let monitor = self.monitor.read().await;
        monitor.is_watching(path).await
    }

    pub async fn get_watched_directories(&self) -> Vec<PathBuf> {
        let monitor = self.monitor.read().await;
        monitor.get_watched_paths().await
    }
}