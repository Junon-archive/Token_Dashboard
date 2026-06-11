use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use crate::{
    config::{validate_refresh_endpoint_url, EndpointError},
    providers::ProviderError,
    refresh_cache::TokenRefreshCache,
    snapshot::{ProviderKind, UsageState},
};

const CLAUDE_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const CLAUDE_SCOPES: &[&str] = &[
    "user:profile",
    "user:inference",
    "user:sessions:claude_code",
    "user:mcp_servers",
    "user:file_upload",
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RefreshPolicyError {
    #[error("refresh endpoint is not documented")]
    MissingEndpoint,
    #[error("refresh endpoint is not allowed: {0}")]
    Endpoint(#[from] EndpointError),
}

#[derive(Clone, PartialEq, Eq)]
pub struct RefreshSuccess {
    pub provider: ProviderKind,
    pub access_token: String,
    pub refreshed_at: DateTime<Utc>,
}

impl fmt::Debug for RefreshSuccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefreshSuccess")
            .field("provider", &self.provider)
            .field("access_token", &"[REDACTED]")
            .field("refreshed_at", &self.refreshed_at)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct OAuthRefreshRequest {
    pub grant_type: String,
    pub refresh_token: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl fmt::Debug for OAuthRefreshRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OAuthRefreshRequest")
            .field("grant_type", &self.grant_type)
            .field("refresh_token", &"[REDACTED]")
            .field("client_id", &self.client_id)
            .field("scope", &self.scope)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct OAuthRefreshToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub scopes: Option<Vec<String>>,
}

impl fmt::Debug for OAuthRefreshToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OAuthRefreshToken")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("expires_in", &self.expires_in)
            .field("scopes", &self.scopes)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct RefreshResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Error)]
pub enum RefreshHttpError {
    #[error("refresh endpoint rejected: {0}")]
    Endpoint(#[from] EndpointError),
    #[error("invalid HTTP header")]
    InvalidHeader,
    #[error("refresh network request failed")]
    Network,
}

#[async_trait::async_trait]
pub trait RefreshHttpClient: Send + Sync {
    async fn post_json(
        &self,
        url: &str,
        body: &OAuthRefreshRequest,
    ) -> Result<RefreshResponse, RefreshHttpError>;
}

#[derive(Debug, Default, Clone)]
pub struct ReqwestRefreshHttpClient {
    client: reqwest::Client,
}

#[async_trait::async_trait]
impl RefreshHttpClient for ReqwestRefreshHttpClient {
    async fn post_json(
        &self,
        url: &str,
        body: &OAuthRefreshRequest,
    ) -> Result<RefreshResponse, RefreshHttpError> {
        validate_refresh_endpoint_url(url)?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("token-dashboard/0.1.0"),
        );

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|_| RefreshHttpError::Network)?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|_| RefreshHttpError::Network)?;

        Ok(RefreshResponse { status, body })
    }
}

pub fn validate_refresh_endpoint(endpoint: Option<&str>) -> Result<(), RefreshPolicyError> {
    let endpoint = endpoint.ok_or(RefreshPolicyError::MissingEndpoint)?;
    validate_refresh_endpoint_url(endpoint)?;
    Ok(())
}

pub fn claude_refresh_request(refresh_token: String) -> OAuthRefreshRequest {
    OAuthRefreshRequest {
        grant_type: "refresh_token".to_string(),
        refresh_token,
        client_id: CLAUDE_CLIENT_ID.to_string(),
        scope: Some(CLAUDE_SCOPES.join(" ")),
    }
}

pub fn codex_refresh_request(refresh_token: String) -> OAuthRefreshRequest {
    OAuthRefreshRequest {
        grant_type: "refresh_token".to_string(),
        refresh_token,
        client_id: CODEX_CLIENT_ID.to_string(),
        scope: None,
    }
}

pub async fn refresh_access_token(
    endpoint: &str,
    request: &OAuthRefreshRequest,
    http: &dyn RefreshHttpClient,
) -> Result<OAuthRefreshToken, ProviderError> {
    validate_refresh_endpoint(Some(endpoint)).map_err(|_| ProviderError::AuthError)?;
    let response = http
        .post_json(endpoint, request)
        .await
        .map_err(|error| match error {
            RefreshHttpError::Endpoint(_) | RefreshHttpError::InvalidHeader => {
                ProviderError::AuthError
            }
            RefreshHttpError::Network => ProviderError::Network,
        })?;

    match response.status {
        200 => parse_oauth_refresh_response(&response.body),
        status => Err(provider_error_for_refresh_failure(Some(status))),
    }
}

pub fn parse_oauth_refresh_response(input: &str) -> Result<OAuthRefreshToken, ProviderError> {
    #[derive(Debug, Deserialize)]
    struct RawRefreshResponse {
        #[serde(rename = "accessToken", alias = "access_token")]
        access_token: Option<String>,
        #[serde(rename = "refreshToken", alias = "refresh_token")]
        refresh_token: Option<String>,
        #[serde(rename = "expiresIn", alias = "expires_in")]
        expires_in: Option<u64>,
        scopes: Option<Vec<String>>,
        scope: Option<String>,
    }

    let raw: RawRefreshResponse =
        serde_json::from_str(input).map_err(|_| ProviderError::SchemaMismatch)?;
    let access_token = raw
        .access_token
        .filter(|value| !value.is_empty())
        .ok_or(ProviderError::SchemaMismatch)?;
    let scopes = raw.scopes.or_else(|| {
        raw.scope
            .map(|scope| scope.split_whitespace().map(str::to_string).collect())
    });

    Ok(OAuthRefreshToken {
        access_token,
        refresh_token: raw.refresh_token,
        expires_in: raw.expires_in,
        scopes,
    })
}

