use std::io::{self, IsTerminal, Write};

use anyhow::{Result, bail};

use crate::config::{AnthropicConfig, Config, OllamaConfig, OpenAiConfig};

fn prompt(question: &str) -> Result<String> {
    eprint!("{question}");
    io::stderr().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn prompt_default(question: &str, default: &str) -> Result<String> {
    let answer = prompt(&format!("{question} [{default}]: "))?;
    if answer.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(answer)
    }
}

fn prompt_secret(question: &str) -> Result<String> {
    let answer = prompt(&format!("{question}: "))?;
    if answer.is_empty() {
        bail!("API key is required.");
    }
    Ok(answer)
}

pub fn can_run_interactive() -> bool {
    io::stdin().is_terminal()
}

pub fn run_wizard() -> Result<Config> {
    eprintln!("Welcome to trlt! Let's set up your translator.\n");

    let provider = prompt_provider()?;

    let (openai, anthropic, ollama) = match provider.as_str() {
        "openai" => (Some(prompt_openai()?), None, None),
        "anthropic" => (None, Some(prompt_anthropic()?), None),
        "ollama" => (None, None, Some(prompt_ollama()?)),
        _ => unreachable!(),
    };

    eprintln!();
    let default_target = prompt_default("Default target language", "en")?;

    let config = Config {
        provider,
        default_target,
        openai,
        anthropic,
        ollama,
    };

    config.save_with_comments()?;

    let path = Config::config_path()?;
    eprintln!("\nConfig saved to {}", path.display());
    eprintln!("You can edit it anytime or use `trlt config set <key> <value>`.\n");

    Ok(config)
}

fn prompt_provider() -> Result<String> {
    eprintln!("Select a provider:");
    eprintln!("  1) OpenAI");
    eprintln!("  2) Anthropic (Claude)");
    eprintln!("  3) Ollama (local, no API key needed)");

    loop {
        let answer = prompt_default("Choice", "1")?;
        match answer.as_str() {
            "1" | "openai" => return Ok("openai".into()),
            "2" | "anthropic" => return Ok("anthropic".into()),
            "3" | "ollama" => return Ok("ollama".into()),
            _ => eprintln!("  Please enter 1, 2, or 3."),
        }
    }
}

fn prompt_openai() -> Result<OpenAiConfig> {
    eprintln!();
    let api_key = prompt_secret("OpenAI API key")?;
    let model = prompt_default("Model", "gpt-4o-mini")?;
    Ok(OpenAiConfig {
        api_key,
        model,
        base_url: "https://api.openai.com".into(),
    })
}

fn prompt_anthropic() -> Result<AnthropicConfig> {
    eprintln!();
    let api_key = prompt_secret("Anthropic API key")?;
    let model = prompt_default("Model", "claude-haiku-4-5-20251001")?;
    Ok(AnthropicConfig { api_key, model })
}

fn prompt_ollama() -> Result<OllamaConfig> {
    eprintln!();
    let model = prompt_default("Model", "llama3")?;
    let base_url = prompt_default("Ollama URL", "http://localhost:11434")?;
    Ok(OllamaConfig { model, base_url })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_run_interactive_returns_false_in_tests() {
        // In test context, stdin is piped, not a terminal
        assert!(!can_run_interactive());
    }
}
