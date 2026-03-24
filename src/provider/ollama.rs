use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{Provider, TranslateRequest};

pub struct Ollama {
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl Ollama {
    pub fn new(model: String, base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build HTTP client");
        Self {
            model,
            base_url,
            client,
        }
    }
}

#[async_trait]
impl Provider for Ollama {
    fn name(&self) -> &str {
        "Ollama"
    }

    async fn translate(&self, request: &TranslateRequest) -> Result<String> {
        let prompt = match &request.from {
            Some(from) => format!(
                "Translate this from {} to {}: {}",
                from, request.to, request.text
            ),
            None => format!("Translate this to {}: {}", request.to, request.text),
        };

        let url = format!("{}/api/chat", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "model": self.model,
                "stream": false,
                "messages": [{
                    "role": "system",
                    "content": "You are a translator that only gives the translated text."
                }, {
                    "role": "user",
                    "content": prompt
                }]
            }))
            .send()
            .await
            .context("failed to send request to Ollama")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama error ({}): {}", status, body);
        }

        let body: serde_json::Value = response
            .json()
            .await
            .context("failed to parse Ollama response")?;

        body["message"]["content"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("empty response from Ollama"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_provider() {
        let provider = Ollama::new("model".into(), "http://localhost:11434".into());
        assert_eq!(provider.name(), "Ollama");
    }

    #[tokio::test]
    async fn translate_without_from_lang() {
        let provider = Ollama::new("model".into(), "http://localhost:1".into());
        let request = TranslateRequest {
            text: "hello".into(),
            from: None,
            to: "pt".into(),
        };
        let result = provider.translate(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn translate_with_from_lang() {
        let provider = Ollama::new("model".into(), "http://localhost:1".into());
        let request = TranslateRequest {
            text: "hello".into(),
            from: Some("en".into()),
            to: "pt".into(),
        };
        let result = provider.translate(&request).await;
        assert!(result.is_err());
    }
}