pub fn apply_memory_only_refresh_success(
    cache: &mut TokenRefreshCache,
    success: RefreshSuccess,
) -> UsageState {
    cache.store_memory_only(success.provider, success.access_token, success.refreshed_at);
    UsageState::Normal
}

pub fn provider_error_for_refresh_failure(status: Option<u16>) -> ProviderError {
    match status {
        Some(401 | 403) => ProviderError::AuthError,
        Some(429) => ProviderError::RateLimited,
        _ => ProviderError::Network,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct FixtureRefreshClient {
        response: RefreshResponse,
        requests: Arc<Mutex<Vec<(String, OAuthRefreshRequest)>>>,
    }

    impl FixtureRefreshClient {
        fn new(status: u16, body: impl Into<String>) -> Self {
            Self {
                response: RefreshResponse {
                    status,
                    body: body.into(),
                },
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn requests(&self) -> Vec<(String, OAuthRefreshRequest)> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl RefreshHttpClient for FixtureRefreshClient {
        async fn post_json(
            &self,
            url: &str,
            body: &OAuthRefreshRequest,
        ) -> Result<RefreshResponse, RefreshHttpError> {
            validate_refresh_endpoint_url(url)?;
            self.requests
                .lock()
                .unwrap()
                .push((url.to_string(), body.clone()));
            Ok(self.response.clone())
        }
    }

    #[test]
    fn undocumented_refresh_endpoint_is_blocked_before_network() {
        assert_eq!(
            validate_refresh_endpoint(None),
            Err(RefreshPolicyError::MissingEndpoint)
        );
        assert!(matches!(
            validate_refresh_endpoint(Some("https://auth.example.invalid/token")),
            Err(RefreshPolicyError::Endpoint(EndpointError::HostNotAllowed))
        ));
    }

    #[test]
    fn refresh_success_updates_memory_cache_without_warning_state() {
        let mut cache = TokenRefreshCache::default();
        let refreshed_at = Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap();

        let state = apply_memory_only_refresh_success(
            &mut cache,
            RefreshSuccess {
                provider: ProviderKind::Codex,
                access_token: "synthetic-access".to_string(),
                refreshed_at,
            },
        );

        assert_eq!(state, UsageState::Normal);
        assert!(cache.get_if_newer_than(ProviderKind::Codex, None).is_some());
    }

    #[test]
    fn refresh_failure_status_maps_to_provider_state_errors() {
        assert!(matches!(
            provider_error_for_refresh_failure(Some(401)),
            ProviderError::AuthError
        ));
        assert!(matches!(
            provider_error_for_refresh_failure(Some(403)),
            ProviderError::AuthError
        ));
        assert!(matches!(
            provider_error_for_refresh_failure(Some(429)),
            ProviderError::RateLimited
        ));
    }

    #[test]
    fn builds_source_backed_claude_refresh_request_without_logging_token() {
        let request = claude_refresh_request("synthetic-refresh".to_string());

        assert_eq!(request.grant_type, "refresh_token");
        assert_eq!(request.client_id, CLAUDE_CLIENT_ID);
        assert_eq!(
            request.scope.as_deref(),
            Some("user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload")
        );
    }

    #[test]
    fn builds_source_backed_codex_refresh_request_without_file_write() {
        let request = codex_refresh_request("synthetic-refresh".to_string());

        assert_eq!(request.grant_type, "refresh_token");
        assert_eq!(request.client_id, CODEX_CLIENT_ID);
        assert_eq!(request.scope, None);
    }

    #[test]
    fn parses_refresh_response_aliases() {
        let token = parse_oauth_refresh_response(
            r#"{"accessToken":"synthetic-access","refreshToken":"synthetic-refresh","expiresIn":3600,"scopes":["user:profile"]}"#,
        )
        .unwrap();

        assert_eq!(token.access_token, "synthetic-access");
        assert_eq!(token.refresh_token.as_deref(), Some("synthetic-refresh"));
        assert_eq!(token.expires_in, Some(3600));
        assert_eq!(token.scopes.unwrap(), vec!["user:profile"]);
    }

    #[tokio::test]
    async fn refresh_posts_only_to_allowlisted_endpoint_with_fixture_client() {
        let client = FixtureRefreshClient::new(200, r#"{"access_token":"synthetic-access"}"#);
        let request = claude_refresh_request("synthetic-refresh".to_string());

        let token = refresh_access_token(
            "https://platform.claude.com/v1/oauth/token",
            &request,
            &client,
        )
        .await
        .unwrap();

        assert_eq!(token.access_token, "synthetic-access");
        let requests = client.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].0, "https://platform.claude.com/v1/oauth/token");
    }

    #[test]
    fn refresh_debug_output_is_redacted() {
        let request = claude_refresh_request("synthetic-refresh".to_string());
        let token = OAuthRefreshToken {
            access_token: "synthetic-access".to_string(),
            refresh_token: Some("synthetic-refresh".to_string()),
            expires_in: Some(3600),
            scopes: None,
        };
        let success = RefreshSuccess {
            provider: ProviderKind::Claude,
            access_token: "synthetic-access".to_string(),
            refreshed_at: Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap(),
        };

        let debug = format!("{request:?} {token:?} {success:?}");
        assert!(!debug.contains("synthetic-access"));
        assert!(!debug.contains("synthetic-refresh"));
        assert!(debug.contains("[REDACTED]"));
    }
}
