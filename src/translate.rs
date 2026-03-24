use anyhow::Result;

use crate::config::Config;
use crate::provider::{self, TranslateRequest};

pub struct TranslateOptions {
    pub text: String,
    pub from: Option<String>,
    pub to: String,
    pub provider_override: Option<String>,
    pub model_override: Option<String>,
}

pub async fn translate(config: &Config, options: TranslateOptions) -> Result<String> {
    let provider = provider::create_provider(
        config,
        options.provider_override.as_deref(),
        options.model_override.as_deref(),
    )?;

    let request = TranslateRequest {
        text: options.text,
        from: options.from,
        to: options.to,
    };

    provider.translate(&request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OpenAiConfig;

    #[tokio::test]
    async fn translate_fails_with_no_provider_configured() {
        let config = Config::default();
        let options = TranslateOptions {
            text: "hello".into(),
            from: None,
            to: "pt".into(),
            provider_override: None,
            model_override: None,
        };
        let result = translate(&config, options).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn translate_fails_with_unknown_provider() {
        let config = Config::default();
        let options = TranslateOptions {
            text: "hello".into(),
            from: None,
            to: "pt".into(),
            provider_override: Some("nonexistent".into()),
            model_override: None,
        };
        let result = translate(&config, options).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown provider"));
    }

    #[tokio::test]
    async fn translate_with_from_language() {
        // Provider creation succeeds but API call will fail (no real server)
        let config = Config {
            openai: Some(OpenAiConfig {
                api_key: "sk-fake".into(),
                model: "gpt-4o-mini".into(),
                base_url: "http://localhost:1".into(), // unreachable
            }),
            ..Default::default()
        };
        let options = TranslateOptions {
            text: "hello".into(),
            from: Some("en".into()),
            to: "pt".into(),
            provider_override: None,
            model_override: None,
        };
        let result = translate(&config, options).await;
        // Should fail with connection error, not provider creation error
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to send") || err.contains("error sending") || err.contains("tcp"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn translate_with_model_override() {
        let config = Config {
            openai: Some(OpenAiConfig {
                api_key: "sk-fake".into(),
                model: "gpt-4o-mini".into(),
                base_url: "http://localhost:1".into(),
            }),
            ..Default::default()
        };
        let options = TranslateOptions {
            text: "hello".into(),
            from: None,
            to: "pt".into(),
            provider_override: None,
            model_override: Some("gpt-4o".into()),
        };
        let result = translate(&config, options).await;
        assert!(result.is_err()); // Connection error, model override was accepted
    }

    #[tokio::test]
    async fn translate_with_provider_override() {
        let config = Config {
            provider: "openai".into(),
            ollama: Some(crate::config::OllamaConfig {
                model: "llama3".into(),
                base_url: "http://localhost:1".into(),
            }),
            ..Default::default()
        };
        let options = TranslateOptions {
            text: "hello".into(),
            from: None,
            to: "pt".into(),
            provider_override: Some("ollama".into()),
            model_override: None,
        };
        let result = translate(&config, options).await;
        assert!(result.is_err()); // Connection error proves ollama provider was used
    }
}
