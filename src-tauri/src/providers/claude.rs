use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::{
    config::EndpointConfig,
    http::{SafeHeader, UsageHttpClient},
    providers::{validate_usage_endpoint, ProviderError, UsageProvider},
    refresh::{
        apply_memory_only_refresh_success, claude_refresh_request, refresh_access_token,
        RefreshHttpClient, RefreshSuccess,
    },
    refresh_cache::TokenRefreshCache,
    snapshot::{ExtraUsage, ProviderKind, UsageSnapshot, UsageWindow},
    state::state_for_success,
    time::parse_rfc3339_utc,
    token_source::ClaudeCredentials,
};

pub struct ClaudeProvider;

#[async_trait]
impl UsageProvider for ClaudeProvider {
    fn provider(&self) -> ProviderKind {
        ProviderKind::Claude
    }

    async fn snapshot(&self, endpoints: &EndpointConfig) -> Result<UsageSnapshot, ProviderError> {
        validate_usage_endpoint(&endpoints.claude_usage)?;
        Err(ProviderError::Network)
    }
}

impl ClaudeProvider {
    pub async fn snapshot_with_http(
        &self,
        endpoints: &EndpointConfig,
        access_token: &str,
        http: &dyn UsageHttpClient,
    ) -> Result<UsageSnapshot, ProviderError> {
        validate_usage_endpoint(&endpoints.claude_usage)?;
        let headers = [
            SafeHeader {
                name: "anthropic-beta".to_string(),
                value: endpoints.claude_beta_header.clone(),
            },
            SafeHeader {
                name: "User-Agent".to_string(),
                value: "claude-code/2.1.138 token-dashboard/0.1.0".to_string(),
            },
        ];
        let response = http
            .get_with_bearer(&endpoints.claude_usage, access_token, &headers)
            .await?;
        match response.status {
            200 => parse_claude_usage(&response.body),
            401 | 403 => Err(ProviderError::AuthError),
            429 => Err(ProviderError::RateLimited),
            _ => Err(ProviderError::Network),
        }
    }

