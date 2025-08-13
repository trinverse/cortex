use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size_bytes: u64,
    pub ram_required: u64,
    pub quantization: String,  // e.g., "Q4_K_M", "Q5_K_S"
    pub capabilities: Vec<String>,  // ["chat", "code", "instruct"]
    pub download_url: String,
    pub sha256: String,
    pub format: ModelFormat,
    pub tier: ModelTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelFormat {
    GGUF,  // Latest llama.cpp format
    GGML,  // Legacy format
    ONNX,  // ONNX format
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelTier {
    Free,      // Embedded models
    Premium,   // Cloud API models
    Hybrid,    // Can run locally or in cloud
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub models: Vec<ModelInfo>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ModelRegistry {
    pub fn default_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "llama2-7b-chat-q4".to_string(),
                name: "Llama 2 7B Chat".to_string(),
                description: "Meta's Llama 2 optimized for dialogue. Good balance of speed and quality.".to_string(),
                size_bytes: 3_800_000_000,  // ~3.8GB
                ram_required: 5_000_000_000, // ~5GB RAM
                quantization: "Q4_K_M".to_string(),
                capabilities: vec!["chat".to_string(), "general".to_string()],
                download_url: "https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF/resolve/main/llama-2-7b-chat.Q4_K_M.gguf".to_string(),
                sha256: "".to_string(),  // Would be actual hash
                format: ModelFormat::GGUF,
                tier: ModelTier::Free,
            },
            ModelInfo {
                id: "codellama-7b-instruct-q4".to_string(),
                name: "CodeLlama 7B Instruct".to_string(),
                description: "Specialized for code generation and analysis. Perfect for file operations.".to_string(),
                size_bytes: 3_800_000_000,
                ram_required: 5_000_000_000,
                quantization: "Q4_K_M".to_string(),
                capabilities: vec!["code".to_string(), "instruct".to_string()],
                download_url: "https://huggingface.co/TheBloke/CodeLlama-7B-Instruct-GGUF/resolve/main/codellama-7b-instruct.Q4_K_M.gguf".to_string(),
                sha256: "".to_string(),
                format: ModelFormat::GGUF,
                tier: ModelTier::Free,
            },
            ModelInfo {
                id: "mistral-7b-instruct-q4".to_string(),
                name: "Mistral 7B Instruct".to_string(),
                description: "Fast and efficient. Great for quick file operations.".to_string(),
                size_bytes: 4_100_000_000,
                ram_required: 5_500_000_000,
                quantization: "Q4_K_M".to_string(),
                capabilities: vec!["chat".to_string(), "instruct".to_string()],
                download_url: "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
                sha256: "".to_string(),
                format: ModelFormat::GGUF,
                tier: ModelTier::Free,
            },
            ModelInfo {
                id: "tinyllama-1b-chat".to_string(),
                name: "TinyLlama 1.1B".to_string(),
                description: "Ultra-fast tiny model. Runs on any hardware.".to_string(),
                size_bytes: 650_000_000,  // ~650MB
                ram_required: 1_000_000_000, // ~1GB RAM
                quantization: "Q4_K_M".to_string(),
                capabilities: vec!["chat".to_string(), "fast".to_string()],
                download_url: "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf".to_string(),
                sha256: "".to_string(),
                format: ModelFormat::GGUF,
                tier: ModelTier::Free,
            },
            // Premium models (require API key)
            ModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                description: "OpenAI's most capable model. Requires API key.".to_string(),
                size_bytes: 0,  // Cloud model
                ram_required: 0,
                quantization: "none".to_string(),
                capabilities: vec!["chat".to_string(), "code".to_string(), "analysis".to_string()],
                download_url: "".to_string(),
                sha256: "".to_string(),
                format: ModelFormat::GGUF,  // Placeholder
                tier: ModelTier::Premium,
            },
            ModelInfo {
                id: "claude-3-sonnet".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                description: "Anthropic's balanced model. Requires API key.".to_string(),
                size_bytes: 0,
                ram_required: 0,
                quantization: "none".to_string(),
                capabilities: vec!["chat".to_string(), "code".to_string(), "analysis".to_string()],
                download_url: "".to_string(),
                sha256: "".to_string(),
                format: ModelFormat::GGUF,
                tier: ModelTier::Premium,
            },
        ]
    }
}

pub struct ModelManager {
    models_dir: PathBuf,
    registry: ModelRegistry,
    downloaded_models: HashMap<String, PathBuf>,
    _active_model: Option<String>,
}

impl ModelManager {
    pub fn new(data_dir: Option<PathBuf>) -> Self {
        let models_dir = data_dir
            .unwrap_or_else(|| dirs::data_dir().unwrap().join("cortex"))
            .join("models");
        
        // Create models directory if it doesn't exist
        std::fs::create_dir_all(&models_dir).ok();
        
        Self {
            models_dir,
            registry: ModelRegistry {
                models: ModelRegistry::default_models(),
                last_updated: chrono::Utc::now(),
            },
            downloaded_models: HashMap::new(),
            _active_model: None,
        }
    }
    
    pub fn list_available_models(&self) -> Vec<&ModelInfo> {
        self.registry.models.iter().collect()
    }
    
    pub fn list_downloaded_models(&self) -> Vec<(&String, &PathBuf)> {
        self.downloaded_models.iter().collect()
    }
    
    pub fn is_model_downloaded(&self, model_id: &str) -> bool {
        self.downloaded_models.contains_key(model_id)
    }
    
    pub fn get_model_path(&self, model_id: &str) -> Option<&PathBuf> {
        self.downloaded_models.get(model_id)
    }
    
    pub async fn download_model(
        &mut self,
        model_id: &str,
        progress_callback: impl Fn(u64, u64) + Send + 'static,
    ) -> Result<PathBuf, String> {
        let model = self.registry.models
            .iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Model {} not found", model_id))?
            .clone();
        
        if model.tier == ModelTier::Premium {
            return Err("Premium models don't need to be downloaded".to_string());
        }
        
        let file_name = format!("{}.gguf", model_id);
        let model_path = self.models_dir.join(&file_name);
        
        // Check if already downloaded
        if model_path.exists() {
            self.downloaded_models.insert(model_id.to_string(), model_path.clone());
            return Ok(model_path);
        }
        
        // Download the model
        let response = reqwest::get(&model.download_url)
            .await
            .map_err(|e| e.to_string())?;
        
        let total_size = response.content_length().unwrap_or(model.size_bytes);
        
        let mut file = fs::File::create(&model_path)
            .await
            .map_err(|e| e.to_string())?;
        
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }
        
        self.downloaded_models.insert(model_id.to_string(), model_path.clone());
        Ok(model_path)
    }
    
    pub async fn delete_model(&mut self, model_id: &str) -> Result<(), String> {
        if let Some(path) = self.downloaded_models.remove(model_id) {
            fs::remove_file(path).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    
    pub fn estimate_ram_usage(&self, model_id: &str) -> Option<u64> {
        self.registry.models
            .iter()
            .find(|m| m.id == model_id)
            .map(|m| m.ram_required)
    }
    
    pub fn can_run_locally(&self, model_id: &str) -> bool {
        if let Some(model) = self.registry.models.iter().find(|m| m.id == model_id) {
            if model.tier == ModelTier::Premium {
                return false;
            }
            
            // Check available RAM
            if let Ok(mem_info) = sys_info::mem_info() {
                let available_ram = (mem_info.avail as u64) * 1024; // Convert to bytes
                return available_ram >= model.ram_required;
            }
        }
        false
    }
}