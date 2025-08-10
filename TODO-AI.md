# TODO: AI Features Implementation

## Quick Start for Tomorrow

### 1. First Priority - Local AI with Ollama
```bash
# Install Ollama locally
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model (3.2B model, fast and efficient)
ollama pull llama3.2

# Test it's working
ollama run llama3.2 "Hello, can you help organize files?"
```

### 2. Add AI Dependencies
```toml
# Add to cortex-core/Cargo.toml
[dependencies]
# For Ollama integration
reqwest = { version = "0.11", features = ["json", "stream"] }
serde_json = "1.0"
tokio-stream = "0.1"

# For OpenAI/Anthropic (optional, later)
async-openai = "0.20"
```

### 3. Basic AI Module Structure
```rust
// cortex-core/src/ai/mod.rs
pub mod provider;
pub mod commands;
pub mod analyzer;

use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait AIProvider {
    async fn complete(&self, prompt: &str) -> Result<String>;
    async fn stream_complete(&self, prompt: &str) -> Result<Box<dyn Stream<Item = String>>>;
}

pub struct AIManager {
    provider: Box<dyn AIProvider>,
    config: AIConfig,
}

// cortex-core/src/ai/ollama.rs
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}
```

### 4. Natural Language Commands
```rust
// Example commands to implement
"organize files by type" -> Creates folders and moves files
"find duplicate files" -> Scans and identifies duplicates
"clean up downloads folder" -> Moves old files to archive
"rename photos by date" -> Renames based on EXIF data
"compress large files" -> Identifies and compresses big files
"backup important documents" -> Copies critical files to backup
```

### 5. UI Integration Points

#### Command Input
- `/ai <natural language command>` - Execute AI command
- `/ai-chat` - Open AI assistant chat panel
- `/ai-suggest` - Get AI suggestions for current directory

#### Visual Components
```rust
// New dialog for AI interactions
pub struct AIAssistantDialog {
    pub input: String,
    pub messages: Vec<Message>,
    pub is_processing: bool,
    pub suggestions: Vec<String>,
}
```

## Implementation Steps

### Morning (2-3 hours)
1. [ ] Set up Ollama locally
2. [ ] Create `cortex-core/src/ai/` module
3. [ ] Implement OllamaProvider
4. [ ] Add `/ai` command handler
5. [ ] Test basic prompt completion

### Afternoon (2-3 hours)
1. [ ] Implement natural language parser
2. [ ] Add file operation executor
3. [ ] Create AI assistant dialog
4. [ ] Add streaming response support
5. [ ] Test with real file operations

### If Time Permits
1. [ ] Add OpenAI provider option
2. [ ] Implement caching layer
3. [ ] Add batch operations
4. [ ] Create suggestion engine

## Example Implementation

```rust
// Quick test command for tomorrow
async fn handle_ai_command(&mut self, command: &str) -> Result<()> {
    // Parse natural language
    let ai_response = self.ai_manager.complete(&format!(
        "Convert this command to file operations: {}
        Respond with JSON: {{\"action\": \"move|copy|delete|rename\", \"files\": [...], \"destination\": \"...\"}}",
        command
    )).await?;
    
    // Parse JSON response
    let operation: FileOperation = serde_json::from_str(&ai_response)?;
    
    // Show preview to user
    self.dialog = Some(Dialog::AIPreview(AIPreviewDialog::new(operation)));
    
    Ok(())
}
```

## Packaging Quick Start

### macOS (Priority)
```bash
# Tomorrow morning, after AI features:
cargo build --release
cargo bundle --release  # Creates .app bundle

# Quick DMG creation
mkdir dist
cp -r target/release/bundle/osx/Cortex.app dist/
hdiutil create -volname "Cortex" -srcfolder dist -ov -format UDZO Cortex.dmg
```

### Windows (If time)
```bash
# Install cargo-wix
cargo install cargo-wix

# Initialize WiX
cargo wix init

# Build MSI
cargo wix --nocapture
```

## Testing Checklist
- [ ] Test Ollama connection
- [ ] Test natural language parsing
- [ ] Test file operation preview
- [ ] Test error handling
- [ ] Test without AI provider (graceful fallback)
- [ ] Test packaging on target platform
- [ ] Test auto-update mechanism

## Privacy & Security Notes
- Default to local AI (Ollama)
- Never send file contents without permission
- Clear indication when AI is processing
- Option to disable AI features completely
- Audit log for AI operations

## Resources for Tomorrow
- [Ollama API Docs](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)
- [cargo-wix](https://github.com/volks73/cargo-wix)
- [create-dmg](https://github.com/create-dmg/create-dmg)

## MVP Goal for Tomorrow
✅ Working natural language file operations with Ollama
✅ Basic packaged application for macOS
✅ Simple auto-update check

Let's start with local AI (Ollama) for privacy and no API keys needed!