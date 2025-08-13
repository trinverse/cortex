use cortex_core::ai::AIManager;
use cortex_core::ai::AIContext;
use cortex_core::config::AIConfig;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("Testing AI Manager...");
    
    // Create AI manager with default config
    let config = AIConfig::default();
    let manager = std::sync::Arc::new(AIManager::new(config));
    
    // Create context
    let context = AIContext::new(PathBuf::from("/tmp"));
    
    // Test a simple query
    let query = "help me list files";
    println!("Query: {}", query);
    
    match manager.complete(query, context).await {
        Ok(response) => {
            println!("Response from {}: {}", response.provider, response.content);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}