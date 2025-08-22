# AI Setup Guide for Cortex

Cortex comes with **AI enabled out-of-the-box**! We've bundled a demo API key so you can start using intelligent AI assistance immediately.

## 🚀 Quick Start

**No setup required!** Just run Cortex and press `Ctrl+A` to open the AI chat. The bundled Groq API provides fast, intelligent responses powered by Llama3-70b.

## Available AI Providers

### 1. Simple AI (Built-in)
- **Status**: Always available
- **Cost**: Free
- **Features**: Basic pattern-based responses for common file operations
- **No setup required** - works out of the box

### 2. Groq (Cloud - Pre-configured!)
- **Status**: ✅ Ready to use with bundled demo key
- **Cost**: Free
- **Features**: Fast, intelligent responses using Llama3-70b model
- **Note**: The demo key is shared and has rate limits. For unlimited personal use:
  1. Visit https://console.groq.com
  2. Sign up for a free account (takes 1 minute)
  3. Generate your own API key
  4. Set the environment variable:
     ```bash
     export GROQ_API_KEY="your-api-key-here"
     ```
  5. Restart Cortex

### 3. Ollama (Local)
- **Status**: Requires local installation
- **Cost**: Free (runs on your machine)
- **Features**: Privacy-focused, runs entirely offline
- **Setup**:
  1. Install Ollama from https://ollama.ai
  2. Pull a model: `ollama pull llama2`
  3. Ensure Ollama is running: `ollama serve`
  4. Cortex will automatically detect it

## Switching Between Providers

The AI chat in Cortex will automatically use the best available provider:
1. **Default**: Groq with bundled demo key (works immediately!)
2. If you set your own GROQ_API_KEY, it uses your personal key
3. If Ollama is running locally, it can be selected
4. Falls back to Simple AI if cloud services are unavailable

## Testing Your Setup

1. Launch Cortex: `./target/debug/cortex`
2. Press `Ctrl+A` to open the AI chat
3. Ask a question like "How do I organize my downloads folder?"
4. The response will indicate which provider is being used

## Troubleshooting

### Groq Not Working
- Verify your API key is correct
- Check internet connection
- Ensure the environment variable is set in the same terminal

### Ollama Not Detected
- Verify Ollama is running: `curl http://localhost:11434/api/tags`
- Check if a model is installed: `ollama list`
- Restart Cortex after starting Ollama

### Performance Tips
- Groq provides the fastest responses (typically <1 second)
- Ollama speed depends on your hardware
- Simple AI is instant but provides basic responses

## Privacy Notes
- **Simple AI**: No data leaves your machine
- **Groq**: Queries are sent to Groq's servers (see their privacy policy)
- **Ollama**: Completely local, no data leaves your machine

For the best free experience with intelligent responses, we recommend using Groq with their free API tier.