# Chapter 1: Getting Started with Claude API

## 1.1 Introduction to Claude

Claude is Anthropic's most capable AI assistant, designed to be helpful, harmless, and honest. This guide will teach you how to architect applications that leverage Claude's capabilities effectively.

## 1.2 API Basics

To get started, you'll need an API key from Anthropic. Once you have it, you can make your first request:

```bash
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  --data '{"model":"claude-opus-4-5","max_tokens":1024,"messages":[{"role":"user","content":"Hello, Claude"}]}'
```

## 1.3 Response Format

Claude's responses include the generated text along with metadata:

- **content**: The generated text response
- **stop_reason**: Why generation stopped (e.g., `end_turn`, `max_tokens`)
- **model**: The model used for generation
- **usage**: Token usage statistics

## 1.4 Rate Limits

Be aware of rate limits when building production applications. The Haiku model has higher limits than Opus.

# Chapter 2: Building Architecture Patterns

## 2.1 System Prompts

System prompts define Claude's behavior and context. A well-designed system prompt includes:

- **Role definition**: What Claude should act as
- **Task description**: What Claude should accomplish
- **Constraints**: Rules Claude must follow
- **Output format**: How Claude should structure responses

## 2.2 Few-Shot Learning

Few-shot learning lets you teach Claude patterns through examples:

```json
{
  "messages": [
    {"role": "user", "content": "Extract the name: John Doe works at Acme Corp"},
    {"role": "assistant", "content": "Name: John Doe"},
    {"role": "user", "content": "Extract the name: Jane Smith is CEO of Global Inc"},
    {"role": "assistant", "content": "Name: Jane Smith"}
  ]
}
```

## 2.3 Tool Use

Claude can use tools for extended capabilities:

> **Tools available:** web_search, bash, read, write, edit, glob

Tools extend Claude's capabilities beyond pure text generation, allowing it to interact with the real world.

## 2.4 Error Handling

When building production systems, handle these common errors:

1. **Authentication errors**: Check your API key
2. **Rate limit errors**: Implement exponential backoff
3. **Invalid request errors**: Validate your request format
4. **Timeout errors**: Consider reducing max_tokens