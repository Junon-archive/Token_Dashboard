use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::snapshot::ProviderKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedToken {
    pub access_token: String,
    pub refreshed_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct TokenRefreshCache {
    tokens: HashMap<ProviderKind, CachedToken>,
}

impl TokenRefreshCache {
    pub fn get_if_newer_than(
        &self,
        provider: ProviderKind,
        file_last_refresh: Option<DateTime<Utc>>,
    ) -> Option<&CachedToken> {
        let token = self.tokens.get(&provider)?;
        match file_last_refresh {
            Some(file_time) if file_time >= token.refreshed_at => None,
            _ => Some(token),
        }
    }

    pub fn store_memory_only(
        &mut self,
        provider: ProviderKind,
        access_token: String,
        refreshed_at: DateTime<Utc>,
    ) {
        self.tokens.insert(
            provider,
            CachedToken {
                access_token,
                refreshed_at,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn refreshed_token_is_memory_only_and_file_newness_wins() {
        let mut cache = TokenRefreshCache::default();
        let cached_at = Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap();
        cache.store_memory_only(
            ProviderKind::Codex,
            "synthetic-access".to_string(),
            cached_at,
        );

        assert!(cache
            .get_if_newer_than(
                ProviderKind::Codex,
                Some(cached_at - chrono::Duration::seconds(1))
            )
            .is_some());
        assert!(cache
            .get_if_newer_than(ProviderKind::Codex, Some(cached_at))
            .is_none());
        assert!(cache
            .get_if_newer_than(
                ProviderKind::Codex,
                Some(cached_at + chrono::Duration::seconds(1))
            )
            .is_none());
    }
}
