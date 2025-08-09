use anyhow::Result;
use async_trait::async_trait;
use mlua::{Lua, Result as LuaResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    async fn initialize(&mut self) -> Result<()>;
    async fn execute(&self, command: &str, args: Vec<String>) -> Result<String>;
    async fn shutdown(&mut self) -> Result<()>;
}

pub struct LuaPlugin {
    lua: Lua,
    info: PluginInfo,
    script_path: PathBuf,
}

impl LuaPlugin {
    pub fn new(script_path: PathBuf) -> Result<Self> {
        let lua = Lua::new();
        
        let info = PluginInfo {
            name: "Unnamed Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Unknown".to_string(),
            description: "No description".to_string(),
        };
        
        Ok(Self {
            lua,
            info,
            script_path,
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
        }
        
        Ok(())
    }
}

#[async_trait]
impl Plugin for LuaPlugin {
    fn info(&self) -> PluginInfo {
        self.info.clone()
    }
    
    async fn initialize(&mut self) -> Result<()> {
        self.load_script()?;
        
        let globals = self.lua.globals();
        if let Ok(init_fn) = globals.get::<mlua::Function>("initialize") {
            init_fn.call::<()>(())?;
        }
        
        Ok(())
    }
    
    async fn execute(&self, command: &str, args: Vec<String>) -> Result<String> {
        let globals = self.lua.globals();
        
        if let Ok(execute_fn) = globals.get::<mlua::Function>("execute") {
            let result = execute_fn.call::<String>((command, args))?;
            Ok(result)
        } else {
            Ok(String::new())
        }
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        let globals = self.lua.globals();
        
        if let Ok(shutdown_fn) = globals.get::<mlua::Function>("shutdown") {
            shutdown_fn.call::<()>(())?;
        }
        
        Ok(())
    }
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    pub async fn load_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        self.plugins.push(plugin);
        Ok(())
    }
    
    pub async fn initialize_all(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.initialize().await?;
        }
        Ok(())
    }
    
    pub async fn execute_command(&self, plugin_name: &str, command: &str, args: Vec<String>) -> Result<String> {
        for plugin in &self.plugins {
            if plugin.info().name == plugin_name {
                return plugin.execute(command, args).await;
            }
        }
        
        Err(anyhow::anyhow!("Plugin not found: {}", plugin_name))
    }
    
    pub async fn shutdown_all(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.shutdown().await?;
        }
        Ok(())
    }
}