use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_target")]
    pub default_target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openai: Option<OpenAiConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anthropic: Option<AnthropicConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    #[serde(default = "default_openai_model")]
    pub model: String,
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    #[serde(default = "default_anthropic_model")]
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_model")]
    pub model: String,
    #[serde(default = "default_ollama_base_url")]
    pub base_url: String,
}

fn default_provider() -> String {
    "openai".into()
}
fn default_target() -> String {
    "en".into()
}
fn default_openai_model() -> String {
    "gpt-4o-mini".into()
}
fn default_openai_base_url() -> String {
    "https://api.openai.com".into()
}
fn default_anthropic_model() -> String {
    "claude-haiku-4-5-20251001".into()
}
fn default_ollama_model() -> String {
    "llama3".into()
}
fn default_ollama_base_url() -> String {
    "http://localhost:11434".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            default_target: default_target(),
            openai: None,
            anthropic: None,
            ollama: None,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("could not determine config directory")?
            .join("trlt");
        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    fn legacy_config_path() -> Result<PathBuf> {
        let dir = dirs::config_dir().context("could not determine config directory")?;
        Ok(dir.join("trlt.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let contents = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let config: Config = toml::from_str(&contents)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            return Ok(config);
        }

        let legacy = Self::legacy_config_path()?;
        if legacy.exists() {
            return Self::migrate_legacy(&legacy);
        }

        anyhow::bail!(
            "No configuration found. Run `trlt init` to create one at {}",
            path.display()
        )
    }

    fn migrate_legacy(legacy: &PathBuf) -> Result<Self> {
        #[derive(Deserialize)]
        struct LegacyConfig {
            api_key: String,
            model: String,
        }

        let contents = std::fs::read_to_string(legacy)
            .with_context(|| format!("failed to read legacy config {}", legacy.display()))?;
        let old: LegacyConfig = toml::from_str(&contents)?;

        let config = Config {
            provider: "openai".into(),
            default_target: "en".into(),
            openai: Some(OpenAiConfig {
                api_key: old.api_key,
                model: old.model,
                base_url: default_openai_base_url(),
            }),
            ..Default::default()
        };

        config.save()?;
        std::fs::remove_file(legacy)?;

        let new_path = Self::config_path()?;
        eprintln!(
            "Migrated config from {} to {}",
            legacy.display(),
            new_path.display()
        );
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        let toml_str = toml::to_string(self)?;
        let mut table: toml::Value = toml_str.parse()?;
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &mut table;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current
                    .as_table_mut()
                    .context("config is not a table")?
                    .insert(part.to_string(), toml::Value::String(value.to_string()));
            } else {
                current = current
                    .as_table_mut()
                    .context("config is not a table")?
                    .entry(part.to_string())
                    .or_insert(toml::Value::Table(toml::map::Map::new()));
            }
        }

        *self = table.try_into()?;
        Ok(())
    }

    pub fn to_toml_pretty(&self) -> Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_values() {
        let config = Config::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.default_target, "en");
        assert!(config.openai.is_none());
        assert!(config.anthropic.is_none());
        assert!(config.ollama.is_none());
    }

    #[test]
    fn serialize_minimal_config() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("provider = \"openai\""));
        assert!(toml_str.contains("default_target = \"en\""));
        assert!(!toml_str.contains("[openai]"));
    }

    #[test]
    fn serialize_config_with_openai() {
        let config = Config {
            openai: Some(OpenAiConfig {
                api_key: "sk-test".into(),
                model: "gpt-4o".into(),
                base_url: "https://api.openai.com".into(),
            }),
            ..Default::default()
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[openai]"));
        assert!(toml_str.contains("api_key = \"sk-test\""));
    }

    #[test]
    fn deserialize_minimal_toml() {
        let toml_str = r#"
            provider = "anthropic"
            default_target = "pt"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.default_target, "pt");
        assert!(config.openai.is_none());
    }

    #[test]
    fn deserialize_empty_toml_uses_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.default_target, "en");
    }

    #[test]
    fn deserialize_full_config() {
        let toml_str = r#"
            provider = "openai"
            default_target = "en"

            [openai]
            api_key = "sk-123"
            model = "gpt-4o-mini"

            [anthropic]
            api_key = "sk-ant-456"

            [ollama]
            model = "llama3"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.openai.as_ref().unwrap().api_key, "sk-123");
        assert_eq!(config.anthropic.as_ref().unwrap().api_key, "sk-ant-456");
        assert_eq!(
            config.anthropic.as_ref().unwrap().model,
            "claude-haiku-4-5-20251001"
        );
        assert_eq!(config.ollama.as_ref().unwrap().model, "llama3");
        assert_eq!(
            config.ollama.as_ref().unwrap().base_url,
            "http://localhost:11434"
        );
    }

    #[test]
    fn roundtrip_serialization() {
        let config = Config {
            provider: "anthropic".into(),
            default_target: "pt-BR".into(),
            openai: Some(OpenAiConfig {
                api_key: "sk-test".into(),
                model: "gpt-4o".into(),
                base_url: "https://custom.api.com".into(),
            }),
            anthropic: Some(AnthropicConfig {
                api_key: "sk-ant-test".into(),
                model: "claude-sonnet-4-6-20250514".into(),
            }),
            ollama: Some(OllamaConfig {
                model: "mistral".into(),
                base_url: "http://localhost:11434".into(),
            }),
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.provider, "anthropic");
        assert_eq!(deserialized.default_target, "pt-BR");
        assert_eq!(deserialized.openai.as_ref().unwrap().api_key, "sk-test");
        assert_eq!(
            deserialized.anthropic.as_ref().unwrap().api_key,
            "sk-ant-test"
        );
        assert_eq!(deserialized.ollama.as_ref().unwrap().model, "mistral");
    }

    #[test]
    fn set_value_top_level_key() {
        let mut config = Config::default();
        config.set_value("provider", "anthropic").unwrap();
        assert_eq!(config.provider, "anthropic");
    }

    #[test]
    fn set_value_nested_key_creates_section() {
        let mut config = Config::default();
        config.set_value("openai.api_key", "sk-new").unwrap();
        assert_eq!(config.openai.as_ref().unwrap().api_key, "sk-new");
    }

    #[test]
    fn set_value_nested_key_updates_existing() {
        let mut config = Config {
            openai: Some(OpenAiConfig {
                api_key: "sk-old".into(),
                model: "gpt-4o-mini".into(),
                base_url: "https://api.openai.com".into(),
            }),
            ..Default::default()
        };
        config.set_value("openai.model", "gpt-4o").unwrap();
        assert_eq!(config.openai.as_ref().unwrap().model, "gpt-4o");
        assert_eq!(config.openai.as_ref().unwrap().api_key, "sk-old");
    }

    #[test]
    fn set_value_default_target() {
        let mut config = Config::default();
        config.set_value("default_target", "ja").unwrap();
        assert_eq!(config.default_target, "ja");
    }

    #[test]
    fn to_toml_pretty_works() {
        let config = Config::default();
        let output = config.to_toml_pretty().unwrap();
        assert!(output.contains("provider"));
        assert!(output.contains("default_target"));
    }

    #[test]
    fn openai_config_defaults() {
        let toml_str = r#"api_key = "sk-test""#;
        let config: OpenAiConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.model, "gpt-4o-mini");
        assert_eq!(config.base_url, "https://api.openai.com");
    }

    #[test]
    fn anthropic_config_defaults() {
        let toml_str = r#"api_key = "sk-ant-test""#;
        let config: AnthropicConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.model, "claude-haiku-4-5-20251001");
    }

    #[test]
    fn ollama_config_defaults() {
        let config: OllamaConfig = toml::from_str("").unwrap();
        assert_eq!(config.model, "llama3");
        assert_eq!(config.base_url, "http://localhost:11434");
    }

    // Note: Tests for load(), save(), migrate_legacy(), config_path(), config_dir()
    // are covered by integration tests in tests/cli.rs because they require
    // setting XDG_CONFIG_HOME, which is unsafe in Rust 2024 edition
    // (env::set_var is unsafe due to potential data races).

    #[test]
    fn set_value_creates_deeply_nested() {
        let mut config = Config::default();
        config.set_value("anthropic.api_key", "sk-deep").unwrap();
        assert_eq!(config.anthropic.as_ref().unwrap().api_key, "sk-deep");
    }

    #[test]
    fn set_value_ollama_base_url() {
        let mut config = Config {
            ollama: Some(OllamaConfig {
                model: "llama3".into(),
                base_url: "http://localhost:11434".into(),
            }),
            ..Default::default()
        };
        config
            .set_value("ollama.base_url", "http://remote:11434")
            .unwrap();
        assert_eq!(
            config.ollama.as_ref().unwrap().base_url,
            "http://remote:11434"
        );
    }

    #[test]
    fn skip_serializing_none_sections() {
        let config = Config {
            openai: Some(OpenAiConfig {
                api_key: "sk-test".into(),
                model: "gpt-4o".into(),
                base_url: "https://api.openai.com".into(),
            }),
            ..Default::default()
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[openai]"));
        assert!(!toml_str.contains("[anthropic]"));
        assert!(!toml_str.contains("[ollama]"));
    }
}
