use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::config::AIConfig;

pub mod provider;
pub mod ollama;
pub mod groq;
pub mod context;
pub mod prompts;
pub mod embedded;
pub mod hybrid;
pub mod simple;

pub use provider::{AIProvider, AIError, AIResult};
pub use ollama::OllamaProvider;
pub use groq::GroqProvider;
pub use context::AIContext;
pub use prompts::PromptBuilder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: Option<usize>,
}

#[derive(Debug)]
pub struct StreamingResponse {
    pub receiver: mpsc::Receiver<String>,
}

pub struct AIManager {
    providers: Vec<Box<dyn AIProvider>>,
    _config: AIConfig,
    current_provider: Option<String>,
}

impl AIManager {
    pub fn new(config: AIConfig) -> Self {
        let mut providers: Vec<Box<dyn AIProvider>> = Vec::new();
        
        // Always add the simple provider as a fallback
        providers.push(Box::new(simple::SimpleAIProvider::new()));
        
        // Try to initialize Groq provider (free cloud API)
        // Priority: 1) Environment variable, 2) Config file, 3) Demo key
        let groq_api_key = std::env::var("GROQ_API_KEY").ok()
            .or_else(|| config.cloud.groq_api_key.clone());
        
        let groq_provider = if let Some(api_key) = groq_api_key {
            GroqProvider::new(Some(api_key))
        } else {
            // Use bundled demo key for out-of-the-box experience
            // Users should get their own free key from https://console.groq.com
            GroqProvider::with_demo_key()
        };
        
        if let Ok(groq) = groq_provider {
            providers.push(Box::new(groq));
        }
        
        // Try to initialize Ollama provider (local)
        if let Ok(ollama) = OllamaProvider::new(None, None) {
            providers.push(Box::new(ollama));
        }
        
        // Determine the default provider
        let default_provider = if providers.iter().any(|p| p.name() == "groq") {
            "groq".to_string()
        } else {
            "simple-ai".to_string()
        };
        
        Self {
            providers,
            current_provider: Some(default_provider),
            _config: config,
        }
    }
    
    pub fn get_provider(&self, name: Option<&str>) -> Option<&dyn AIProvider> {
        let provider_name = name.or(self.current_provider.as_deref())?;
        
        self.providers
            .iter()
            .find(|p| p.name() == provider_name)
            .map(|p| p.as_ref())
    }
    
    pub async fn complete(&self, prompt: &str, context: AIContext) -> AIResult<AIResponse> {
        let provider = self.get_provider(None)
            .ok_or(AIError::ProviderUnavailable)?;
        
        provider.complete(prompt, context).await
    }
    
    pub async fn stream(&self, prompt: &str, context: AIContext) -> AIResult<StreamingResponse> {
        let provider = self.get_provider(None)
            .ok_or(AIError::ProviderUnavailable)?;
        
        provider.stream(prompt, context).await
    }
    
    pub fn is_available(&self) -> bool {
        self.providers.iter().any(|p| p.is_available())
    }
    
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.name().to_string()).collect()
    }
    
    pub fn get_current_provider(&self) -> Option<String> {
        self.current_provider.clone()
    }
    
    pub fn set_provider(&mut self, name: &str) -> bool {
        if self.providers.iter().any(|p| p.name() == name) {
            self.current_provider = Some(name.to_string());
            true
        } else {
            false
        }
    }
}