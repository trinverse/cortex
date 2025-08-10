use anyhow::Result;
use async_trait::async_trait;
use mlua::{Lua, Result as LuaResult, UserData, UserDataMethods};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub min_cortex_version: String,
    pub commands: Vec<String>,
    pub event_hooks: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    FileSelected {
        path: PathBuf,
    },
    DirectoryChanged {
        path: PathBuf,
    },
    FileOperation {
        operation: String,
        source: PathBuf,
        destination: Option<PathBuf>,
    },
    PanelFocused {
        panel: String,
    },
    CommandExecuted {
        command: String,
        args: Vec<String>,
    },
    ApplicationStartup,
    ApplicationShutdown,
}

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub current_file: Option<PathBuf>,
    pub current_directory: PathBuf,
    pub selected_files: Vec<PathBuf>,
    pub active_panel: String,
    pub other_panel_directory: PathBuf,
}

impl UserData for PluginContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_current_file", |_, this, ()| {
            Ok(this
                .current_file
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()))
        });

        methods.add_method("get_current_directory", |_, this, ()| {
            Ok(this.current_directory.to_string_lossy().to_string())
        });

        methods.add_method("get_selected_files", |_, this, ()| {
            Ok(this
                .selected_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<String>>())
        });

        methods.add_method("get_active_panel", |_, this, ()| {
            Ok(this.active_panel.clone())
        });

        methods.add_method("get_other_panel_directory", |_, this, ()| {
            Ok(this.other_panel_directory.to_string_lossy().to_string())
        });
    }
}

#[derive(Debug, Clone)]
pub struct CortexAPI {
    pub context: PluginContext,
}

impl UserData for CortexAPI {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // File system operations
        methods.add_method(
            "read_file",
            |_, _, path: String| match std::fs::read_to_string(&path) {
                Ok(content) => Ok(Some(content)),
                Err(_) => Ok(None),
            },
        );

        methods.add_method("write_file", |_, _, (path, content): (String, String)| {
            Ok(std::fs::write(&path, content).is_ok())
        });

        methods.add_method("file_exists", |_, _, path: String| {
            Ok(std::path::Path::new(&path).exists())
        });

        methods.add_method("is_directory", |_, _, path: String| {
            Ok(std::path::Path::new(&path).is_dir())
        });

        methods.add_method(
            "list_directory",
            |_, _, path: String| match std::fs::read_dir(&path) {
                Ok(entries) => {
                    let files: Vec<String> = entries
                        .filter_map(|entry| entry.ok()?.file_name().to_str().map(|s| s.to_string()))
                        .collect();
                    Ok(files)
                }
                Err(_) => Ok(Vec::new()),
            },
        );

        // UI operations
        methods.add_method("show_message", |_, _, message: String| {
            // This will be implemented to show messages in the UI
            println!("Plugin Message: {}", message);
            Ok(())
        });

        methods.add_method("show_error", |_, _, error: String| {
            eprintln!("Plugin Error: {}", error);
            Ok(())
        });

        methods.add_method("prompt_user", |_, _, prompt: String| {
            // This will be implemented to show input dialogs
            println!("Plugin Prompt: {}", prompt);
            Ok(Some("".to_string())) // Placeholder
        });

        // System operations
        methods.add_method("execute_command", |_, _, command: String| {
            match std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .output()
            {
                Ok(output) => Ok(Some(String::from_utf8_lossy(&output.stdout).to_string())),
                Err(_) => Ok(None),
            }
        });

        methods.add_method("get_env", |_, _, var: String| Ok(std::env::var(&var).ok()));

        methods.add_method("set_env", |_, _, (var, value): (String, String)| {
            std::env::set_var(&var, &value);
            Ok(())
        });
    }
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    async fn initialize(&mut self, api: CortexAPI) -> Result<()>;
    async fn execute(
        &self,
        command: &str,
        args: Vec<String>,
        context: PluginContext,
    ) -> Result<String>;
    async fn handle_event(&self, event: PluginEvent, context: PluginContext) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

pub struct LuaPlugin {
    lua: Lua,
    info: PluginInfo,
    script_path: PathBuf,
    enabled: bool,
}

impl LuaPlugin {
    pub fn new(script_path: PathBuf) -> Result<Self> {
        let lua = Lua::new();

        let info = PluginInfo {
            name: "Unnamed Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Unknown".to_string(),
            description: "No description".to_string(),
            min_cortex_version: "0.1.0".to_string(),
            commands: Vec::new(),
            event_hooks: Vec::new(),
        };

        Ok(Self {
            lua,
            info,
            script_path,
            enabled: true,
        })
    }

