use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TokenSourceError {
    #[error("token source is missing")]
    Missing,
    #[error("token source has unsupported auth mode")]
    UnsupportedAuthMode,
    #[error("token source schema is invalid")]
    InvalidSchema,
    #[error("token source could not be read")]
    ReadFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeCredentials {
    pub access_token: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub account_id: Option<String>,
    pub last_refresh: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePermissionWarning {
    pub path: PathBuf,
    pub mode: u32,
}

#[derive(Debug, Deserialize)]
struct ClaudeAuthJson {
    #[serde(rename = "claudeAiOauth")]
    oauth: Option<ClaudeOauthJson>,
}

#[derive(Debug, Deserialize)]
struct ClaudeOauthJson {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexAuthJson {
    auth_mode: Option<String>,
    tokens: Option<CodexTokensJson>,
    last_refresh: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexTokensJson {
    access_token: Option<String>,
    refresh_token: Option<String>,
    account_id: Option<String>,
}

pub fn parse_claude_credentials(input: &str) -> Result<ClaudeCredentials, TokenSourceError> {
    let parsed: ClaudeAuthJson =
        serde_json::from_str(input).map_err(|_| TokenSourceError::InvalidSchema)?;
    let oauth = parsed.oauth.ok_or(TokenSourceError::InvalidSchema)?;
    let access_token = oauth
        .access_token
        .filter(|value| !value.is_empty())
        .ok_or(TokenSourceError::InvalidSchema)?;

    Ok(ClaudeCredentials {
        access_token,
        expires_at: oauth.expires_at,
    })
}

pub fn parse_codex_credentials(input: &str) -> Result<CodexCredentials, TokenSourceError> {
    let parsed: CodexAuthJson =
        serde_json::from_str(input).map_err(|_| TokenSourceError::InvalidSchema)?;

    if parsed.auth_mode.as_deref() != Some("chatgpt") {
        return Err(TokenSourceError::UnsupportedAuthMode);
    }

    let tokens = parsed.tokens.ok_or(TokenSourceError::InvalidSchema)?;
    let access_token = tokens
        .access_token
        .filter(|value| !value.is_empty())
        .ok_or(TokenSourceError::InvalidSchema)?;

    Ok(CodexCredentials {
        access_token,
        refresh_token: tokens.refresh_token,
        account_id: tokens.account_id,
        last_refresh: parsed.last_refresh,
    })
}

pub fn read_file(path: PathBuf) -> Result<String, TokenSourceError> {
    fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            TokenSourceError::Missing
        } else {
            TokenSourceError::ReadFailed
        }
    })
}

pub fn default_codex_auth_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
        .join("auth.json")
}

#[cfg(target_os = "linux")]
pub fn default_claude_credentials_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join(".credentials.json")
}

pub fn read_codex_credentials_from_path(
    path: &Path,
) -> Result<(CodexCredentials, Option<FilePermissionWarning>), TokenSourceError> {
    let warning = permission_warning_if_broader_than_600(path.to_path_buf());
    let raw = read_file(path.to_path_buf())?;
    Ok((parse_codex_credentials(&raw)?, warning))
}

#[cfg(target_os = "linux")]
pub fn read_claude_credentials_default(
) -> Result<(ClaudeCredentials, Option<FilePermissionWarning>), TokenSourceError> {
    read_claude_credentials_from_path(&default_claude_credentials_path())
}

#[cfg(target_os = "linux")]
pub fn read_claude_credentials_from_path(
    path: &Path,
) -> Result<(ClaudeCredentials, Option<FilePermissionWarning>), TokenSourceError> {
    let warning = permission_warning_if_broader_than_600(path.to_path_buf());
    let raw = read_file(path.to_path_buf())?;
    Ok((parse_claude_credentials(&raw)?, warning))
}

#[cfg(target_os = "macos")]
pub fn read_claude_credentials_default(
) -> Result<(ClaudeCredentials, Option<FilePermissionWarning>), TokenSourceError> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .map_err(|_| TokenSourceError::ReadFailed)?;

    if !output.status.success() {
        return Err(TokenSourceError::Missing);
    }

    let raw = String::from_utf8(output.stdout).map_err(|_| TokenSourceError::InvalidSchema)?;
    Ok((parse_claude_credentials(&raw)?, None))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn read_claude_credentials_default(
) -> Result<(ClaudeCredentials, Option<FilePermissionWarning>), TokenSourceError> {
    Err(TokenSourceError::Missing)
}

#[cfg(unix)]
pub fn permission_warning_if_broader_than_600(path: PathBuf) -> Option<FilePermissionWarning> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(&path).ok()?;
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        Some(FilePermissionWarning { path, mode })
    } else {
        None
    }
}

#[cfg(not(unix))]
pub fn permission_warning_if_broader_than_600(_path: PathBuf) -> Option<FilePermissionWarning> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_claude_credentials_without_logging_tokens() {
        let parsed = parse_claude_credentials(
            r#"{"claudeAiOauth":{"accessToken":"synthetic-access","refreshToken":"synthetic-refresh","expiresAt":"2026-06-10T13:40:00Z"}}"#,
        )
        .unwrap();

        assert_eq!(parsed.access_token, "synthetic-access");
        assert_eq!(parsed.expires_at.as_deref(), Some("2026-06-10T13:40:00Z"));
    }

    #[test]
    fn parses_codex_chatgpt_credentials() {
        let parsed = parse_codex_credentials(
            r#"{"auth_mode":"chatgpt","last_refresh":"2026-06-10T09:11:45Z","tokens":{"access_token":"synthetic-access","refresh_token":"synthetic-refresh","id_token":"synthetic-id","account_id":"synthetic-account"}}"#,
        )
        .unwrap();

        assert_eq!(parsed.access_token, "synthetic-access");
        assert_eq!(parsed.refresh_token.as_deref(), Some("synthetic-refresh"));
        assert_eq!(parsed.account_id.as_deref(), Some("synthetic-account"));
    }

    #[test]
    fn codex_non_chatgpt_auth_mode_is_not_logged_in_condition() {
        let result = parse_codex_credentials(
            r#"{"auth_mode":"api-key","tokens":{"access_token":"synthetic-access"}}"#,
        );

        assert_eq!(result, Err(TokenSourceError::UnsupportedAuthMode));
    }

    #[test]
    fn reads_codex_credentials_from_explicit_synthetic_file() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        fs::write(
            temp.path(),
            r#"{"auth_mode":"chatgpt","tokens":{"access_token":"synthetic-access","refresh_token":"synthetic-refresh","account_id":"synthetic-account"}}"#,
        )
        .unwrap();

        let (credentials, _warning) = read_codex_credentials_from_path(temp.path()).unwrap();

        assert_eq!(credentials.access_token, "synthetic-access");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn reads_claude_credentials_from_explicit_synthetic_file() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        fs::write(
            temp.path(),
            r#"{"claudeAiOauth":{"accessToken":"synthetic-access","expiresAt":"2026-06-10T13:40:00Z"}}"#,
        )
        .unwrap();

        let (credentials, _warning) = read_claude_credentials_from_path(temp.path()).unwrap();

        assert_eq!(credentials.access_token, "synthetic-access");
    }

    #[cfg(unix)]
    #[test]
    fn warns_for_linux_token_file_permissions_broader_than_600() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::NamedTempFile::new().unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o644)).unwrap();

        let warning = permission_warning_if_broader_than_600(temp.path().to_path_buf()).unwrap();
        assert_eq!(warning.mode, 0o644);

        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o600)).unwrap();
        assert!(permission_warning_if_broader_than_600(temp.path().to_path_buf()).is_none());
    }
}
