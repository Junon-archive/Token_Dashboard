use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::{
    config::EndpointConfig,
    providers::{validate_usage_endpoint, ProviderError, UsageProvider},
    snapshot::{ExtraUsage, ProviderKind, UsageSnapshot, UsageWindow},
    state::state_for_success,
    time::parse_rfc3339_utc,
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
}
