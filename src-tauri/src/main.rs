use chrono::{Duration, Utc};
use token_dashboard::{ProviderKind, UsageSnapshot, UsageState, UsageWindow};

#[tauri::command]
fn mock_claude_snapshot() -> UsageSnapshot {
    let now = Utc::now();
    UsageSnapshot {
        provider: ProviderKind::Claude,
        state: UsageState::Normal,
        primary: Some(UsageWindow {
            used_pct: 34.0,
            resets_at: now + Duration::hours(3) + Duration::minutes(17),
        }),
        secondary: Some(UsageWindow {
            used_pct: 8.0,
            resets_at: now + Duration::days(4),
        }),
        extra: None,
        fetched_at: now,
        is_stale: false,
        error: None,
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![mock_claude_snapshot])
        .run(tauri::generate_context!())
        .expect("failed to run Token Dashboard");
}
