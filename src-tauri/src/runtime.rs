use chrono::{DateTime, Duration, Utc};

use crate::{
    providers::ProviderError,
    snapshot::{ProviderKind, UsageSnapshot, UsageState},
};

#[derive(Debug, Clone)]
pub struct ProviderRuntimeState {
    provider: ProviderKind,
    last_good: Option<UsageSnapshot>,
    last_success_at: Option<DateTime<Utc>>,
}

impl ProviderRuntimeState {
    pub fn new(provider: ProviderKind) -> Self {
        Self {
            provider,
            last_good: None,
            last_success_at: None,
        }
    }

    pub fn apply_success(&mut self, snapshot: UsageSnapshot) -> UsageSnapshot {
        self.last_success_at = Some(snapshot.fetched_at);
        self.last_good = Some(snapshot.clone());
        snapshot
    }

    pub fn apply_error(&self, error: ProviderError) -> UsageSnapshot {
        match error {
            ProviderError::SchemaMismatch | ProviderError::Network => self
                .last_good
                .as_ref()
                .map(|snapshot| UsageSnapshot::stale_from_last_good(snapshot, error.to_string()))
                .unwrap_or_else(|| {
                    UsageSnapshot::degraded(self.provider, UsageState::Stale, error.to_string())
                }),
            ProviderError::RateLimited => self
                .last_good
                .as_ref()
                .map(|snapshot| {
                    let mut degraded =
                        UsageSnapshot::stale_from_last_good(snapshot, error.to_string());
                    degraded.state = UsageState::RateLimited;
                    degraded
                })
                .unwrap_or_else(|| {
                    UsageSnapshot::degraded(
                        self.provider,
                        UsageState::RateLimited,
                        error.to_string(),
                    )
                }),
            ProviderError::NotLoggedIn => {
                UsageSnapshot::degraded(self.provider, UsageState::NotLoggedIn, error.to_string())
            }
            ProviderError::AuthError => {
                UsageSnapshot::degraded(self.provider, UsageState::AuthError, error.to_string())
            }
            ProviderError::EndpointRejected(_) => {
                UsageSnapshot::degraded(self.provider, UsageState::Stale, error.to_string())
            }
        }
    }

    pub fn stale_if_older_than(
        &self,
        now: DateTime<Utc>,
        max_age: Duration,
    ) -> Option<UsageSnapshot> {
        let last_success_at = self.last_success_at?;
        if now - last_success_at > max_age {
            self.last_good
                .as_ref()
                .map(|snapshot| UsageSnapshot::stale_from_last_good(snapshot, "snapshot is stale"))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{UsageState, UsageWindow};
    use chrono::TimeZone;

    fn snapshot(provider: ProviderKind, fetched_at: DateTime<Utc>) -> UsageSnapshot {
        UsageSnapshot {
            provider,
            state: UsageState::Normal,
            primary: Some(UsageWindow {
                used_pct: 42.0,
                resets_at: fetched_at + Duration::hours(1),
            }),
            secondary: None,
            extra: None,
            fetched_at,
            is_stale: false,
            error: None,
        }
    }

    #[test]
    fn schema_mismatch_preserves_last_good_values_as_stale() {
        let now = Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap();
        let mut runtime = ProviderRuntimeState::new(ProviderKind::Claude);
        runtime.apply_success(snapshot(ProviderKind::Claude, now));

        let degraded = runtime.apply_error(ProviderError::SchemaMismatch);

        assert_eq!(degraded.state, UsageState::Stale);
        assert_eq!(degraded.primary.unwrap().used_pct, 42.0);
    }

    #[test]
    fn rate_limited_preserves_last_good_values() {
        let now = Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap();
        let mut runtime = ProviderRuntimeState::new(ProviderKind::Codex);
        runtime.apply_success(snapshot(ProviderKind::Codex, now));

        let degraded = runtime.apply_error(ProviderError::RateLimited);

        assert_eq!(degraded.state, UsageState::RateLimited);
        assert_eq!(degraded.primary.unwrap().used_pct, 42.0);
    }

    #[test]
    fn last_good_becomes_stale_after_ten_minutes() {
        let now = Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap();
        let mut runtime = ProviderRuntimeState::new(ProviderKind::Claude);
        runtime.apply_success(snapshot(ProviderKind::Claude, now));

        let stale = runtime
            .stale_if_older_than(now + Duration::minutes(11), Duration::minutes(10))
            .unwrap();

        assert_eq!(stale.state, UsageState::Stale);
        assert_eq!(stale.primary.unwrap().used_pct, 42.0);
    }
}
