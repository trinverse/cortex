use cortex_core::ai::{AIManager, AIContext};
use cortex_core::config::AIConfig;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("Testing AI Providers in Cortex\n");
    println!("===============================\n");
    
    // Create AI manager
    let config = AIConfig::default();
    let manager = AIManager::new(config);
    
    // List available providers
    println!("Available AI Providers:");
    let providers = manager.list_providers();
    for provider in providers {
        println!("  - {}", provider);
    }
    
    // Test with a simple query
    let query = "How do I organize my downloads folder?";
    println!("\nTest Query: {}", query);
    println!("Response:\n");
    
    let context = AIContext::default();
    match manager.complete(query, context).await {
        Ok(response) => {
            println!("Provider: {}", response.provider);
            println!("Model: {}", response.model);
            if let Some(tokens) = response.tokens_used {
                println!("Tokens: {}", tokens);
            }
            println!("\n{}", response.content);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    
    // Check if Groq is available
    println!("\n---");
    if std::env::var("GROQ_API_KEY").is_ok() {
        println!("✓ Groq API key is configured");
        println!("  To use Groq, the app will automatically select it as the default provider");
    } else {
        println!("ℹ Groq API key not configured");
        println!("  To enable Groq (free, fast cloud AI):");
        println!("  1. Visit https://console.groq.com");
        println!("  2. Sign up for free and get an API key");
        println!("  3. Run: export GROQ_API_KEY=\"your-key-here\"");
    }
}