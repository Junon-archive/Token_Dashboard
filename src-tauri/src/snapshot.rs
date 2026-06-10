use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Claude,
    Codex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UsageState {
    Normal,
    Warn,
    Critical,
    Stale,
    NotLoggedIn,
    AuthError,
    RateLimited,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageWindow {
    pub used_pct: f64,
    pub resets_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraUsage {
    pub used_credits: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub provider: ProviderKind,
    pub state: UsageState,
    pub primary: Option<UsageWindow>,
    pub secondary: Option<UsageWindow>,
    pub extra: Option<ExtraUsage>,
    pub fetched_at: DateTime<Utc>,
    pub is_stale: bool,
    pub error: Option<String>,
}

impl UsageSnapshot {
    pub fn degraded(provider: ProviderKind, state: UsageState, error: impl Into<String>) -> Self {
        Self {
            provider,
            state,
            primary: None,
            secondary: None,
            extra: None,
            fetched_at: Utc::now(),
            is_stale: matches!(state, UsageState::Stale),
            error: Some(error.into()),
        }
    }

    pub fn stale_from_last_good(last_good: &Self, error: impl Into<String>) -> Self {
        let mut snapshot = last_good.clone();
        snapshot.state = UsageState::Stale;
        snapshot.is_stale = true;
        snapshot.error = Some(error.into());
        snapshot
    }
}
