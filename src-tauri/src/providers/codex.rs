use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::{
    config::{join_codex_usage_url, EndpointConfig},
    http::UsageHttpClient,
    providers::{ProviderError, UsageProvider},
    refresh::{
        apply_memory_only_refresh_success, codex_refresh_request, refresh_access_token,
        RefreshHttpClient, RefreshSuccess,
    },
    refresh_cache::TokenRefreshCache,
    snapshot::{ProviderKind, UsageSnapshot, UsageWindow},
    state::state_for_success,
    time::{epoch_seconds_to_utc, parse_rfc3339_utc},
    token_source::CodexCredentials,
};

const FIVE_HOURS_SECONDS: u64 = 18_000;
const WEEK_SECONDS: u64 = 604_800;

pub struct CodexProvider;

#[async_trait]
impl UsageProvider for CodexProvider {
    fn provider(&self) -> ProviderKind {
        ProviderKind::Codex
    }

    async fn snapshot(&self, endpoints: &EndpointConfig) -> Result<UsageSnapshot, ProviderError> {
        let Some(_url) = join_codex_usage_url(endpoints)? else {
            return Err(ProviderError::SchemaMismatch);
        };
        Err(ProviderError::Network)
    }
}

impl CodexProvider {
    pub async fn snapshot_with_http(
        &self,
        endpoints: &EndpointConfig,
        access_token: &str,
        account_id: Option<&str>,
        http: &dyn UsageHttpClient,
    ) -> Result<UsageSnapshot, ProviderError> {
        let Some(url) = join_codex_usage_url(endpoints)? else {
            return Err(ProviderError::SchemaMismatch);
        };

        let headers = account_id
            .map(|account_id| {
                vec![crate::http::SafeHeader {
                    name: "ChatGPT-Account-Id".to_string(),
                    value: account_id.to_string(),
                }]
            })
            .unwrap_or_default();
        let response = http.get_with_bearer(&url, access_token, &headers).await?;
        match response.status {
            200 => parse_codex_raw_usage(&response.body),
            401 | 403 => Err(ProviderError::AuthError),
            429 => Err(ProviderError::RateLimited),
            _ => Err(ProviderError::Network),
        }
    }

