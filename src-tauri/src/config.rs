use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub claude_usage: String,
    pub claude_beta_header: String,
    pub codex_base: String,
    pub codex_usage_path: Option<String>,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            claude_usage: "https://api.anthropic.com/api/oauth/usage".to_string(),
            claude_beta_header: "oauth-2025-04-20".to_string(),
            codex_base: "https://chatgpt.com/backend-api".to_string(),
            codex_usage_path: None,
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
}
