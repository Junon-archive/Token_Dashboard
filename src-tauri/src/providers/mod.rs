use async_trait::async_trait;
use thiserror::Error;

use crate::{
    config::{validate_endpoint_url, EndpointConfig, EndpointError},
    snapshot::{ProviderKind, UsageSnapshot, UsageState},
};

pub mod claude;
pub mod codex;

pub use claude::ClaudeProvider;
pub use codex::CodexProvider;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("not logged in")]
    NotLoggedIn,
    #[error("auth error")]
    AuthError,
    #[error("rate limited")]
    RateLimited,
    #[error("schema mismatch")]
    SchemaMismatch,
    #[error("endpoint rejected: {0}")]
    EndpointRejected(#[from] EndpointError),
    #[error("network request failed")]
    Network,
}

#[async_trait]
pub trait UsageProvider {
    fn provider(&self) -> ProviderKind;
    async fn snapshot(&self, endpoints: &EndpointConfig) -> Result<UsageSnapshot, ProviderError>;
}

pub fn degraded(provider: ProviderKind, error: ProviderError) -> UsageSnapshot {
    let state = match error {
        ProviderError::NotLoggedIn => UsageState::NotLoggedIn,
        ProviderError::AuthError => UsageState::AuthError,
        ProviderError::RateLimited => UsageState::RateLimited,
        ProviderError::SchemaMismatch
        | ProviderError::EndpointRejected(_)
        | ProviderError::Network => UsageState::Stale,
    };

    UsageSnapshot::degraded(provider, state, error.to_string())
}

pub(crate) fn validate_usage_endpoint(url: &str) -> Result<(), ProviderError> {
    Ok(validate_endpoint_url(url)?)
}
