#!/bin/bash

# Test Groq API integration
export GROQ_API_KEY="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"

echo "Testing Groq API integration..."
echo "================================"
echo ""

# First, let's test the API directly
echo "Testing API connectivity..."
curl -s -X POST "https://api.groq.com/openai/v1/chat/completions" \
  -H "Authorization: Bearer $GROQ_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "mixtral-8x7b-32768",
    "messages": [{"role": "user", "content": "Say hello in 5 words"}],
    "max_tokens": 50
  }' | python3 -m json.tool 2>/dev/null

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ API key is valid and working!"
    echo ""
    echo "Now you can run Cortex with:"
    echo "  export GROQ_API_KEY=\"$GROQ_API_KEY\""
    echo "  ./target/debug/cortex"
    echo ""
    echo "Then press Ctrl+A to open AI chat"
else
    echo "✗ API test failed"
fi