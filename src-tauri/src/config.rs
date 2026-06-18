use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointConfig {
    #[serde(default = "default_claude_usage_endpoint")]
    pub claude_usage: String,
    #[serde(default = "default_claude_refresh_endpoint")]
    pub claude_refresh: String,
    #[serde(default = "default_claude_beta_header")]
    pub claude_beta_header: String,
    #[serde(default = "default_codex_base_endpoint")]
    pub codex_base: String,
    #[serde(default = "default_codex_usage_path")]
    pub codex_usage_path: Option<String>,
    #[serde(default = "default_codex_refresh_endpoint")]
    pub codex_refresh: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_config_version")]
    pub version: u16,
    #[serde(default)]
    pub widgets: WidgetConfigSet,
    #[serde(default = "default_widget_scale")]
    pub widget_scale: f64,
    #[serde(default = "default_true")]
    pub grouped_widgets: bool,
    #[serde(default)]
    pub polling: PollingConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub click_through: bool,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub pomodoro: PomodoroConfig,
    #[serde(default)]
    pub endpoints: EndpointConfig,
    #[serde(default)]
    pub advanced: AdvancedConfig,
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollingConfig {
    #[serde(default = "default_polling_interval_sec")]
    pub interval_sec: u64,
    #[serde(default = "default_min_polling_interval_sec")]
    pub min_interval_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetConfigSet {
    #[serde(default = "default_claude_widget")]
    pub claude: WidgetConfig,
    #[serde(default = "default_codex_widget")]
    pub codex: WidgetConfig,
    #[serde(default = "default_pomodoro_widget")]
    pub pomodoro: WidgetConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub position: WindowPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPosition {
    #[serde(default = "default_window_x")]
    pub x: i32,
    #[serde(default = "default_window_y")]
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_notification_thresholds")]
    pub thresholds: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PomodoroConfig {
    #[serde(default = "default_focus_min")]
    pub focus_min: u16,
    #[serde(default = "default_break_min")]
    pub break_min: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedConfig {
    #[serde(default)]
    pub claude_token_source_override: Option<String>,
    #[serde(default = "default_codex_auth_path_string")]
    pub codex_auth_path: String,
}

fn default_config_version() -> u16 {
    1
}

fn default_widget_scale() -> f64 {
    1.0
}

fn default_polling_interval_sec() -> u64 {
    180
}

fn default_min_polling_interval_sec() -> u64 {
    120
}

fn default_true() -> bool {
    true
}

fn default_notification_thresholds() -> Vec<u8> {
    vec![80, 95]
}

fn default_focus_min() -> u16 {
    20
}

fn default_break_min() -> u16 {
    5
}

fn default_codex_auth_path_string() -> String {
    "~/.codex/auth.json".to_string()
}

fn default_window_x() -> i32 {
    120
}

fn default_window_y() -> i32 {
    80
}

fn default_claude_usage_endpoint() -> String {
    "https://api.anthropic.com/api/oauth/usage".to_string()
}

fn default_claude_refresh_endpoint() -> String {
    "https://platform.claude.com/v1/oauth/token".to_string()
}

fn default_claude_beta_header() -> String {
    "oauth-2025-04-20".to_string()
}

fn default_codex_base_endpoint() -> String {
    "https://chatgpt.com/backend-api/".to_string()
}

fn default_codex_usage_path() -> Option<String> {
    Some("wham/usage".to_string())
}

fn default_codex_refresh_endpoint() -> String {
    "https://auth.openai.com/oauth/token".to_string()
}

fn default_claude_widget() -> WidgetConfig {
    WidgetConfig {
        enabled: true,
        position: WindowPosition { x: 120, y: 80 },
    }
}

fn default_codex_widget() -> WidgetConfig {
    WidgetConfig {
        enabled: true,
        position: WindowPosition { x: 280, y: 80 },
    }
}

fn default_pomodoro_widget() -> WidgetConfig {
    WidgetConfig {
        enabled: true,
        position: WindowPosition { x: 440, y: 80 },
    }
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self { x: 120, y: 80 }
    }
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            position: WindowPosition::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            widgets: WidgetConfigSet::default(),
            widget_scale: 1.0,
            grouped_widgets: true,
            polling: PollingConfig::default(),
            notifications: NotificationConfig::default(),
            click_through: false,
            autostart: false,
            pomodoro: PomodoroConfig::default(),
            endpoints: EndpointConfig::default(),
            advanced: AdvancedConfig::default(),
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

impl Default for WidgetConfigSet {
    fn default() -> Self {
        Self {
            claude: default_claude_widget(),
            codex: default_codex_widget(),
            pomodoro: default_pomodoro_widget(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thresholds: vec![80, 95],
        }
    }
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            focus_min: 20,
            break_min: 5,
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            claude_token_source_override: None,
            codex_auth_path: "~/.codex/auth.json".to_string(),
        }
    }
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            claude_usage: default_claude_usage_endpoint(),
            claude_refresh: default_claude_refresh_endpoint(),
            claude_beta_header: default_claude_beta_header(),
            codex_base: default_codex_base_endpoint(),
            codex_usage_path: default_codex_usage_path(),
            codex_refresh: default_codex_refresh_endpoint(),
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
    #[error("config endpoint is invalid")]
    Endpoint(#[from] EndpointError),
    #[error("config contains token-like material")]
    TokenMaterial,
}

impl AppConfig {
    pub fn normalized(mut self) -> Self {
        self.grouped_widgets = true;
        self.widget_scale = self.widget_scale.clamp(0.5, 2.0);
        if self.polling.min_interval_sec < 120 {
            self.polling.min_interval_sec = 120;
        }
        if self.polling.interval_sec < self.polling.min_interval_sec {
            self.polling.interval_sec = self.polling.min_interval_sec;
        }
        self.notifications.thresholds = self
            .notifications
            .thresholds
            .into_iter()
            .filter(|threshold| (1..=100).contains(threshold))
            .collect();
        if self.notifications.thresholds.is_empty() {
            self.notifications.thresholds = NotificationConfig::default().thresholds;
        }
        self.notifications.thresholds.sort_unstable();
        self.notifications.thresholds.dedup();
        self.pomodoro.focus_min = self.pomodoro.focus_min.clamp(1, 180);
        self.pomodoro.break_min = self.pomodoro.break_min.clamp(1, 180);
        self
    }
}

pub fn load_or_create_config(path: &Path) -> Result<AppConfig, ConfigError> {
    match fs::read_to_string(path) {
        Ok(raw) => match serde_json::from_str::<AppConfig>(&raw) {
            Ok(config) => {
                let value: Value = serde_json::from_str(&raw)?;
                if config_value_contains_token_material(&value) {
                    return Err(ConfigError::TokenMaterial);
                }
                Ok(config.normalized())
            }
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
    let normalized = config.clone().normalized();
    validate_endpoint_config(&normalized.endpoints)?;
    let value = serde_json::to_value(&normalized)?;
    if config_value_contains_token_material(&value) {
        return Err(ConfigError::TokenMaterial);
    }
    let raw = serde_json::to_string_pretty(&normalized)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn config_value_contains_token_material(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            config_key_is_token_material(key) || config_value_contains_token_material(value)
        }),
        Value::Array(items) => items.iter().any(config_value_contains_token_material),
        Value::String(value) => config_string_looks_secret(value),
        _ => false,
    }
}

fn config_key_is_token_material(key: &str) -> bool {
    let normalized: String = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect();

    matches!(
        normalized.as_str(),
        "accesstoken"
            | "refreshtoken"
            | "idtoken"
            | "openaiapikey"
            | "apikey"
            | "xapikey"
            | "authorization"
            | "clientsecret"
            | "secret"
    )
}

fn config_string_looks_secret(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("bearer ")
        || trimmed.starts_with("sk-")
        || trimmed.starts_with("sk_")
        || trimmed.starts_with("eyJ")
}

pub fn validate_endpoint_config(config: &EndpointConfig) -> Result<(), EndpointError> {
    validate_endpoint_url(&config.claude_usage)?;
    validate_refresh_endpoint_url(&config.claude_refresh)?;
    validate_codex_base_url(&config.codex_base)?;
    let _ = join_codex_usage_url(config)?;
    validate_refresh_endpoint_url(&config.codex_refresh)?;
    Ok(())
}

pub fn validate_endpoint_url(url: &str) -> Result<(), EndpointError> {
    let parsed = reqwest::Url::parse(url).map_err(|_| EndpointError::InvalidUrl)?;
    if parsed.scheme() != "https" {
        return Err(EndpointError::InvalidScheme);
    }

    match parsed.host_str() {
        Some("api.anthropic.com") if parsed.path() == "/api/oauth/usage" => Ok(()),
        Some("chatgpt.com") if parsed.path() == "/backend-api/wham/usage" => Ok(()),
        Some("api.anthropic.com" | "chatgpt.com") => Err(EndpointError::HostNotAllowed),
        _ => Err(EndpointError::HostNotAllowed),
    }
}

fn validate_codex_base_url(url: &str) -> Result<(), EndpointError> {
    let parsed = reqwest::Url::parse(url).map_err(|_| EndpointError::InvalidUrl)?;
    if parsed.scheme() != "https" {
        return Err(EndpointError::InvalidScheme);
    }

    match parsed.host_str() {
        Some("chatgpt.com") if parsed.path() == "/backend-api/" => Ok(()),
        Some("chatgpt.com") => Err(EndpointError::HostNotAllowed),
        _ => Err(EndpointError::HostNotAllowed),
    }
}

pub fn validate_refresh_endpoint_url(url: &str) -> Result<(), EndpointError> {
    let parsed = reqwest::Url::parse(url).map_err(|_| EndpointError::InvalidUrl)?;
    if parsed.scheme() != "https" {
        return Err(EndpointError::InvalidScheme);
    }

    match parsed.host_str() {
        Some("platform.claude.com") if parsed.path() == "/v1/oauth/token" => Ok(()),
        Some("auth.openai.com") if parsed.path() == "/oauth/token" => Ok(()),
        Some("platform.claude.com" | "auth.openai.com") => Err(EndpointError::HostNotAllowed),
        _ => Err(EndpointError::HostNotAllowed),
    }
}

pub fn join_codex_usage_url(config: &EndpointConfig) -> Result<Option<String>, EndpointError> {
    validate_codex_base_url(&config.codex_base)?;
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
            validate_endpoint_url("https://chatgpt.com/backend-api/wham/usage"),
            Ok(())
        );
        assert_eq!(
            validate_endpoint_url("https://chatgpt.com/backend-api/example"),
            Err(EndpointError::HostNotAllowed)
        );
        assert_eq!(
            validate_endpoint_url("https://api.anthropic.com/api/oauth/profile"),
            Err(EndpointError::HostNotAllowed)
        );
    }

    #[test]
    fn allows_documented_refresh_hosts_separately() {
        assert_eq!(
            validate_refresh_endpoint_url("https://auth.openai.com/oauth/token"),
            Ok(())
        );
        assert_eq!(
            validate_refresh_endpoint_url("https://platform.claude.com/v1/oauth/token"),
            Ok(())
        );
        assert_eq!(
            validate_refresh_endpoint_url("https://platform.claude.com/v1/oauth/other"),
            Err(EndpointError::HostNotAllowed)
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
            "widgets": WidgetConfigSet::default(),
            "widget_scale": 1.0,
            "grouped_widgets": true,
            "polling": { "interval_sec": 180, "min_interval_sec": 120 },
            "notifications": NotificationConfig::default(),
            "click_through": false,
            "autostart": false,
            "pomodoro": PomodoroConfig::default(),
            "endpoints": EndpointConfig::default(),
            "advanced": AdvancedConfig::default(),
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
        assert!(config_value_contains_token_material(&json!({
            "advanced": { "apiKey": "synthetic" }
        })));
        assert!(config_value_contains_token_material(&json!({
            "endpoint": "sk-synthetic"
        })));
        assert!(config_value_contains_token_material(&json!({
            "endpoint": "eyJsynthetic.jwt"
        })));
        assert!(!config_value_contains_token_material(&json!({
            "endpoints": { "codex_base": "https://chatgpt.com/backend-api/" }
        })));
    }

    #[test]
    fn normalizes_m3_settings_without_touching_tokens() {
        let config = AppConfig {
            widget_scale: 5.0,
            grouped_widgets: true,
            polling: PollingConfig {
                interval_sec: 5,
                min_interval_sec: 1,
            },
            notifications: NotificationConfig {
                enabled: true,
                thresholds: vec![95, 0, 80, 95, 101],
            },
            pomodoro: PomodoroConfig {
                focus_min: 0,
                break_min: 999,
            },
            ..AppConfig::default()
        }
        .normalized();

        assert_eq!(config.widget_scale, 2.0);
        assert_eq!(config.polling.min_interval_sec, 120);
        assert_eq!(config.polling.interval_sec, 120);
        assert_eq!(config.notifications.thresholds, vec![80, 95]);
        assert_eq!(config.pomodoro.focus_min, 1);
        assert_eq!(config.pomodoro.break_min, 180);
    }

    #[test]
    fn accepts_partial_m3_config_with_widget_defaults() {
        let config: AppConfig = serde_json::from_value(json!({
            "version": 1,
            "widgets": {
                "codex": { "enabled": false }
            },
            "polling": { "interval_sec": 240 }
        }))
        .unwrap();

        assert!(config.widgets.claude.enabled);
        assert!(!config.widgets.codex.enabled);
        assert!(config.widgets.pomodoro.enabled);
        assert!(config.grouped_widgets);
        assert_eq!(config.widgets.codex.position, WindowPosition::default());
        assert_eq!(config.polling.min_interval_sec, 120);
        assert_eq!(config.endpoints, EndpointConfig::default());
    }

    #[test]
    fn rejects_token_material_on_load_and_write() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "widgets": WidgetConfigSet::default(),
                "widget_scale": 1.0,
                "grouped_widgets": true,
                "polling": { "interval_sec": 180, "min_interval_sec": 120 },
                "notifications": NotificationConfig::default(),
                "click_through": false,
                "autostart": false,
                "pomodoro": PomodoroConfig::default(),
                "endpoints": EndpointConfig::default(),
                "advanced": { "refresh_token": "synthetic" }
            }))
            .unwrap(),
        )
        .unwrap();

        assert!(matches!(
            load_or_create_config(&path),
            Err(ConfigError::TokenMaterial)
        ));

        let mut config = AppConfig::default();
        config.unknown.insert(
            "advanced".to_string(),
            json!({ "Authorization": "Bearer synthetic" }),
        );
        assert!(matches!(
            write_config(&temp.path().join("out.json"), &config),
            Err(ConfigError::TokenMaterial)
        ));
    }

    #[test]
    fn rejects_invalid_endpoints_before_persisting_config() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        let mut config = AppConfig::default();
        config.endpoints.codex_base = "https://example.com/backend-api/".to_string();

        assert!(matches!(
            write_config(&path, &config),
            Err(ConfigError::Endpoint(EndpointError::HostNotAllowed))
        ));
        assert!(!path.exists());
    }
}