    pub async fn snapshot_with_refresh_http(
        &self,
        endpoints: &EndpointConfig,
        credentials: &ClaudeCredentials,
        usage_http: &dyn UsageHttpClient,
        refresh_http: &dyn RefreshHttpClient,
        cache: &mut TokenRefreshCache,
    ) -> Result<UsageSnapshot, ProviderError> {
        match self
            .snapshot_with_http(endpoints, &credentials.access_token, usage_http)
            .await
        {
            Err(ProviderError::AuthError) => {
                let refresh_token = credentials
                    .refresh_token
                    .as_ref()
                    .ok_or(ProviderError::AuthError)?;
                let token = refresh_access_token(
                    &endpoints.claude_refresh,
                    &claude_refresh_request(refresh_token.clone()),
                    refresh_http,
                )
                .await?;
                apply_memory_only_refresh_success(
                    cache,
                    RefreshSuccess {
                        provider: ProviderKind::Claude,
                        access_token: token.access_token.clone(),
                        refreshed_at: Utc::now(),
                    },
                );
                self.snapshot_with_http(endpoints, &token.access_token, usage_http)
                    .await
            }
            result => result,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    five_hour: Option<ClaudeUsageWindow>,
    seven_day: Option<ClaudeUsageWindow>,
    extra_usage: Option<ClaudeExtraUsage>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageWindow {
    utilization: f64,
    resets_at: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeExtraUsage {
    used_credits: Option<f64>,
    monthly_limit: Option<f64>,
    currency: Option<String>,
}

pub fn parse_claude_usage(input: &str) -> Result<UsageSnapshot, ProviderError> {
    let raw: ClaudeUsageResponse =
        serde_json::from_str(input).map_err(|_| ProviderError::SchemaMismatch)?;

    let primary = raw.five_hour.ok_or(ProviderError::SchemaMismatch)?;
    let secondary = raw.seven_day.ok_or(ProviderError::SchemaMismatch)?;

    let primary = UsageWindow {
        used_pct: primary.utilization,
        resets_at: parse_rfc3339_utc(&primary.resets_at)
            .map_err(|_| ProviderError::SchemaMismatch)?,
    };
    let secondary = UsageWindow {
        used_pct: secondary.utilization,
        resets_at: parse_rfc3339_utc(&secondary.resets_at)
            .map_err(|_| ProviderError::SchemaMismatch)?,
    };
    let max_used = primary.used_pct.max(secondary.used_pct);

    Ok(UsageSnapshot {
        provider: ProviderKind::Claude,
        state: state_for_success(max_used),
        primary: Some(primary),
        secondary: Some(secondary),
        extra: raw.extra_usage.map(|extra| ExtraUsage {
            used_credits: extra.used_credits,
            monthly_limit: extra.monthly_limit,
            currency: extra.currency,
        }),
        fetched_at: Utc::now(),
        is_stale: false,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::UsageState;
    use std::sync::{Arc, Mutex};

    #[test]
    fn converts_claude_fixture_to_usage_snapshot() {
        let snapshot =
            parse_claude_usage(include_str!("../../tests/fixtures/claude_usage.json")).unwrap();

        assert_eq!(snapshot.provider, ProviderKind::Claude);
        assert_eq!(snapshot.state, UsageState::Normal);
        assert_eq!(snapshot.primary.unwrap().used_pct, 34.0);
        assert_eq!(snapshot.secondary.unwrap().used_pct, 8.0);
        assert_eq!(snapshot.extra.unwrap().currency.as_deref(), Some("USD"));
    }

    #[test]
    fn ignores_unknown_null_fields() {
        let snapshot =
            parse_claude_usage(include_str!("../../tests/fixtures/claude_usage.json")).unwrap();
        assert!(snapshot.error.is_none());
    }

    #[test]
    fn missing_five_hour_is_schema_mismatch_for_stale_degradation() {
        let result = parse_claude_usage(
            r#"{"seven_day":{"utilization":8.0,"resets_at":"2026-06-16T11:00:00.531748+00:00"}}"#,
        );
        assert!(matches!(result, Err(ProviderError::SchemaMismatch)));
    }

    #[tokio::test]
    async fn provider_maps_http_status_without_real_api() {
        use crate::http::testsupport::FixtureHttpClient;

        let provider = ClaudeProvider;
        let endpoints = EndpointConfig::default();
        let auth = FixtureHttpClient::new(403, "{}");
        let rate = FixtureHttpClient::new(429, "{}");

        assert!(matches!(
            provider
                .snapshot_with_http(&endpoints, "synthetic-access", &auth)
                .await,
            Err(ProviderError::AuthError)
        ));
        assert!(matches!(
            provider
                .snapshot_with_http(&endpoints, "synthetic-access", &rate)
                .await,
            Err(ProviderError::RateLimited)
        ));
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
            headers: &[SafeHeader],
        ) -> Result<crate::http::UsageResponse, crate::http::UsageHttpError> {
            crate::config::validate_endpoint_url(url)?;
            assert!(!bearer_token.is_empty());
            assert!(!headers.is_empty());
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
            assert_eq!(body.grant_type, "refresh_token");
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
                    body: include_str!("../../tests/fixtures/claude_usage.json").to_string(),
                },
            ])),
        };
        let refresh = FixtureRefreshClient {
            status: 200,
            body: r#"{"accessToken":"synthetic-refreshed"}"#,
        };
        let credentials = ClaudeCredentials {
            access_token: "synthetic-expired".to_string(),
            refresh_token: Some("synthetic-refresh".to_string()),
            expires_at: None,
        };
        let mut cache = TokenRefreshCache::default();

        let snapshot = ClaudeProvider
            .snapshot_with_refresh_http(
                &EndpointConfig::default(),
                &credentials,
                &usage,
                &refresh,
                &mut cache,
            )
            .await
            .unwrap();

        assert_eq!(snapshot.provider, ProviderKind::Claude);
        assert!(cache
            .get_if_newer_than(ProviderKind::Claude, None)
            .is_some());
    }
}
