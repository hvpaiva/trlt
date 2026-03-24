pub mod anthropic;
pub mod ollama;
pub mod openai;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::Config;

pub struct TranslateRequest {
    pub text: String,
    pub from: Option<String>,
    pub to: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn translate(&self, request: &TranslateRequest) -> Result<String>;
    fn name(&self) -> &str;
}

pub fn create_provider(
    config: &Config,
    provider_override: Option<&str>,
    model_override: Option<&str>,
) -> Result<Box<dyn Provider>> {
    let provider_name = provider_override.unwrap_or(&config.provider);
    match provider_name {
        "openai" => {
            let cfg = config.openai.as_ref().ok_or_else(|| {
                anyhow::anyhow!("OpenAI not configured. Run: trlt config set openai.api_key <key>")
            })?;
            let model = model_override.unwrap_or(&cfg.model).to_string();
            Ok(Box::new(openai::OpenAi::new(
                cfg.api_key.clone(),
                model,
                cfg.base_url.clone(),
            )))
        }
        "anthropic" => {
            let cfg = config.anthropic.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Anthropic not configured. Run: trlt config set anthropic.api_key <key>"
                )
            })?;
            let model = model_override.unwrap_or(&cfg.model).to_string();
            Ok(Box::new(anthropic::Anthropic::new(
                cfg.api_key.clone(),
                model,
            )))
        }
        "ollama" => {
            let default_cfg = crate::config::OllamaConfig {
                model: "llama3".into(),
                base_url: "http://localhost:11434".into(),
            };
            let cfg = config.ollama.as_ref().unwrap_or(&default_cfg);
            let model = model_override.unwrap_or(&cfg.model).to_string();
            Ok(Box::new(ollama::Ollama::new(model, cfg.base_url.clone())))
        }
        other => anyhow::bail!("Unknown provider: {other}. Supported: openai, anthropic, ollama"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AnthropicConfig, OllamaConfig, OpenAiConfig};

    fn config_with_openai() -> Config {
        Config {
            provider: "openai".into(),
            openai: Some(OpenAiConfig {
                api_key: "sk-test".into(),
                model: "gpt-4o-mini".into(),
                base_url: "https://api.openai.com".into(),
            }),
            ..Default::default()
        }
    }

    fn config_with_all_providers() -> Config {
        Config {
            provider: "openai".into(),
            openai: Some(OpenAiConfig {
                api_key: "sk-test".into(),
                model: "gpt-4o-mini".into(),
                base_url: "https://api.openai.com".into(),
            }),
            anthropic: Some(AnthropicConfig {
                api_key: "sk-ant-test".into(),
                model: "claude-haiku-4-5-20251001".into(),
            }),
            ollama: Some(OllamaConfig {
                model: "llama3".into(),
                base_url: "http://localhost:11434".into(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn create_openai_provider() {
        let config = config_with_openai();
        let provider = create_provider(&config, None, None).unwrap();
        assert_eq!(provider.name(), "OpenAI");
    }

    #[test]
    fn create_openai_provider_with_model_override() {
        let config = config_with_openai();
        let provider = create_provider(&config, None, Some("gpt-4o")).unwrap();
        assert_eq!(provider.name(), "OpenAI");
    }

    #[test]
    fn create_anthropic_provider() {
        let config = config_with_all_providers();
        let provider = create_provider(&config, Some("anthropic"), None).unwrap();
        assert_eq!(provider.name(), "Anthropic");
    }

    #[test]
    fn create_anthropic_provider_with_model_override() {
        let config = config_with_all_providers();
        let provider = create_provider(
            &config,
            Some("anthropic"),
            Some("claude-sonnet-4-6-20250514"),
        )
        .unwrap();
        assert_eq!(provider.name(), "Anthropic");
    }

    #[test]
    fn create_ollama_provider() {
        let config = config_with_all_providers();
        let provider = create_provider(&config, Some("ollama"), None).unwrap();
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn create_ollama_provider_without_config_uses_defaults() {
        let config = Config::default();
        let provider = create_provider(&config, Some("ollama"), None).unwrap();
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn create_ollama_provider_with_model_override() {
        let config = Config::default();
        let provider = create_provider(&config, Some("ollama"), Some("mistral")).unwrap();
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn provider_override_takes_precedence() {
        let config = config_with_all_providers();
        let provider = create_provider(&config, Some("anthropic"), None).unwrap();
        assert_eq!(provider.name(), "Anthropic");
    }

    #[test]
    fn unknown_provider_returns_error() {
        let config = Config::default();
        match create_provider(&config, Some("gemini"), None) {
            Err(e) => assert!(e.to_string().contains("Unknown provider: gemini")),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn openai_not_configured_returns_error() {
        let config = Config::default();
        match create_provider(&config, Some("openai"), None) {
            Err(e) => assert!(e.to_string().contains("OpenAI not configured")),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn anthropic_not_configured_returns_error() {
        let config = Config::default();
        match create_provider(&config, Some("anthropic"), None) {
            Err(e) => assert!(e.to_string().contains("Anthropic not configured")),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn uses_config_provider_when_no_override() {
        let config = Config {
            provider: "anthropic".into(),
            anthropic: Some(AnthropicConfig {
                api_key: "sk-ant-test".into(),
                model: "claude-haiku-4-5-20251001".into(),
            }),
            ..Default::default()
        };
        let provider = create_provider(&config, None, None).unwrap();
        assert_eq!(provider.name(), "Anthropic");
    }
}
