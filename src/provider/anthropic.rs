use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{Provider, TranslateRequest};

pub struct Anthropic {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl Anthropic {
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self {
            api_key,
            model,
            client,
        }
    }
}

#[async_trait]
impl Provider for Anthropic {
    fn name(&self) -> &str {
        "Anthropic"
    }

    async fn translate(&self, request: &TranslateRequest) -> Result<String> {
        let prompt = match &request.from {
            Some(from) => format!(
                "Translate this from {} to {}: {}",
                from, request.to, request.text
            ),
            None => format!("Translate this to {}: {}", request.to, request.text),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": self.model,
                "max_tokens": 4096,
                "system": "You are a translator that only gives the translated text.",
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            }))
            .send()
            .await
            .context("failed to send request to Anthropic")?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .context("failed to parse Anthropic response")?;

        if let Some(err) = body["error"]["message"].as_str() {
            anyhow::bail!("Anthropic API error ({}): {}", status, err);
        }

        body["content"][0]["text"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("empty response from Anthropic"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_provider() {
        let provider = Anthropic::new("key".into(), "model".into());
        assert_eq!(provider.name(), "Anthropic");
    }

    #[tokio::test]
    async fn translate_without_from_lang() {
        let provider = Anthropic::new("key".into(), "model".into());
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
        let provider = Anthropic::new("key".into(), "model".into());
        let request = TranslateRequest {
            text: "hello".into(),
            from: Some("en".into()),
            to: "pt".into(),
        };
        let result = provider.translate(&request).await;
        assert!(result.is_err());
    }
}