    pub fn load_script(&mut self) -> LuaResult<()> {
        let script = std::fs::read_to_string(&self.script_path)?;
        self.lua.load(&script).exec()?;

        let globals = self.lua.globals();
        if let Ok(plugin_table) = globals.get::<mlua::Table>("plugin") {
            if let Ok(name) = plugin_table.get::<String>("name") {
                self.info.name = name;
            }
            if let Ok(version) = plugin_table.get::<String>("version") {
                self.info.version = version;
            }
            if let Ok(author) = plugin_table.get::<String>("author") {
                self.info.author = author;
            }
            if let Ok(description) = plugin_table.get::<String>("description") {
                self.info.description = description;
            }
            if let Ok(min_version) = plugin_table.get::<String>("min_cortex_version") {
                self.info.min_cortex_version = min_version;
            }
            if let Ok(commands) = plugin_table.get::<Vec<String>>("commands") {
                self.info.commands = commands;
            }
            if let Ok(event_hooks) = plugin_table.get::<Vec<String>>("event_hooks") {
                self.info.event_hooks = event_hooks;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Plugin for LuaPlugin {
    fn info(&self) -> PluginInfo {
        self.info.clone()
    }

    async fn initialize(&mut self, api: CortexAPI) -> Result<()> {
        self.load_script()?;

        // Provide API to Lua
        let globals = self.lua.globals();
        globals.set("cortex", api)?;

        if let Ok(init_fn) = globals.get::<mlua::Function>("initialize") {
            init_fn.call::<()>(())?;
        }

        Ok(())
    }

    async fn execute(
        &self,
        command: &str,
        args: Vec<String>,
        context: PluginContext,
    ) -> Result<String> {
        let globals = self.lua.globals();

        // Provide context to Lua
        globals.set("context", context)?;

        if let Ok(execute_fn) = globals.get::<mlua::Function>("execute") {
            let result = execute_fn.call::<String>((command, args))?;
            Ok(result)
        } else {
            Ok(String::new())
        }
    }

    async fn handle_event(&self, event: PluginEvent, context: PluginContext) -> Result<()> {
        let globals = self.lua.globals();

        // Provide context to Lua
        globals.set("context", context)?;

        // Convert event to Lua-friendly format
        let event_name = match event {
            PluginEvent::FileSelected { .. } => "file_selected",
            PluginEvent::DirectoryChanged { .. } => "directory_changed",
            PluginEvent::FileOperation { .. } => "file_operation",
            PluginEvent::PanelFocused { .. } => "panel_focused",
            PluginEvent::CommandExecuted { .. } => "command_executed",
            PluginEvent::ApplicationStartup => "application_startup",
            PluginEvent::ApplicationShutdown => "application_shutdown",
        };

        if let Ok(handle_event_fn) = globals.get::<mlua::Function>("handle_event") {
            handle_event_fn.call::<()>((event_name, self.serialize_event_data(&event)?))?;
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        let globals = self.lua.globals();

        if let Ok(shutdown_fn) = globals.get::<mlua::Function>("shutdown") {
            shutdown_fn.call::<()>(())?;
        }

        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl LuaPlugin {
    fn serialize_event_data(&self, event: &PluginEvent) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        match event {
            PluginEvent::FileSelected { path } => {
                data.insert("path".to_string(), path.to_string_lossy().to_string());
            }
            PluginEvent::DirectoryChanged { path } => {
                data.insert("path".to_string(), path.to_string_lossy().to_string());
            }
            PluginEvent::FileOperation {
                operation,
                source,
                destination,
            } => {
                data.insert("operation".to_string(), operation.clone());
                data.insert("source".to_string(), source.to_string_lossy().to_string());
                if let Some(dest) = destination {
                    data.insert(
                        "destination".to_string(),
                        dest.to_string_lossy().to_string(),
                    );
                }
            }
            PluginEvent::PanelFocused { panel } => {
                data.insert("panel".to_string(), panel.clone());
            }
            PluginEvent::CommandExecuted { command, args } => {
                data.insert("command".to_string(), command.clone());
                data.insert("args".to_string(), args.join(" "));
            }
            PluginEvent::ApplicationStartup | PluginEvent::ApplicationShutdown => {
                // No additional data for these events
            }
        }

        Ok(data)
    }
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_commands: HashMap<String, String>, // command -> plugin_name
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_commands: HashMap::new(),
        }
    }

    pub async fn load_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let plugin_info = plugin.info();

        // Register plugin commands
        for command in &plugin_info.commands {
            self.plugin_commands
                .insert(command.clone(), plugin_info.name.clone());
        }

        self.plugins.push(plugin);
        Ok(())
    }

    pub async fn initialize_all(&mut self, base_context: PluginContext) -> Result<()> {
        for plugin in &mut self.plugins {
            let api = CortexAPI {
                context: base_context.clone(),
            };
            plugin.initialize(api).await?;
        }
        Ok(())
    }

    pub async fn execute_command(
        &self,
        command: &str,
        args: Vec<String>,
        context: PluginContext,
    ) -> Result<String> {
        if let Some(plugin_name) = self.plugin_commands.get(command) {
            for plugin in &self.plugins {
                if plugin.info().name == *plugin_name && plugin.is_enabled() {
                    return plugin.execute(command, args, context).await;
                }
            }
        }

        Err(anyhow::anyhow!("Plugin command not found: {}", command))
    }

    pub async fn handle_event(&self, event: PluginEvent, context: PluginContext) -> Result<()> {
        for plugin in &self.plugins {
            if plugin.is_enabled() {
                plugin.handle_event(event.clone(), context.clone()).await?;
            }
        }
        Ok(())
    }

    pub async fn shutdown_all(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.shutdown().await?;
        }
        Ok(())
    }

    pub fn get_available_commands(&self) -> Vec<String> {
        self.plugin_commands.keys().cloned().collect()
    }

    pub fn get_plugin_info(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info()).collect()
    }

    pub fn toggle_plugin(&mut self, plugin_name: &str) -> Result<bool> {
        for plugin in &mut self.plugins {
            if plugin.info().name == plugin_name {
                let new_state = !plugin.is_enabled();
                plugin.set_enabled(new_state);
                return Ok(new_state);
            }
        }
        Err(anyhow::anyhow!("Plugin not found: {}", plugin_name))
    }

    pub fn is_plugin_enabled(&self, plugin_name: &str) -> bool {
        for plugin in &self.plugins {
            if plugin.info().name == plugin_name {
                return plugin.is_enabled();
            }
        }
        false
    }
}
