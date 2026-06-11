use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{
    config::{validate_refresh_endpoint_url, EndpointError},
    providers::ProviderError,
    refresh_cache::TokenRefreshCache,
    snapshot::{ProviderKind, UsageState},
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RefreshPolicyError {
    #[error("refresh endpoint is not documented")]
    MissingEndpoint,
    #[error("refresh endpoint is not allowed: {0}")]
    Endpoint(#[from] EndpointError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshSuccess {
    pub provider: ProviderKind,
    pub access_token: String,
    pub refreshed_at: DateTime<Utc>,
}

pub fn validate_refresh_endpoint(endpoint: Option<&str>) -> Result<(), RefreshPolicyError> {
    let endpoint = endpoint.ok_or(RefreshPolicyError::MissingEndpoint)?;
    validate_refresh_endpoint_url(endpoint)?;
    Ok(())
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
}
