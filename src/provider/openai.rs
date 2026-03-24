use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{Provider, TranslateRequest};

pub struct OpenAi {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAi {
    pub fn new(api_key: String, model: String, base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self {
            api_key,
            model,
            base_url,
            client,
        }
    }
}

#[async_trait]
impl Provider for OpenAi {
    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn translate(&self, request: &TranslateRequest) -> Result<String> {
        let prompt = match &request.from {
            Some(from) => format!(
                "Translate this from {} to {}: {}",
                from, request.to, request.text
            ),
            None => format!("Translate this to {}: {}", request.to, request.text),
        };

        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.model,
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
            .context("failed to send request to OpenAI")?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .context("failed to parse OpenAI response")?;

        if let Some(err_msg) = body["error"]["message"].as_str() {
            anyhow::bail!("OpenAI API error ({}): {}", status, err_msg);
        }

        body["choices"][0]["message"]["content"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("empty response from OpenAI"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_provider() {
        let provider = OpenAi::new("key".into(), "model".into(), "http://localhost".into());
        assert_eq!(provider.name(), "OpenAI");
    }

    #[tokio::test]
    async fn translate_without_from_lang() {
        let provider = OpenAi::new("key".into(), "model".into(), "http://localhost:1".into());
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
        let provider = OpenAi::new("key".into(), "model".into(), "http://localhost:1".into());
        let request = TranslateRequest {
            text: "hello".into(),
            from: Some("en".into()),
            to: "pt".into(),
        };
        let result = provider.translate(&request).await;
        assert!(result.is_err());
    }
}
