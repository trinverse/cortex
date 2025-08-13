use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::config::AIConfig;

pub mod provider;
pub mod ollama;
pub mod context;
pub mod prompts;
pub mod embedded;
pub mod hybrid;
pub mod simple;

pub use provider::{AIProvider, AIError, AIResult};
pub use ollama::OllamaProvider;
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
    config: AIConfig,
    current_provider: Option<String>,
}

impl AIManager {
    pub fn new(config: AIConfig) -> Self {
        let mut providers: Vec<Box<dyn AIProvider>> = Vec::new();
        
        // Always add the simple provider as a fallback
        providers.push(Box::new(simple::SimpleAIProvider::new()));
        
        // Try to initialize Ollama provider
        if let Ok(ollama) = OllamaProvider::new(None, None) {
            providers.push(Box::new(ollama));
        }
        
        Self {
            providers,
            current_provider: Some("simple-ai".to_string()), // Use simple AI by default
            config,
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
}