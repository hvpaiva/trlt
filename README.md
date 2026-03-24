# trlt

A multi-provider CLI translator powered by LLMs.

[![CI](https://github.com/hvpaiva/trlt/actions/workflows/ci.yml/badge.svg)](https://github.com/hvpaiva/trlt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/hvpaiva/trlt/graph/badge.svg)](https://codecov.io/gh/hvpaiva/trlt)
[![Crates.io](https://img.shields.io/crates/v/trlt.svg)](https://crates.io/crates/trlt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Translate text between languages from the terminal using OpenAI, Anthropic, or Ollama.

## Features

- **Multi-provider** -- switch between OpenAI, Anthropic, and Ollama (local)
- **Flexible input** -- pass text directly, read from a file, or pipe via stdin
- **Auto-detect source language** -- or specify it explicitly
- **Configurable defaults** -- default target language, provider, and model
- **Shell completions** -- for Bash, Zsh, Fish, and PowerShell
- **Clipboard support** -- opt-in copy with `--copy`

## Installation

### From crates.io

```bash
cargo install trlt
```

### From source

```bash
git clone https://github.com/hvpaiva/trlt.git
cd trlt
cargo install --path .
```

## Quick Start

```bash
# Initialize configuration
trlt init

# Set your API key
trlt config set openai.api_key sk-your-key-here

# Translate
trlt translate "Hallo Welt" --to en
# Hello World
```

## Usage

### Translate text

```bash
# Direct text input
trlt translate "Bonjour le monde" --to en

# With explicit source language
trlt translate "Hello world" --from en --to pt

# From a file
trlt translate document.txt --to ja

# From stdin
cat article.txt | trlt translate --to es

# Save to file
trlt translate "Hello" --to fr --output output.txt

# Copy to clipboard
trlt translate "Hello" --to de --copy
```

### Switch providers

```bash
# Use Anthropic
trlt translate "Hello" --to pt --provider anthropic

# Use a local Ollama model
trlt translate "Hello" --to pt --provider ollama --model llama3

# Override model for a single command
trlt translate "Hello" --to pt --model gpt-4o
```

### Manage configuration

```bash
# Show current config
trlt config show

# Change default target language
trlt config set default_target pt

# Change default provider
trlt config set provider anthropic

# Set Anthropic API key
trlt config set anthropic.api_key sk-ant-your-key

# Configure Ollama endpoint
trlt config set ollama.base_url http://localhost:11434
trlt config set ollama.model mistral
```

### Shell completions

```bash
# Bash
trlt completions bash > ~/.local/share/bash-completion/completions/trlt

# Zsh
trlt completions zsh > ~/.zfunc/_trlt

# Fish
trlt completions fish > ~/.config/fish/completions/trlt.fish
```

## Configuration

Configuration is stored at `~/.config/trlt/config.toml`:

```toml
provider = "openai"
default_target = "en"

[openai]
api_key = "sk-..."
model = "gpt-4o-mini"
base_url = "https://api.openai.com"  # optional, for compatible endpoints

[anthropic]
api_key = "sk-ant-..."
model = "claude-haiku-4-5-20251001"

[ollama]
model = "llama3"
base_url = "http://localhost:11434"
```

Only the active provider's section needs to be configured.

## Providers

| Provider | Auth | Models | Notes |
|----------|------|--------|-------|
| OpenAI | API key | gpt-4o-mini (default), gpt-4o, etc. | Also works with compatible endpoints (Azure, LM Studio) via `base_url` |
| Anthropic | API key | claude-haiku-4-5-20251001 (default), claude-sonnet-4-6-20250514, etc. | Uses the Messages API |
| Ollama | None | llama3 (default), mistral, etc. | Runs locally, no API key needed |

## Development

```bash
just setup    # install tools, hooks, and dependencies
just check    # fmt + lint + test + audit
just run      # run the CLI
just coverage # HTML coverage report
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide.

## License

[MIT](LICENSE)
