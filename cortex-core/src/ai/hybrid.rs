use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::{AIProvider, AIError, AIResult, AIResponse, StreamingResponse, AIContext};

#[derive(Debug, Clone)]
pub enum ProviderMode {
    LocalOnly,      // Only use embedded models
    CloudOnly,      // Only use cloud APIs
    PreferLocal,    // Try local first, fallback to cloud
    PreferCloud,    // Try cloud first, fallback to local
    CostOptimized,  // Use local for simple tasks, cloud for complex
}

pub struct HybridProvider {
    local_provider: Option<Box<dyn AIProvider>>,
    cloud_provider: Option<Box<dyn AIProvider>>,
    mode: Arc<RwLock<ProviderMode>>,
    usage_tracker: Arc<RwLock<UsageTracker>>,
}

#[derive(Debug, Default)]
struct UsageTracker {
    local_tokens: usize,
    cloud_tokens: usize,
    local_requests: usize,
    cloud_requests: usize,
    estimated_cost: f64,
}

impl HybridProvider {
    pub fn new(mode: ProviderMode) -> Self {
        Self {
            local_provider: None,
            cloud_provider: None,
            mode: Arc::new(RwLock::new(mode)),
            usage_tracker: Arc::new(RwLock::new(UsageTracker::default())),
        }
    }
    
    pub fn with_local(mut self, provider: Box<dyn AIProvider>) -> Self {
        self.local_provider = Some(provider);
        self
    }
    
    pub fn with_cloud(mut self, provider: Box<dyn AIProvider>) -> Self {
        self.cloud_provider = Some(provider);
        self
    }
    
    pub async fn set_mode(&self, mode: ProviderMode) {
        *self.mode.write().await = mode;
    }
    
    pub async fn get_usage_stats(&self) -> UsageStats {
        let tracker = self.usage_tracker.read().await;
        UsageStats {
            local_tokens: tracker.local_tokens,
            cloud_tokens: tracker.cloud_tokens,
            local_requests: tracker.local_requests,
            cloud_requests: tracker.cloud_requests,
            estimated_cost: tracker.estimated_cost,
        }
    }
    
    async fn should_use_cloud(&self, prompt: &str) -> bool {
        let mode = self.mode.read().await;
        
        match *mode {
            ProviderMode::LocalOnly => false,
            ProviderMode::CloudOnly => true,
            ProviderMode::PreferLocal => {
                // Use cloud only if local is unavailable
                self.local_provider.is_none() || 
                !self.local_provider.as_ref().unwrap().is_available()
            }
            ProviderMode::PreferCloud => {
                // Use local only if cloud is unavailable
                self.cloud_provider.is_some() && 
                self.cloud_provider.as_ref().unwrap().is_available()
            }
            ProviderMode::CostOptimized => {
                // Simple heuristic: use cloud for complex queries
                self.is_complex_query(prompt)
            }
        }
    }
    
    fn is_complex_query(&self, prompt: &str) -> bool {
        // Simple heuristic for complexity
        prompt.len() > 500 || 
        prompt.contains("analyze") || 
        prompt.contains("explain") ||
        prompt.contains("compare") ||
        prompt.lines().count() > 10
    }
    
    async fn track_usage(&self, is_cloud: bool, tokens: Option<usize>) {
        let mut tracker = self.usage_tracker.write().await;
        
        if is_cloud {
            tracker.cloud_requests += 1;
            if let Some(t) = tokens {
                tracker.cloud_tokens += t;
                // Rough estimate: $0.01 per 1K tokens for GPT-4
                tracker.estimated_cost += (t as f64 / 1000.0) * 0.01;
            }
        } else {
            tracker.local_requests += 1;
            if let Some(t) = tokens {
                tracker.local_tokens += t;
            }
        }
    }
}

#[async_trait]
impl AIProvider for HybridProvider {
    async fn complete(&self, prompt: &str, context: AIContext) -> AIResult<AIResponse> {
        let use_cloud = self.should_use_cloud(prompt).await;
        
        let result = if use_cloud {
            if let Some(provider) = &self.cloud_provider {
                provider.complete(prompt, context).await
            } else {
                Err(AIError::ProviderUnavailable)
            }
        } else {
            if let Some(provider) = &self.local_provider {
                provider.complete(prompt, context).await
            } else {
                Err(AIError::ProviderUnavailable)
            }
        };
        
        // Track usage
        if let Ok(ref response) = result {
            self.track_usage(use_cloud, response.tokens_used).await;
        }
        
        result
    }
    
    async fn stream(&self, prompt: &str, context: AIContext) -> AIResult<StreamingResponse> {
        let use_cloud = self.should_use_cloud(prompt).await;
        
        if use_cloud {
            if let Some(provider) = &self.cloud_provider {
                provider.stream(prompt, context).await
            } else {
                Err(AIError::ProviderUnavailable)
            }
        } else {
            if let Some(provider) = &self.local_provider {
                provider.stream(prompt, context).await
            } else {
                Err(AIError::ProviderUnavailable)
            }
        }
    }
    
    fn name(&self) -> &str {
        "hybrid"
    }
    
    fn model(&self) -> &str {
        "hybrid-auto"
    }
    
    fn is_available(&self) -> bool {
        (self.local_provider.is_some() && self.local_provider.as_ref().unwrap().is_available()) ||
        (self.cloud_provider.is_some() && self.cloud_provider.as_ref().unwrap().is_available())
    }
}

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub local_tokens: usize,
    pub cloud_tokens: usize,
    pub local_requests: usize,
    pub cloud_requests: usize,
    pub estimated_cost: f64,
}