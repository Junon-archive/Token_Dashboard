use token_dashboard::{
    config::{join_codex_usage_url, validate_endpoint_url, EndpointConfig, EndpointError},
    providers::{claude::parse_claude_usage, codex::parse_codex_usage, degraded, ProviderError},
    snapshot::{ProviderKind, UsageState},
};

#[test]
fn schema_mismatch_degrades_to_stale_without_panic() {
    let result = parse_claude_usage(r#"{"unexpected":true}"#);
    assert!(matches!(result, Err(ProviderError::SchemaMismatch)));

    let snapshot = degraded(ProviderKind::Claude, ProviderError::SchemaMismatch);
    assert_eq!(snapshot.state, UsageState::Stale);
    assert!(snapshot.is_stale);
}

#[test]
fn auth_and_rate_errors_map_to_logical_states() {
    assert_eq!(
        degraded(ProviderKind::Codex, ProviderError::NotLoggedIn).state,
        UsageState::NotLoggedIn
    );
    assert_eq!(
        degraded(ProviderKind::Codex, ProviderError::AuthError).state,
        UsageState::AuthError
    );
    assert_eq!(
        degraded(ProviderKind::Codex, ProviderError::RateLimited).state,
        UsageState::RateLimited
    );
}

#[test]
fn codex_usage_path_defaults_to_verified_wham_usage_path() {
    let config = EndpointConfig::default();
    assert_eq!(
        join_codex_usage_url(&config),
        Ok(Some(
            "https://chatgpt.com/backend-api/wham/usage".to_string()
        ))
    );
}

#[test]
fn endpoint_allowlist_blocks_unknown_hosts_before_tokens_attach() {
    assert_eq!(
        validate_endpoint_url("https://chatgpt.com/backend-api/usage"),
        Ok(())
    );
    assert_eq!(
        validate_endpoint_url("https://chatgpt.com.evil.invalid/backend-api/usage"),
        Err(EndpointError::HostNotAllowed)
    );
}

#[test]
fn fixtures_are_parseable_contract_examples() {
    let claude = parse_claude_usage(include_str!("fixtures/claude_usage.json")).unwrap();
    let codex = parse_codex_usage(include_str!("fixtures/codex_usage.json")).unwrap();

    assert_eq!(claude.provider, ProviderKind::Claude);
    assert_eq!(codex.provider, ProviderKind::Codex);
}
