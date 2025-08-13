use async_trait::async_trait;
use std::fmt;
use super::{AIResponse, StreamingResponse, AIContext};

pub type AIResult<T> = Result<T, AIError>;

#[derive(Debug)]
pub enum AIError {
    ProviderUnavailable,
    ModelNotFound(String),
    ContextTooLarge,
    InvalidResponse(String),
    OperationDenied(String),
    NetworkError(String),
    ConfigurationError(String),
    StreamingError(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIError::ProviderUnavailable => write!(f, "AI provider is not available"),
            AIError::ModelNotFound(model) => write!(f, "Model '{}' not found", model),
            AIError::ContextTooLarge => write!(f, "Context exceeds maximum token limit"),
            AIError::InvalidResponse(msg) => write!(f, "Invalid AI response: {}", msg),
            AIError::OperationDenied(reason) => write!(f, "Operation denied: {}", reason),
            AIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AIError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            AIError::StreamingError(msg) => write!(f, "Streaming error: {}", msg),
        }
    }
}

impl std::error::Error for AIError {}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, prompt: &str, context: AIContext) -> AIResult<AIResponse>;
    
    async fn stream(&self, prompt: &str, context: AIContext) -> AIResult<StreamingResponse>;
    
    fn name(&self) -> &str;
    
    fn model(&self) -> &str;
    
    fn is_available(&self) -> bool;
    
    fn max_context_tokens(&self) -> usize {
        4096 // Default context window
    }
}