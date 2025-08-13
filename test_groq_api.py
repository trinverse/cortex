#!/usr/bin/env python3
import os
import json
import requests

api_key = "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"

headers = {
    "Authorization": f"Bearer {api_key}",
    "Content-Type": "application/json"
}

data = {
    "model": "llama3-70b-8192",
    "messages": [
        {"role": "system", "content": "You are a helpful file manager assistant."},
        {"role": "user", "content": "How do I organize files efficiently?"}
    ],
    "max_tokens": 200,
    "temperature": 0.7
}

print("Testing Groq API with Llama3-70b model...")
print("-" * 40)

response = requests.post(
    "https://api.groq.com/openai/v1/chat/completions",
    headers=headers,
    json=data
)

if response.status_code == 200:
    result = response.json()
    content = result['choices'][0]['message']['content']
    print("✓ API working successfully!")
    print("\nResponse:")
    print(content)
    print("\n" + "-" * 40)
    print(f"Model: {result['model']}")
    print(f"Tokens used: {result.get('usage', {}).get('total_tokens', 'N/A')}")
else:
    print(f"✗ API Error: {response.status_code}")
    print(response.text)