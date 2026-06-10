use crate::snapshot::UsageState;

pub fn state_for_success(max_used_pct: f64) -> UsageState {
    if max_used_pct >= 95.0 {
        UsageState::Critical
    } else if max_used_pct >= 80.0 {
        UsageState::Warn
    } else {
        UsageState::Normal
    }
}

pub fn visual_class(state: UsageState, primary_used_pct: Option<f64>) -> Option<&'static str> {
    match state {
        UsageState::Normal => None,
        UsageState::Warn => Some("low"),
        UsageState::Critical if primary_used_pct.unwrap_or_default() >= 100.0 => Some("depleted"),
        UsageState::Critical => Some("critical"),
        UsageState::Stale | UsageState::RateLimited => Some("stale"),
        UsageState::NotLoggedIn => Some("notin"),
        UsageState::AuthError => Some("autherr"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_usage_thresholds() {
        assert_eq!(state_for_success(79.9), UsageState::Normal);
        assert_eq!(state_for_success(80.0), UsageState::Warn);
        assert_eq!(state_for_success(95.0), UsageState::Critical);
    }

    #[test]
    fn maps_logical_states_to_visual_classes() {
        assert_eq!(visual_class(UsageState::Normal, Some(1.0)), None);
        assert_eq!(visual_class(UsageState::Warn, Some(80.0)), Some("low"));
        assert_eq!(
            visual_class(UsageState::Critical, Some(95.0)),
            Some("critical")
        );
        assert_eq!(
            visual_class(UsageState::Critical, Some(100.0)),
            Some("depleted")
        );
        assert_eq!(visual_class(UsageState::Stale, Some(30.0)), Some("stale"));
        assert_eq!(
            visual_class(UsageState::RateLimited, Some(30.0)),
            Some("stale")
        );
        assert_eq!(visual_class(UsageState::NotLoggedIn, None), Some("notin"));
        assert_eq!(visual_class(UsageState::AuthError, None), Some("autherr"));
    }
}
