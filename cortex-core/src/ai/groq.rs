use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::time::Duration;
use super::{AIProvider, AIError, AIResult, AIResponse, StreamingResponse, AIContext};

#[derive(Debug, Clone)]
pub struct GroqProvider {
    api_key: Option<String>,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GroqResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

impl GroqProvider {
    pub fn new(api_key: Option<String>) -> Result<Self, AIError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AIError::ConfigurationError(e.to_string()))?;
        
        Ok(Self {
            api_key,
            model: "llama3-70b-8192".to_string(), // Updated free tier model
            client,
        })
    }
    
    pub fn with_demo_key() -> Result<Self, AIError> {
        // Demo key for initial testing - users should get their own free API key
        // from https://console.groq.com for production use
        let demo_key = "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";
        Self::new(Some(demo_key.to_string()))
    }
    
    #[allow(dead_code)]
    async fn check_availability(&self) -> bool {
        self.api_key.is_some()
    }
}

#[async_trait]
impl AIProvider for GroqProvider {
    async fn complete(&self, prompt: &str, _context: AIContext) -> AIResult<AIResponse> {
        let api_key = self.api_key.as_ref()
            .ok_or(AIError::ConfigurationError(
                "Groq API key not configured. Get a free key at https://console.groq.com".to_string()
            ))?;
        
        let request = GroqRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful file manager assistant. Help users with file operations, navigation, and organization tasks.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 1000,
            stream: false,
        };
        
        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::InvalidResponse(format!("API error: {}", error_text)));
        }
        
        let groq_response: GroqResponse = response.json().await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;
        
        let content = groq_response.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or(AIError::InvalidResponse("No response from API".to_string()))?;
        
        Ok(AIResponse {
            content,
            provider: self.name().to_string(),
            model: self.model.clone(),
            tokens_used: groq_response.usage.map(|u| u.total_tokens),
        })
    }
    
    async fn stream(&self, prompt: &str, _context: AIContext) -> AIResult<StreamingResponse> {
        let api_key = self.api_key.as_ref()
            .ok_or(AIError::ConfigurationError(
                "Groq API key not configured. Get a free key at https://console.groq.com".to_string()
            ))?;
        
        let request = GroqRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful file manager assistant. Help users with file operations, navigation, and organization tasks.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 1000,
            stream: true,
        };
        
        let (tx, rx) = mpsc::channel(100);
        let client = self.client.clone();
        let api_key = api_key.clone();
        
        tokio::spawn(async move {
            let response = match client
                .post("https://api.groq.com/openai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    let _ = tx.send(format!("Error: {}", e)).await;
                    return;
                }
            };
            
            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                let _ = tx.send(format!("API error: {}", error_text)).await;
                return;
            }
            
            let mut stream = response.bytes_stream();
            use futures_util::StreamExt;
            
            while let Some(chunk) = stream.next().await {
                if let Ok(bytes) = chunk {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let json_str = &line[6..];
                            if json_str == "[DONE]" {
                                break;
                            }
                            
                            if let Ok(stream_resp) = serde_json::from_str::<StreamResponse>(json_str) {
                                if let Some(choice) = stream_resp.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        let _ = tx.send(content.clone()).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        
        Ok(StreamingResponse { receiver: rx })
    }
    
    fn name(&self) -> &str {
        "groq"
    }
    
    fn model(&self) -> &str {
        &self.model
    }
    
    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }
    
    fn max_context_tokens(&self) -> usize {
        8192 // Llama3-70b model context window
    }
}