    pub async fn snapshot_with_refresh_http(
        &self,
        endpoints: &EndpointConfig,
        credentials: &CodexCredentials,
        usage_http: &dyn UsageHttpClient,
        refresh_http: &dyn RefreshHttpClient,
        cache: &mut TokenRefreshCache,
    ) -> Result<UsageSnapshot, ProviderError> {
        let cached_token = cache
            .get_if_newer_than(ProviderKind::Codex, None)
            .map(|cached| cached.access_token.clone())
            .unwrap_or_else(|| credentials.access_token.clone());

        match self
            .snapshot_with_http(
                endpoints,
                &cached_token,
                credentials.account_id.as_deref(),
                usage_http,
            )
            .await
        {
            Err(ProviderError::AuthError) => {
                let refresh_token = credentials
                    .refresh_token
                    .as_ref()
                    .ok_or(ProviderError::AuthError)?;
                let token = refresh_access_token(
                    &endpoints.codex_refresh,
                    &codex_refresh_request(refresh_token.clone()),
                    refresh_http,
                )
                .await?;
                apply_memory_only_refresh_success(
                    cache,
                    RefreshSuccess {
                        provider: ProviderKind::Codex,
                        access_token: token.access_token.clone(),
                        refreshed_at: Utc::now(),
                    },
                );
                self.snapshot_with_http(
                    endpoints,
                    &token.access_token,
                    credentials.account_id.as_deref(),
                    usage_http,
                )
                .await
            }
            result => result,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CodexRawUsageResponse {
    rate_limit: Option<CodexRateLimit>,
}

#[derive(Debug, Deserialize)]
struct CodexRateLimit {
    primary_window: Option<CodexRawWindow>,
    secondary_window: Option<CodexRawWindow>,
}

#[derive(Debug, Deserialize)]
struct CodexUsageResponse {
    windows: CodexWindows,
}

#[derive(Debug, Deserialize)]
struct CodexWindows {
    primary: Option<CodexWindow>,
    secondary: Option<CodexWindow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexWindow {
    percent_used: Option<f64>,
    resets_at: Option<String>,
    raw: Option<CodexRawWindow>,
}

#[derive(Debug, Deserialize)]
struct CodexRawWindow {
    used_percent: Option<f64>,
    limit_window_seconds: u64,
    reset_at: Option<i64>,
}

pub fn parse_codex_usage(input: &str) -> Result<UsageSnapshot, ProviderError> {
    let mut raw_values: Vec<CodexUsageResponse> =
        serde_json::from_str(input).map_err(|_| ProviderError::SchemaMismatch)?;
    let raw = raw_values.pop().ok_or(ProviderError::SchemaMismatch)?;

    let windows = [raw.windows.primary, raw.windows.secondary];
    let mut primary = None;
    let mut secondary = None;

    for window in windows.into_iter().flatten() {
        let limit = window
            .raw
            .as_ref()
            .map(|raw| raw.limit_window_seconds)
            .ok_or(ProviderError::SchemaMismatch)?;
        let parsed = parse_codex_window(window)?;
        match limit {
            FIVE_HOURS_SECONDS => primary = Some(parsed),
            WEEK_SECONDS => secondary = Some(parsed),
            _ => {}
        }
    }

    let primary = primary.ok_or(ProviderError::SchemaMismatch)?;
    let secondary = secondary.ok_or(ProviderError::SchemaMismatch)?;
    let max_used = primary.used_pct.max(secondary.used_pct);

    Ok(UsageSnapshot {
        provider: ProviderKind::Codex,
        state: state_for_success(max_used),
        primary: Some(primary),
        secondary: Some(secondary),
        extra: None,
        fetched_at: Utc::now(),
        is_stale: false,
        error: None,
    })
}

pub fn parse_codex_raw_usage(input: &str) -> Result<UsageSnapshot, ProviderError> {
    let raw: CodexRawUsageResponse =
        serde_json::from_str(input).map_err(|_| ProviderError::SchemaMismatch)?;
    let rate_limit = raw.rate_limit.ok_or(ProviderError::SchemaMismatch)?;
    let windows = [rate_limit.primary_window, rate_limit.secondary_window];
    let mut primary = None;
    let mut secondary = None;

    for window in windows.into_iter().flatten() {
        let limit = window.limit_window_seconds;
        let parsed = parse_codex_raw_window(window)?;
        match limit {
            FIVE_HOURS_SECONDS => primary = Some(parsed),
            WEEK_SECONDS => secondary = Some(parsed),
            _ => {}
        }
    }

    let primary = primary.ok_or(ProviderError::SchemaMismatch)?;
    let secondary = secondary.ok_or(ProviderError::SchemaMismatch)?;
    let max_used = primary.used_pct.max(secondary.used_pct);

    Ok(UsageSnapshot {
        provider: ProviderKind::Codex,
        state: state_for_success(max_used),
        primary: Some(primary),
        secondary: Some(secondary),
        extra: None,
        fetched_at: Utc::now(),
        is_stale: false,
        error: None,
    })
}

fn parse_codex_window(window: CodexWindow) -> Result<UsageWindow, ProviderError> {
    let raw = window.raw.ok_or(ProviderError::SchemaMismatch)?;
    let used_pct = window
        .percent_used
        .or(raw.used_percent)
        .ok_or(ProviderError::SchemaMismatch)?;

    let resets_at = match (window.resets_at, raw.reset_at) {
        (Some(iso), _) => parse_rfc3339_utc(&iso).map_err(|_| ProviderError::SchemaMismatch)?,
        (None, Some(epoch)) => {
            epoch_seconds_to_utc(epoch).map_err(|_| ProviderError::SchemaMismatch)?
        }
        (None, None) => return Err(ProviderError::SchemaMismatch),
    };

    Ok(UsageWindow {
        used_pct,
        resets_at,
    })
}

fn parse_codex_raw_window(window: CodexRawWindow) -> Result<UsageWindow, ProviderError> {
    let used_pct = window.used_percent.ok_or(ProviderError::SchemaMismatch)?;
    let reset_at = window.reset_at.ok_or(ProviderError::SchemaMismatch)?;
    Ok(UsageWindow {
        used_pct,
        resets_at: epoch_seconds_to_utc(reset_at).map_err(|_| ProviderError::SchemaMismatch)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{ProviderKind, UsageState};
    use std::sync::{Arc, Mutex};

    #[test]
    fn converts_codex_fixture_to_usage_snapshot() {
        let snapshot =
            parse_codex_usage(include_str!("../../tests/fixtures/codex_usage.json")).unwrap();

        assert_eq!(snapshot.provider, ProviderKind::Codex);
        assert_eq!(snapshot.state, UsageState::Normal);
        assert_eq!(snapshot.primary.unwrap().used_pct, 45.0);
        assert_eq!(snapshot.secondary.unwrap().used_pct, 23.0);
    }

    #[test]
    fn converts_codex_raw_usage_fixture_to_usage_snapshot() {
        let snapshot =
            parse_codex_raw_usage(include_str!("../../tests/fixtures/codex_raw_usage.json"))
                .unwrap();

        assert_eq!(snapshot.provider, ProviderKind::Codex);
        assert_eq!(snapshot.primary.unwrap().used_pct, 45.0);
        assert_eq!(snapshot.secondary.unwrap().used_pct, 23.0);
    }

    #[test]
    fn falls_back_to_epoch_reset_at() {
        let snapshot = parse_codex_usage(include_str!(
            "../../tests/fixtures/codex_usage_epoch_only.json"
        ))
        .unwrap();
        assert_eq!(
            snapshot.primary.unwrap().resets_at.to_rfc3339(),
            "2026-06-10T09:07:44+00:00"
        );
    }

    #[test]
    fn identifies_windows_by_limit_seconds_not_label_or_position() {
        let snapshot = parse_codex_usage(include_str!(
            "../../tests/fixtures/codex_usage_swapped.json"
        ))
        .unwrap();
        assert_eq!(snapshot.primary.unwrap().used_pct, 45.0);
        assert_eq!(snapshot.secondary.unwrap().used_pct, 23.0);
    }

    #[tokio::test]
    async fn provider_uses_override_path_and_fixture_http_without_real_api() {
        use crate::http::testsupport::FixtureHttpClient;

        let provider = CodexProvider;
        let endpoints = EndpointConfig {
            codex_usage_path: Some("/backend-api/synthetic-usage".to_string()),
            ..EndpointConfig::default()
        };
        let http = FixtureHttpClient::new(
            200,
            include_str!("../../tests/fixtures/codex_raw_usage.json"),
        );

        let snapshot = provider
            .snapshot_with_http(
                &endpoints,
                "synthetic-access",
                Some("synthetic-account"),
                &http,
            )
            .await
            .unwrap();

        assert_eq!(snapshot.provider, ProviderKind::Codex);
        assert_eq!(http.requests().len(), 1);
        assert!(http.requests()[0].bearer_was_attached);
    }

    #[tokio::test]
    async fn provider_uses_verified_default_usage_path() {
        use crate::http::testsupport::FixtureHttpClient;

        let provider = CodexProvider;
        let endpoints = EndpointConfig::default();
        let http = FixtureHttpClient::new(
            200,
            include_str!("../../tests/fixtures/codex_raw_usage.json"),
        );

        let snapshot = provider
            .snapshot_with_http(&endpoints, "synthetic-access", None, &http)
            .await
            .unwrap();

        assert_eq!(snapshot.provider, ProviderKind::Codex);
        assert_eq!(http.requests().len(), 1);
        assert_eq!(
            http.requests()[0].url,
            "https://chatgpt.com/backend-api/wham/usage"
        );
    }

    #[tokio::test]
    async fn provider_rejects_empty_usage_path_override() {
        use crate::http::testsupport::FixtureHttpClient;

        let provider = CodexProvider;
        let endpoints = EndpointConfig {
            codex_usage_path: None,
            ..EndpointConfig::default()
        };
        let http = FixtureHttpClient::new(
            200,
            include_str!("../../tests/fixtures/codex_raw_usage.json"),
        );

        assert!(matches!(
            provider
                .snapshot_with_http(&endpoints, "synthetic-access", None, &http)
                .await,
            Err(ProviderError::SchemaMismatch)
        ));
        assert!(http.requests().is_empty());
    }

    #[derive(Clone)]
    struct SequenceUsageClient {
        responses: Arc<Mutex<Vec<crate::http::UsageResponse>>>,
    }

    #[async_trait::async_trait]
    impl UsageHttpClient for SequenceUsageClient {
        async fn get_with_bearer(
            &self,
            url: &str,
            bearer_token: &str,
            _headers: &[crate::http::SafeHeader],
        ) -> Result<crate::http::UsageResponse, crate::http::UsageHttpError> {
            crate::config::validate_endpoint_url(url)?;
            assert!(!bearer_token.is_empty());
            Ok(self.responses.lock().unwrap().remove(0))
        }
    }

    #[derive(Clone)]
    struct FixtureRefreshClient {
        status: u16,
        body: &'static str,
    }

    #[async_trait::async_trait]
    impl RefreshHttpClient for FixtureRefreshClient {
        async fn post_json(
            &self,
            url: &str,
            body: &crate::refresh::OAuthRefreshRequest,
        ) -> Result<crate::refresh::RefreshResponse, crate::refresh::RefreshHttpError> {
            crate::config::validate_refresh_endpoint_url(url)?;
            assert_eq!(body.client_id, "app_EMoamEEZ73f0CkXaXp7hrann");
            Ok(crate::refresh::RefreshResponse {
                status: self.status,
                body: self.body.to_string(),
            })
        }
    }

    #[tokio::test]
    async fn auth_error_refreshes_memory_only_and_retries_usage() {
        let usage = SequenceUsageClient {
            responses: Arc::new(Mutex::new(vec![
                crate::http::UsageResponse {
                    status: 401,
                    body: "{}".to_string(),
                },
                crate::http::UsageResponse {
                    status: 200,
                    body: include_str!("../../tests/fixtures/codex_raw_usage.json").to_string(),
                },
            ])),
        };
        let refresh = FixtureRefreshClient {
            status: 200,
            body: r#"{"access_token":"synthetic-refreshed"}"#,
        };
        let credentials = CodexCredentials {
            access_token: "synthetic-expired".to_string(),
            refresh_token: Some("synthetic-refresh".to_string()),
            account_id: Some("synthetic-account".to_string()),
            last_refresh: None,
        };
        let mut cache = TokenRefreshCache::default();

        let snapshot = CodexProvider
            .snapshot_with_refresh_http(
                &EndpointConfig::default(),
                &credentials,
                &usage,
                &refresh,
                &mut cache,
            )
            .await
            .unwrap();

        assert_eq!(snapshot.provider, ProviderKind::Codex);
        assert!(cache.get_if_newer_than(ProviderKind::Codex, None).is_some());
    }
}
