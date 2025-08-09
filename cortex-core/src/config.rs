use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub panels: PanelConfig,
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub keybindings: KeybindingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub show_hidden: bool,
    #[serde(default = "default_true")]
    pub confirm_delete: bool,
    #[serde(default = "default_false")]
    pub show_icons: bool,
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default = "default_editor")]
    pub editor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    #[serde(default = "default_sort")]
    pub default_sort: String,
    #[serde(default = "default_true")]
    pub show_size: bool,
    #[serde(default = "default_true")]
    pub show_permissions: bool,
    #[serde(default = "default_true")]
    pub show_modified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    #[serde(default = "default_selection_bg")]
    pub selection_bg: String,
    #[serde(default = "default_directory_fg")]
    pub directory_fg: String,
    #[serde(default = "default_executable_fg")]
    pub executable_fg: String,
    #[serde(default = "default_symlink_fg")]
    pub symlink_fg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    #[serde(default)]
    pub custom: Vec<CustomKeybinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomKeybinding {
    pub key: String,
    pub command: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            panels: PanelConfig::default(),
            colors: ColorConfig::default(),
            keybindings: KeybindingConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            confirm_delete: true,
            show_icons: false,
            terminal: default_terminal(),
            editor: default_editor(),
        }
    }
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            default_sort: default_sort(),
            show_size: true,
            show_permissions: true,
            show_modified: true,
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            selection_bg: default_selection_bg(),
            directory_fg: default_directory_fg(),
            executable_fg: default_executable_fg(),
            symlink_fg: default_symlink_fg(),
        }
    }
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            custom: Vec::new(),
        }
    }
}

fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_terminal() -> String { "bash".to_string() }
fn default_editor() -> String { "vim".to_string() }
fn default_sort() -> String { "name".to_string() }
fn default_selection_bg() -> String { "blue".to_string() }
fn default_directory_fg() -> String { "cyan".to_string() }
fn default_executable_fg() -> String { "green".to_string() }
fn default_symlink_fg() -> String { "magenta".to_string() }

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_or_create(&config_path)?;
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        })
    }
    
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        let cortex_config_dir = config_dir.join("cortex");
        if !cortex_config_dir.exists() {
            fs::create_dir_all(&cortex_config_dir)?;
        }
        
        Ok(cortex_config_dir.join("config.toml"))
    }
    
    fn load_or_create(path: &Path) -> Result<Config> {
        if path.exists() {
            let contents = fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            let config = Config::default();
            let contents = toml::to_string_pretty(&config)?;
            fs::write(path, contents)?;
            Ok(config)
        }
    }
    
    pub fn reload(&self) -> Result<()> {
        let config = Self::load_or_create(&self.config_path)?;
        let mut write_guard = self.config.write().unwrap();
        *write_guard = config;
        Ok(())
    }
    
    pub fn get(&self) -> Config {
        self.config.read().unwrap().clone()
    }
    
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut write_guard = self.config.write().unwrap();
        f(&mut write_guard);
        let contents = toml::to_string_pretty(&*write_guard)?;
        fs::write(&self.config_path, contents)?;
        Ok(())
    }
    
    pub fn watch_for_changes(&self) -> Result<()> {
        use notify::{Watcher, RecursiveMode, recommended_watcher, Event, EventKind};
        use std::sync::mpsc::channel;
        use std::time::Duration;
        
        let (tx, rx) = channel();
        let config_path = self.config_path.clone();
        let manager = self.clone();
        
        let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    let _ = tx.send(());
                }
            }
        })?;
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        std::thread::spawn(move || {
            while rx.recv().is_ok() {
                std::thread::sleep(Duration::from_millis(100)); // Debounce
                if let Err(e) = manager.reload() {
                    eprintln!("Failed to reload config: {}", e);
                }
            }
        });
        
        Ok(())
    }
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            config_path: self.config_path.clone(),
        }
    }
}