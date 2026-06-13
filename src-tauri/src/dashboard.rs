use crate::{
    config::EndpointConfig,
    http::{ReqwestUsageHttpClient, UsageHttpClient},
    providers::{ClaudeProvider, CodexProvider, ProviderError},
    refresh::{RefreshHttpClient, ReqwestRefreshHttpClient},
    refresh_cache::TokenRefreshCache,
    runtime::ProviderRuntimeState,
    snapshot::{ProviderKind, UsageSnapshot},
    token_source::{
        default_codex_auth_path, read_claude_credentials_default, read_codex_credentials_from_path,
        ClaudeCredentials, CodexCredentials, TokenSourceError,
    },
};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrontendWindow {
    pub used_pct: f64,
    pub resets_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrontendSnapshot {
    pub provider: ProviderKind,
    pub state: crate::snapshot::UsageState,
    pub primary: Option<FrontendWindow>,
    pub secondary: Option<FrontendWindow>,
    pub fetched_at: DateTime<Utc>,
    pub is_stale: bool,
}

impl From<UsageSnapshot> for FrontendSnapshot {
    fn from(snapshot: UsageSnapshot) -> Self {
        Self {
            provider: snapshot.provider,
            state: snapshot.state,
            primary: snapshot.primary.map(|window| FrontendWindow {
                used_pct: window.used_pct,
                resets_at: window.resets_at,
            }),
            secondary: snapshot.secondary.map(|window| FrontendWindow {
                used_pct: window.used_pct,
                resets_at: window.resets_at,
            }),
            fetched_at: snapshot.fetched_at,
            is_stale: snapshot.is_stale,
        }
    }
}

pub trait CredentialSource: Send + Sync {
    fn claude_credentials(&self) -> Result<ClaudeCredentials, TokenSourceError>;
    fn codex_credentials(&self) -> Result<CodexCredentials, TokenSourceError>;
}

#[derive(Debug, Default, Clone)]
pub struct DefaultCredentialSource;

impl CredentialSource for DefaultCredentialSource {
    fn claude_credentials(&self) -> Result<ClaudeCredentials, TokenSourceError> {
        read_claude_credentials_default().map(|(credentials, _warning)| credentials)
    }

    fn codex_credentials(&self) -> Result<CodexCredentials, TokenSourceError> {
        read_codex_credentials_from_path(&default_codex_auth_path())
            .map(|(credentials, _warning)| credentials)
    }
}

pub struct DashboardRuntime<U, R, C> {
    endpoints: EndpointConfig,
    usage_http: U,
    refresh_http: R,
    credentials: C,
    refresh_cache: TokenRefreshCache,
    claude_state: ProviderRuntimeState,
    codex_state: ProviderRuntimeState,
}

impl Default
    for DashboardRuntime<ReqwestUsageHttpClient, ReqwestRefreshHttpClient, DefaultCredentialSource>
{
    fn default() -> Self {
        Self::new(
            EndpointConfig::default(),
            ReqwestUsageHttpClient::default(),
            ReqwestRefreshHttpClient::default(),
            DefaultCredentialSource,
        )
    }
}

impl<U, R, C> DashboardRuntime<U, R, C>
where
    U: UsageHttpClient,
    R: RefreshHttpClient,
    C: CredentialSource,
{
    pub fn new(endpoints: EndpointConfig, usage_http: U, refresh_http: R, credentials: C) -> Self {
        Self {
            endpoints,
            usage_http,
            refresh_http,
            credentials,
            refresh_cache: TokenRefreshCache::default(),
            claude_state: ProviderRuntimeState::new(ProviderKind::Claude),
            codex_state: ProviderRuntimeState::new(ProviderKind::Codex),
        }
    }

    pub async fn snapshots(&mut self) -> Vec<UsageSnapshot> {
        vec![self.claude_snapshot().await, self.codex_snapshot().await]
    }

    pub async fn frontend_snapshots(&mut self) -> Vec<FrontendSnapshot> {
        self.snapshots().await.into_iter().map(Into::into).collect()
    }

    async fn claude_snapshot(&mut self) -> UsageSnapshot {
        let result = match self.credentials.claude_credentials() {
            Ok(credentials) => {
                ClaudeProvider
                    .snapshot_with_refresh_http(
                        &self.endpoints,
                        &credentials,
                        &self.usage_http,
                        &self.refresh_http,
                        &mut self.refresh_cache,
                    )
                    .await
            }
            Err(error) => Err(provider_error_for_token_source(error)),
        };

        match result {
            Ok(snapshot) => self.claude_state.apply_success(snapshot),
            Err(error) => self.claude_state.apply_error(error),
        }
    }

    async fn codex_snapshot(&mut self) -> UsageSnapshot {
        let result = match self.credentials.codex_credentials() {
            Ok(credentials) => {
                CodexProvider
                    .snapshot_with_refresh_http(
                        &self.endpoints,
                        &credentials,
                        &self.usage_http,
                        &self.refresh_http,
                        &mut self.refresh_cache,
                    )
                    .await
            }
            Err(error) => Err(provider_error_for_token_source(error)),
        };

        match result {
            Ok(snapshot) => self.codex_state.apply_success(snapshot),
            Err(error) => self.codex_state.apply_error(error),
        }
    }
}

fn provider_error_for_token_source(error: TokenSourceError) -> ProviderError {
    match error {
        TokenSourceError::Missing | TokenSourceError::UnsupportedAuthMode => {
            ProviderError::NotLoggedIn
        }
        TokenSourceError::InvalidSchema | TokenSourceError::ReadFailed => ProviderError::AuthError,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        http::{SafeHeader, UsageHttpError, UsageResponse},
        refresh::{OAuthRefreshRequest, RefreshHttpError, RefreshResponse},
        snapshot::UsageState,
    };
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct FixtureCredentials {
        claude: Result<ClaudeCredentials, TokenSourceError>,
        codex: Result<CodexCredentials, TokenSourceError>,
    }

    impl CredentialSource for FixtureCredentials {
        fn claude_credentials(&self) -> Result<ClaudeCredentials, TokenSourceError> {
            self.claude.clone()
        }

        fn codex_credentials(&self) -> Result<CodexCredentials, TokenSourceError> {
            self.codex.clone()
        }
    }

    #[derive(Clone)]
    struct SequenceUsageClient {
        responses: Arc<Mutex<Vec<UsageResponse>>>,
        requests: Arc<Mutex<Vec<String>>>,
    }

    impl SequenceUsageClient {
        fn new(responses: Vec<UsageResponse>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn request_count(&self) -> usize {
            self.requests.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl UsageHttpClient for SequenceUsageClient {
        async fn get_with_bearer(
            &self,
            url: &str,
            bearer_token: &str,
            _headers: &[SafeHeader],
        ) -> Result<UsageResponse, UsageHttpError> {
            crate::config::validate_endpoint_url(url)?;
            assert!(!bearer_token.is_empty());
            self.requests.lock().unwrap().push(url.to_string());
            Ok(self.responses.lock().unwrap().remove(0))
        }
    }

    #[derive(Clone)]
    struct FixtureRefreshClient;

    #[async_trait]
    impl RefreshHttpClient for FixtureRefreshClient {
        async fn post_json(
            &self,
            url: &str,
            _body: &OAuthRefreshRequest,
        ) -> Result<RefreshResponse, RefreshHttpError> {
            crate::config::validate_refresh_endpoint_url(url)?;
            Ok(RefreshResponse {
                status: 500,
                body: "{}".to_string(),
            })
        }
    }

    fn credentials() -> FixtureCredentials {
        FixtureCredentials {
            claude: Ok(ClaudeCredentials {
                access_token: "synthetic-claude".to_string(),
                refresh_token: Some("synthetic-refresh".to_string()),
                expires_at: None,
            }),
            codex: Ok(CodexCredentials {
                access_token: "synthetic-codex".to_string(),
                refresh_token: Some("synthetic-refresh".to_string()),
                account_id: Some("synthetic-account".to_string()),
                last_refresh: None,
            }),
        }
    }

    #[tokio::test]
    async fn returns_provider_snapshots_from_fixture_clients() {
        let usage = SequenceUsageClient::new(vec![
            UsageResponse {
                status: 200,
                body: include_str!("../tests/fixtures/claude_usage.json").to_string(),
            },
            UsageResponse {
                status: 200,
                body: include_str!("../tests/fixtures/codex_raw_usage.json").to_string(),
            },
        ]);
        let mut runtime = DashboardRuntime::new(
            EndpointConfig::default(),
            usage.clone(),
            FixtureRefreshClient,
            credentials(),
        );

        let snapshots = runtime.snapshots().await;

        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].provider, ProviderKind::Claude);
        assert_eq!(snapshots[0].state, UsageState::Normal);
        assert_eq!(snapshots[1].provider, ProviderKind::Codex);
        assert_eq!(snapshots[1].state, UsageState::Normal);
        assert_eq!(usage.request_count(), 2);
    }

    #[tokio::test]
    async fn missing_token_sources_degrade_without_http_requests() {
        let usage = SequenceUsageClient::new(Vec::new());
        let mut runtime = DashboardRuntime::new(
            EndpointConfig::default(),
            usage.clone(),
            FixtureRefreshClient,
            FixtureCredentials {
                claude: Err(TokenSourceError::Missing),
                codex: Err(TokenSourceError::UnsupportedAuthMode),
            },
        );

        let snapshots = runtime.snapshots().await;

        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].state, UsageState::NotLoggedIn);
        assert_eq!(snapshots[1].state, UsageState::NotLoggedIn);
        assert_eq!(usage.request_count(), 0);
    }

    #[tokio::test]
    async fn frontend_snapshots_do_not_expose_error_or_extra_fields() {
        let usage = SequenceUsageClient::new(Vec::new());
        let mut runtime = DashboardRuntime::new(
            EndpointConfig::default(),
            usage,
            FixtureRefreshClient,
            FixtureCredentials {
                claude: Err(TokenSourceError::Missing),
                codex: Err(TokenSourceError::Missing),
            },
        );

        let serialized = serde_json::to_string(&runtime.frontend_snapshots().await).unwrap();

        assert!(serialized.contains("\"provider\":\"claude\""));
        assert!(serialized.contains("\"state\":\"NOT_LOGGED_IN\""));
        assert!(!serialized.contains("access_token"));
        assert!(!serialized.contains("refresh_token"));
        assert!(!serialized.contains("id_token"));
        assert!(!serialized.contains("Authorization"));
        assert!(!serialized.contains("Bearer "));
        assert!(!serialized.contains("error"));
        assert!(!serialized.contains("extra"));
    }
}
