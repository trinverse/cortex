use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::time::Duration;
use super::{AIProvider, AIError, AIResult, AIResponse, StreamingResponse, AIContext};

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: Client,
    _timeout: Duration,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    max_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
    _total_duration: Option<u64>,
    eval_count: Option<usize>,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Result<Self, AIError> {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
        let model = model.unwrap_or_else(|| "llama2".to_string());
        
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| AIError::ConfigurationError(e.to_string()))?;
        
        Ok(Self {
            base_url,
            model,
            client,
            _timeout: Duration::from_secs(60),
        })
    }
    
    #[allow(dead_code)]
    async fn check_availability(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn complete(&self, prompt: &str, context: AIContext) -> AIResult<AIResponse> {
        let full_prompt = format!("{}\n\n{}", context.to_prompt_context(), prompt);
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: full_prompt,
            stream: false,
            options: OllamaOptions {
                temperature: 0.7,
                top_p: 0.9,
                max_tokens: 2048,
            },
        };
        
        let url = format!("{}/api/generate", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(AIError::InvalidResponse(
                format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())
            ));
        }
        
        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;
        
        Ok(AIResponse {
            content: ollama_response.response,
            provider: self.name().to_string(),
            model: self.model.clone(),
            tokens_used: ollama_response.eval_count,
        })
    }
    
    async fn stream(&self, prompt: &str, context: AIContext) -> AIResult<StreamingResponse> {
        let full_prompt = format!("{}\n\n{}", context.to_prompt_context(), prompt);
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: full_prompt,
            stream: true,
            options: OllamaOptions {
                temperature: 0.7,
                top_p: 0.9,
                max_tokens: 2048,
            },
        };
        
        let url = format!("{}/api/generate", self.base_url);
        let (tx, rx) = mpsc::channel(100);
        
        // Spawn a task to handle streaming
        let client = self.client.clone();
        tokio::spawn(async move {
            let response = match client.post(&url).json(&request).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    let _ = tx.send(format!("Error: {}", e)).await;
                    return;
                }
            };
            
            let mut stream = response.bytes_stream();
            use futures::StreamExt;
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        if let Ok(text) = std::str::from_utf8(&bytes) {
                            // Parse JSON lines from the stream
                            for line in text.lines() {
                                if line.is_empty() {
                                    continue;
                                }
                                
                                if let Ok(resp) = serde_json::from_str::<OllamaResponse>(line) {
                                    if !resp.response.is_empty() {
                                        let _ = tx.send(resp.response).await;
                                    }
                                    
                                    if resp.done {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Stream error: {}", e)).await;
                        break;
                    }
                }
            }
        });
        
        Ok(StreamingResponse { receiver: rx })
    }
    
    fn name(&self) -> &str {
        "ollama"
    }
    
    fn model(&self) -> &str {
        &self.model
    }
    
    fn is_available(&self) -> bool {
        // Check if Ollama is running
        // This is a simplified check - in production, we'd cache this result
        true
    }
    
    fn max_context_tokens(&self) -> usize {
        4096 // Default for most Ollama models
    }
}