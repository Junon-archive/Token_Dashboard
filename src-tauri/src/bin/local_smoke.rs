use std::{env, process::ExitCode, time::Duration};

use serde::Serialize;
use token_dashboard::{
    config::EndpointConfig,
    http::ReqwestUsageHttpClient,
    providers::{degraded, ClaudeProvider, CodexProvider, ProviderError},
    snapshot::{ProviderKind, UsageSnapshot},
    token_source::{
        default_codex_auth_path, read_claude_credentials_default, read_codex_credentials_from_path,
        FilePermissionWarning, TokenSourceError,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmokeProvider {
    Claude,
    Codex,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmokeArgs {
    provider: SmokeProvider,
    polls: u32,
    interval_sec: u64,
}

#[derive(Debug, Serialize)]
struct SmokeOutput {
    provider: ProviderKind,
    snapshot: UsageSnapshot,
    permission_warning: Option<PermissionWarningOutput>,
}

#[derive(Debug, Serialize)]
struct PermissionWarningOutput {
    present: bool,
    mode_octal: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

async fn run() -> Result<(), String> {
    guard_local_only()?;
    let args = parse_args(env::args().skip(1))?;
    let endpoints = EndpointConfig::default();
    let http = ReqwestUsageHttpClient::default();

    for poll_index in 0..args.polls {
        let outputs = collect_once(args.provider, &endpoints, &http).await;
        println!(
            "{}",
            serde_json::to_string_pretty(&outputs).map_err(|_| "failed to serialize output")?
        );

        if poll_index + 1 < args.polls {
            tokio::time::sleep(Duration::from_secs(args.interval_sec)).await;
        }
    }

    Ok(())
}

fn guard_local_only() -> Result<(), String> {
    if env::var("CI").as_deref() == Ok("true") {
        return Err("Refusing to run real API smoke test in CI.".to_string());
    }
    if env::var("TOKEN_DASHBOARD_ALLOW_REAL_API").as_deref() != Ok("1") {
        return Err(
            "Refusing to run real API smoke test. Set TOKEN_DASHBOARD_ALLOW_REAL_API=1 locally."
                .to_string(),
        );
    }
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<SmokeArgs, String> {
    let mut provider = SmokeProvider::All;
    let mut polls = 1;
    let mut interval_sec = 180;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--provider" => {
                provider = parse_provider(
                    &args
                        .next()
                        .ok_or_else(|| "--provider requires a value".to_string())?,
                )?;
            }
            "--polls" => {
                polls = args
                    .next()
                    .ok_or_else(|| "--polls requires a value".to_string())?
                    .parse()
                    .map_err(|_| "--polls must be a positive integer".to_string())?;
                if polls == 0 {
                    return Err("--polls must be greater than zero".to_string());
                }
            }
            "--interval-sec" => {
                interval_sec = args
                    .next()
                    .ok_or_else(|| "--interval-sec requires a value".to_string())?
                    .parse()
                    .map_err(|_| "--interval-sec must be a positive integer".to_string())?;
            }
            "--help" | "-h" => {
                return Err(
                    "Usage: local-smoke [--provider claude|codex|all] [--polls N] [--interval-sec SECONDS]"
                        .to_string(),
                );
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(SmokeArgs {
        provider,
        polls,
        interval_sec,
    })
}

fn parse_provider(value: &str) -> Result<SmokeProvider, String> {
    match value {
        "claude" => Ok(SmokeProvider::Claude),
        "codex" => Ok(SmokeProvider::Codex),
        "all" => Ok(SmokeProvider::All),
        _ => Err("--provider must be one of: claude, codex, all".to_string()),
    }
}

async fn collect_once(
    provider: SmokeProvider,
    endpoints: &EndpointConfig,
    http: &ReqwestUsageHttpClient,
) -> Vec<SmokeOutput> {
    let mut outputs = Vec::new();
    if matches!(provider, SmokeProvider::Claude | SmokeProvider::All) {
        outputs.push(collect_claude(endpoints, http).await);
    }
    if matches!(provider, SmokeProvider::Codex | SmokeProvider::All) {
        outputs.push(collect_codex(endpoints, http).await);
    }
    outputs
}

async fn collect_claude(endpoints: &EndpointConfig, http: &ReqwestUsageHttpClient) -> SmokeOutput {
    match read_claude_credentials_default() {
        Ok((credentials, warning)) => {
            let snapshot = ClaudeProvider
                .snapshot_with_http(endpoints, &credentials.access_token, http)
                .await
                .unwrap_or_else(|error| degraded(ProviderKind::Claude, error));
            SmokeOutput {
                provider: ProviderKind::Claude,
                snapshot,
                permission_warning: warning.map(permission_warning_output),
            }
        }
        Err(error) => SmokeOutput {
            provider: ProviderKind::Claude,
            snapshot: degraded(ProviderKind::Claude, token_error_to_provider_error(error)),
            permission_warning: None,
        },
    }
}

async fn collect_codex(endpoints: &EndpointConfig, http: &ReqwestUsageHttpClient) -> SmokeOutput {
    match read_codex_credentials_from_path(&default_codex_auth_path()) {
        Ok((credentials, warning)) => {
            let snapshot = CodexProvider
                .snapshot_with_http(
                    endpoints,
                    &credentials.access_token,
                    credentials.account_id.as_deref(),
                    http,
                )
                .await
                .unwrap_or_else(|error| degraded(ProviderKind::Codex, error));
            SmokeOutput {
                provider: ProviderKind::Codex,
                snapshot,
                permission_warning: warning.map(permission_warning_output),
            }
        }
        Err(error) => SmokeOutput {
            provider: ProviderKind::Codex,
            snapshot: degraded(ProviderKind::Codex, token_error_to_provider_error(error)),
            permission_warning: None,
        },
    }
}

fn token_error_to_provider_error(error: TokenSourceError) -> ProviderError {
    match error {
        TokenSourceError::Missing | TokenSourceError::UnsupportedAuthMode => {
            ProviderError::NotLoggedIn
        }
        TokenSourceError::InvalidSchema | TokenSourceError::ReadFailed => ProviderError::AuthError,
    }
}

fn permission_warning_output(warning: FilePermissionWarning) -> PermissionWarningOutput {
    PermissionWarningOutput {
        present: true,
        mode_octal: format!("{:03o}", warning.mode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_args() {
        let args = parse_args(Vec::<String>::new()).unwrap();
        assert_eq!(args.provider, SmokeProvider::All);
        assert_eq!(args.polls, 1);
        assert_eq!(args.interval_sec, 180);
    }

    #[test]
    fn parses_provider_and_poll_args() {
        let args = parse_args([
            "--provider".to_string(),
            "codex".to_string(),
            "--polls".to_string(),
            "20".to_string(),
            "--interval-sec".to_string(),
            "180".to_string(),
        ])
        .unwrap();
        assert_eq!(args.provider, SmokeProvider::Codex);
        assert_eq!(args.polls, 20);
        assert_eq!(args.interval_sec, 180);
    }

    #[test]
    fn rejects_zero_polls() {
        assert!(parse_args(["--polls".to_string(), "0".to_string()]).is_err());
    }
}
