# API Key Configuration Guide

## Overview
Cortex now includes a secure API key configuration dialog that allows you to safely configure API keys for various AI providers without hardcoding them in the repository.

## Supported Providers
- **Groq** - High-performance AI inference
- **Gemini** - Google's AI model
- **Anthropic** - Claude AI models  
- **OpenAI** - GPT models

## How to Access

### Method 1: Keyboard Shortcut
Press `Ctrl+K` to open the API key configuration dialog

### Method 2: Command Palette
1. Press `:` to enter command mode
2. Type `api-key` or `ai-key`
3. Press Enter

## Using the Dialog

### 1. Select Provider
- Press `Tab` to open the provider dropdown
- Use `↑↓` arrow keys to navigate between providers
- Press `Enter` to select a provider
- Providers with existing keys show a checkmark (✓)

### 2. Enter API Key
- Press `Enter` to start typing your API key
- The key is masked by default for security
- Press `Tab` to toggle show/hide the key
- Press `Enter` to save the key

### 3. Save and Exit
- Press `Enter` to save the API key to your configuration
- Press `Esc` at any time to cancel without saving

## Security Notes

⚠️ **Important Security Practices:**
- Never commit API keys to version control
- Keys are stored in your local configuration file
- Use environment variables for production deployments
- Each key is saved to `~/.config/cortex/config.toml` (or your configured location)

## Environment Variables

You can also set API keys via environment variables:
- `GROQ_API_KEY`
- `GEMINI_API_KEY`
- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`

Environment variables take precedence over configuration file settings.

## Configuration File

API keys are stored in the configuration file under the `[ai.cloud]` section:

```toml
[ai.cloud]
groq_api_key = "your-key-here"
gemini_api_key = "your-key-here"
anthropic_api_key = "your-key-here"
openai_api_key = "your-key-here"
```

## Troubleshooting

- If the AI manager doesn't recognize your new key, the application will automatically reinitialize the AI system after saving
- Check your configuration file permissions to ensure it's writable
- Verify your API key is valid with the provider's dashboard

## Getting API Keys

- **Groq**: https://console.groq.com/keys
- **Gemini**: https://makersuite.google.com/app/apikey
- **Anthropic**: https://console.anthropic.com/api-keys
- **OpenAI**: https://platform.openai.com/api-keys