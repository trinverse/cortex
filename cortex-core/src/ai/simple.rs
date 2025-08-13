// Simple AI provider for testing without external dependencies
use async_trait::async_trait;
use crate::ai::{AIProvider, AIResult, AIResponse, StreamingResponse, AIContext};
use tokio::sync::mpsc;

pub struct SimpleAIProvider {
    name: String,
}

impl SimpleAIProvider {
    pub fn new() -> Self {
        Self {
            name: "simple-ai".to_string(),
        }
    }
    
    fn generate_response(&self, prompt: &str) -> String {
        // Log for debugging
        log::info!("SimpleAI: Generating response for: {}", prompt);
        
        // Simple pattern matching for file operations
        let prompt_lower = prompt.to_lowercase();
        
        if prompt_lower.contains("list") || prompt_lower.contains("show") {
            "To list files in the current directory, you can use:\n\
             - Press Enter on an empty command line to refresh\n\
             - Use arrow keys to navigate\n\
             - Press Space to mark files\n\
             - Press Tab to switch panels".to_string()
        } else if prompt_lower.contains("copy") {
            "To copy files:\n\
             1. Select files with Space\n\
             2. Press F5 or use /copy command\n\
             3. Navigate to destination\n\
             4. Confirm the operation".to_string()
        } else if prompt_lower.contains("move") || prompt_lower.contains("rename") {
            "To move or rename files:\n\
             1. Select the file\n\
             2. Press F6 or use /move command\n\
             3. Enter the new name or path\n\
             4. Press Enter to confirm".to_string()
        } else if prompt_lower.contains("delete") || prompt_lower.contains("remove") {
            "To delete files:\n\
             1. Select files with Space\n\
             2. Press F8 or Delete key\n\
             3. Confirm the deletion\n\
             WARNING: This operation cannot be undone!".to_string()
        } else if prompt_lower.contains("search") || prompt_lower.contains("find") {
            "To search for files:\n\
             1. Press /find or Ctrl+F\n\
             2. Enter your search pattern\n\
             3. Use wildcards like *.txt\n\
             4. Navigate results with arrow keys".to_string()
        } else if prompt_lower.contains("organize") {
            "Here's a suggestion to organize your files:\n\
             1. Create folders by categories (Documents, Images, Code)\n\
             2. Use /mkdir to create new directories\n\
             3. Select and move files to appropriate folders\n\
             4. Consider using date-based organization for archives".to_string()
        } else if prompt_lower.contains("help") {
            "Cortex AI Assistant can help you with:\n\
             - File operations (copy, move, delete)\n\
             - Directory navigation\n\
             - File organization strategies\n\
             - Search and filter operations\n\
             - Keyboard shortcuts\n\
             \n\
             Just ask me what you'd like to do!".to_string()
        } else {
            format!("I understand you want to: {}\n\
                    \n\
                    While I'm still learning, here are some things you can try:\n\
                    - Use arrow keys to navigate\n\
                    - Press F1 for help\n\
                    - Type /help for command list\n\
                    - Press Tab to switch between panels", prompt)
        }
    }
}

#[async_trait]
impl AIProvider for SimpleAIProvider {
    async fn complete(&self, prompt: &str, _context: AIContext) -> AIResult<AIResponse> {
        let response = self.generate_response(prompt);
        
        Ok(AIResponse {
            content: response,
            provider: self.name.clone(),
            model: "simple-patterns".to_string(),
            tokens_used: None,
        })
    }
    
    async fn stream(&self, prompt: &str, _context: AIContext) -> AIResult<StreamingResponse> {
        let response = self.generate_response(prompt);
        let (tx, rx) = mpsc::channel(100);
        
        // Simulate streaming by sending words one at a time
        tokio::spawn(async move {
            for word in response.split_whitespace() {
                let _ = tx.send(format!("{} ", word)).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
            }
        });
        
        Ok(StreamingResponse { receiver: rx })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn model(&self) -> &str {
        "simple-patterns"
    }
    
    fn is_available(&self) -> bool {
        true // Always available since it's built-in
    }
    
    fn max_context_tokens(&self) -> usize {
        1000
    }
}