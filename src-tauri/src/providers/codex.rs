use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::{
    config::{join_codex_usage_url, EndpointConfig},
    providers::{ProviderError, UsageProvider},
    snapshot::{ProviderKind, UsageSnapshot, UsageWindow},
    state::state_for_success,
    time::{epoch_seconds_to_utc, parse_rfc3339_utc},
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{ProviderKind, UsageState};

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
}
