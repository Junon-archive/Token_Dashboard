use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub claude_usage: String,
    pub claude_beta_header: String,
    pub codex_base: String,
    pub codex_usage_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u16,
    pub widget_scale: f64,
    pub polling: PollingConfig,
    pub endpoints: EndpointConfig,
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollingConfig {
    pub interval_sec: u64,
    pub min_interval_sec: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            widget_scale: 1.0,
            polling: PollingConfig::default(),
            endpoints: EndpointConfig::default(),
            unknown: Map::new(),
        }
    }
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval_sec: 180,
            min_interval_sec: 120,
        }
    }
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            claude_usage: "https://api.anthropic.com/api/oauth/usage".to_string(),
            claude_beta_header: "oauth-2025-04-20".to_string(),
            codex_base: "https://chatgpt.com/backend-api/".to_string(),
            codex_usage_path: Some("wham/usage".to_string()),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EndpointError {
    #[error("endpoint URL is invalid")]
    InvalidUrl,
    #[error("endpoint scheme is not allowed")]
    InvalidScheme,
    #[error("endpoint host is not allowed")]
    HostNotAllowed,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config IO failed")]
    Io(#[from] std::io::Error),
    #[error("config JSON failed")]
    Json(#[from] serde_json::Error),
}

impl AppConfig {
    pub fn normalized(mut self) -> Self {
        if self.polling.min_interval_sec < 120 {
            self.polling.min_interval_sec = 120;
        }
        if self.polling.interval_sec < self.polling.min_interval_sec {
            self.polling.interval_sec = self.polling.min_interval_sec;
        }
        self
    }
}

pub fn load_or_create_config(path: &Path) -> Result<AppConfig, ConfigError> {
    match fs::read_to_string(path) {
        Ok(raw) => match serde_json::from_str::<AppConfig>(&raw) {
            Ok(config) => Ok(config.normalized()),
            Err(_) => {
                let backup = path.with_extension("json.bak");
                fs::rename(path, backup)?;
                let config = AppConfig::default();
                write_config(path, &config)?;
                Ok(config)
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let config = AppConfig::default();
            write_config(path, &config)?;
            Ok(config)
        }
        Err(error) => Err(ConfigError::Io(error)),
    }
}

pub fn write_config(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(&config.clone().normalized())?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn config_value_contains_token_material(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            matches!(
                key.as_str(),
                "access_token"
                    | "accessToken"
                    | "refresh_token"
                    | "refreshToken"
                    | "id_token"
                    | "idToken"
                    | "OPENAI_API_KEY"
                    | "Authorization"
                    | "authorization"
            ) || config_value_contains_token_material(value)
        }),
        Value::Array(items) => items.iter().any(config_value_contains_token_material),
        Value::String(value) => value.starts_with("Bearer "),
        _ => false,
    }
}

pub fn validate_endpoint_url(url: &str) -> Result<(), EndpointError> {
    let parsed = reqwest::Url::parse(url).map_err(|_| EndpointError::InvalidUrl)?;
    if parsed.scheme() != "https" {
        return Err(EndpointError::InvalidScheme);
    }

    match parsed.host_str() {
        Some("api.anthropic.com" | "chatgpt.com") => Ok(()),
        _ => Err(EndpointError::HostNotAllowed),
    }
}

pub fn validate_refresh_endpoint_url(url: &str) -> Result<(), EndpointError> {
    let parsed = reqwest::Url::parse(url).map_err(|_| EndpointError::InvalidUrl)?;
    if parsed.scheme() != "https" {
        return Err(EndpointError::InvalidScheme);
    }

    match parsed.host_str() {
        Some("api.anthropic.com" | "auth.openai.com") => Ok(()),
        _ => Err(EndpointError::HostNotAllowed),
    }
}

pub fn join_codex_usage_url(config: &EndpointConfig) -> Result<Option<String>, EndpointError> {
    validate_endpoint_url(&config.codex_base)?;
    let Some(path) = &config.codex_usage_path else {
        return Ok(None);
    };

    let base = reqwest::Url::parse(&config.codex_base).map_err(|_| EndpointError::InvalidUrl)?;
    let joined = base.join(path).map_err(|_| EndpointError::InvalidUrl)?;
    let url = joined.to_string();
    validate_endpoint_url(&url)?;
    Ok(Some(url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn allows_only_documented_https_hosts() {
        assert_eq!(
            validate_endpoint_url("https://api.anthropic.com/api/oauth/usage"),
            Ok(())
        );
        assert_eq!(
            validate_endpoint_url("https://chatgpt.com/backend-api/example"),
            Ok(())
        );
    }

    #[test]
    fn allows_documented_refresh_hosts_separately() {
        assert_eq!(
            validate_refresh_endpoint_url("https://auth.openai.com/oauth/token"),
            Ok(())
        );
        assert_eq!(
            validate_refresh_endpoint_url("https://chatgpt.com/backend-api/wham/usage"),
            Err(EndpointError::HostNotAllowed)
        );
    }

    #[test]
    fn rejects_endpoint_override_exfiltration_hosts() {
        assert_eq!(
            validate_endpoint_url("http://api.anthropic.com/api/oauth/usage"),
            Err(EndpointError::InvalidScheme)
        );
        assert_eq!(
            validate_endpoint_url("https://api.anthropic.com.evil.test/api/oauth/usage"),
            Err(EndpointError::HostNotAllowed)
        );
        assert_eq!(
            validate_endpoint_url("https://127.0.0.1:8000/usage"),
            Err(EndpointError::HostNotAllowed)
        );
        assert_eq!(
            validate_endpoint_url("https://localhost:8000/usage"),
            Err(EndpointError::HostNotAllowed)
        );
    }

    #[test]
    fn clamps_polling_interval_to_minimum() {
        let config = AppConfig {
            polling: PollingConfig {
                interval_sec: 1,
                min_interval_sec: 1,
            },
            ..AppConfig::default()
        }
        .normalized();

        assert_eq!(config.polling.min_interval_sec, 120);
        assert_eq!(config.polling.interval_sec, 120);
    }

    #[test]
    fn preserves_unknown_top_level_keys_on_roundtrip() {
        let config: AppConfig = serde_json::from_value(json!({
            "version": 1,
            "widget_scale": 1.0,
            "polling": { "interval_sec": 180, "min_interval_sec": 120 },
            "endpoints": EndpointConfig::default(),
            "future_key": { "enabled": true }
        }))
        .unwrap();

        assert_eq!(config.unknown["future_key"]["enabled"], true);
        let serialized = serde_json::to_value(&config).unwrap();
        assert_eq!(serialized["future_key"]["enabled"], true);
    }

    #[test]
    fn recovers_corrupted_config_with_backup() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(&path, "{not-json").unwrap();

        let config = load_or_create_config(&path).unwrap();

        assert_eq!(config, AppConfig::default());
        assert!(path.exists());
        assert!(temp.path().join("config.json.bak").exists());
    }

    #[test]
    fn detects_token_material_in_config_values() {
        assert!(config_value_contains_token_material(&json!({
            "advanced": { "access_token": "synthetic" }
        })));
        assert!(config_value_contains_token_material(&json!({
            "headers": { "Authorization": "Bearer synthetic" }
        })));
        assert!(!config_value_contains_token_material(&json!({
            "endpoints": { "codex_base": "https://chatgpt.com/backend-api/" }
        })));
    }
}
