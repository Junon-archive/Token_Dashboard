use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::{
    image::Image, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Position, Size,
    WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use token_dashboard::{
    autostart::set_autostart,
    config::{load_or_create_config, write_config, AppConfig, WidgetConfig, WindowPosition},
    dashboard::{DashboardRuntime, FrontendSnapshot},
    snapshot::ProviderKind,
};
use tokio::sync::Mutex;

type AppDashboardRuntime = DashboardRuntime<
    token_dashboard::http::ReqwestUsageHttpClient,
    token_dashboard::refresh::ReqwestRefreshHttpClient,
    token_dashboard::dashboard::DefaultCredentialSource,
>;

fn thumbnail_icon() -> Result<Image<'static>, String> {
    Image::from_bytes(include_bytes!("../../assets/Thumbnail.png"))
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn usage_snapshots(
    runtime: tauri::State<'_, Mutex<AppDashboardRuntime>>,
) -> Result<Vec<FrontendSnapshot>, String> {
    Ok(runtime.lock().await.frontend_snapshots().await)
}

#[tauri::command]
async fn usage_snapshot(
    provider: String,
    runtime: tauri::State<'_, Mutex<AppDashboardRuntime>>,
) -> Result<FrontendSnapshot, String> {
    let provider_kind = parse_provider_kind(&provider)?;
    Ok(runtime
        .lock()
        .await
        .frontend_snapshot_for_provider(provider_kind)
        .await)
}

#[tauri::command]
async fn get_app_settings(config: tauri::State<'_, Mutex<AppConfig>>) -> Result<AppConfig, String> {
    Ok(config.lock().await.clone())
}

#[tauri::command]
async fn save_app_settings(
    app: AppHandle,
    config: tauri::State<'_, Mutex<AppConfig>>,
    runtime: tauri::State<'_, Mutex<AppDashboardRuntime>>,
    click_through_state: tauri::State<'_, Arc<AtomicBool>>,
    settings: AppConfig,
) -> Result<AppConfig, String> {
    let normalized = compact_widget_positions(settings.normalized());
    set_autostart(normalized.autostart).map_err(|error| error.to_string())?;
    write_config(&config_path(&app)?, &normalized).map_err(|error| error.to_string())?;
    sync_widget_windows(&app, &normalized)?;
    apply_click_through(&app, normalized.click_through)?;
    click_through_state.store(normalized.click_through, Ordering::Relaxed);
    if normalized.click_through {
        ensure_settings_window(&app)?;
    }
    runtime
        .lock()
        .await
        .set_endpoints(normalized.endpoints.clone());
    *config.lock().await = normalized.clone();
    emit_dashboard_settings(&app, &normalized)?;
    Ok(normalized)
}

#[tauri::command]
async fn move_widget_windows(
    app: AppHandle,
    config: tauri::State<'_, Mutex<AppConfig>>,
    provider: String,
    x: i32,
    y: i32,
    persist: bool,
) -> Result<AppConfig, String> {
    let kind = parse_widget_window_kind(&provider)?;
    let mut next_config = {
        let current = config.lock().await;
        current.clone()
    };
    apply_drag_position(&mut next_config, kind, WindowPosition { x, y });
    sync_widget_windows(&app, &next_config)?;
    if persist {
        write_config(&config_path(&app)?, &next_config).map_err(|error| error.to_string())?;
    }
    *config.lock().await = next_config.clone();
    Ok(next_config)
}

#[tauri::command]
async fn open_settings_window(app: AppHandle) -> Result<(), String> {
    ensure_settings_window(&app)?;
    Ok(())
}

#[tauri::command]
async fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WidgetWindowKind {
    Claude,
    Codex,
    Pomodoro,
}

impl WidgetWindowKind {
    const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::Pomodoro];

    fn label(self) -> &'static str {
        match self {
            Self::Claude => "claude-widget",
            Self::Codex => "codex-widget",
            Self::Pomodoro => "pomodoro-widget",
        }
    }

    fn provider(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Pomodoro => "pomodoro",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Claude => "Claude Widget",
            Self::Codex => "Codex Widget",
            Self::Pomodoro => "Pomodoro Widget",
        }
    }

    fn size(self, scale: f64) -> (f64, f64) {
        let clamped = scale.clamp(0.5, 2.0);
        match self {
            Self::Claude | Self::Codex => (160.0 * clamped, 160.0 * clamped),
            Self::Pomodoro => (160.0 * clamped, 198.0 * clamped),
        }
    }

    fn enabled(self, config: &AppConfig) -> bool {
        self.widget_config(config).enabled
    }

    fn widget_config<'a>(self, config: &'a AppConfig) -> &'a WidgetConfig {
        match self {
            Self::Claude => &config.widgets.claude,
            Self::Codex => &config.widgets.codex,
            Self::Pomodoro => &config.widgets.pomodoro,
        }
    }

    fn widget_config_mut<'a>(self, config: &'a mut AppConfig) -> &'a mut WidgetConfig {
        match self {
            Self::Claude => &mut config.widgets.claude,
            Self::Codex => &mut config.widgets.codex,
            Self::Pomodoro => &mut config.widgets.pomodoro,
        }
    }
}

fn ensure_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Token Dashboard Settings")
        .inner_size(760.0, 640.0)
        .min_inner_size(620.0, 520.0)
        .resizable(true)
        .decorations(true)
        .transparent(false)
        .always_on_top(false)
        .skip_taskbar(false)
        .icon(thumbnail_icon()?)
        .map_err(|error| error.to_string())?
        .build()
        .map_err(|error| error.to_string())?;
    let window = app
        .get_webview_window("settings")
        .ok_or_else(|| "settings window was not created".to_string())?;
    let click_through_state = app.state::<Arc<AtomicBool>>().inner().clone();
    let window_for_close = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            if click_through_state.load(Ordering::Relaxed) {
                api.prevent_close();
                let _ = window_for_close.show();
                let _ = window_for_close.set_focus();
            }
        }
    });

    Ok(())
}

fn apply_click_through(app: &AppHandle, enabled: bool) -> Result<(), String> {
    for kind in WidgetWindowKind::ALL {
        if let Some(window) = app.get_webview_window(kind.label()) {
            window
                .set_ignore_cursor_events(enabled)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn emit_dashboard_settings(app: &AppHandle, settings: &AppConfig) -> Result<(), String> {
    for kind in WidgetWindowKind::ALL {
        if app.get_webview_window(kind.label()).is_some() {
            app.emit_to(kind.label(), "app-settings-updated", settings.clone())
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn widget_gap(scale: f64) -> i32 {
    (20.0 * scale.clamp(0.5, 2.0)).round() as i32
}

fn widget_y_offset(_kind: WidgetWindowKind, _scale: f64) -> i32 {
    0
}

fn enabled_widget_anchor(config: &AppConfig) -> Option<WindowPosition> {
    WidgetWindowKind::ALL.into_iter().find_map(|kind| {
        if kind.enabled(config) {
            let position = kind.widget_config(config).position.clone();
            Some(WindowPosition {
                x: position.x,
                y: position.y - widget_y_offset(kind, config.widget_scale),
            })
        } else {
            None
        }
    })
}

/* [REFACTOR] Treat the widget row as one anchored group so enabled-window compaction, shared dragging, and Pomodoro baseline alignment all use the same position model. */
fn apply_group_anchor(config: &mut AppConfig, anchor: WindowPosition) {
    let scale = config.widget_scale;
    let mut next_x = anchor.x;
    let gap = widget_gap(scale);

    for kind in WidgetWindowKind::ALL {
        if !kind.enabled(config) {
            continue;
        }
        let width = kind.size(config.widget_scale).0.round() as i32;
        let widget = kind.widget_config_mut(config);
        widget.position.x = next_x;
        widget.position.y = anchor.y + widget_y_offset(kind, scale);
        next_x += width + gap;
    }
}

fn grouped_widget_offset(config: &AppConfig, target: WidgetWindowKind) -> Option<WindowPosition> {
    let scale = config.widget_scale;
    let gap = widget_gap(scale);
    let mut next_x = 0;

    for kind in WidgetWindowKind::ALL {
        if !kind.enabled(config) {
            continue;
        }
        if kind == target {
            return Some(WindowPosition {
                x: next_x,
                y: widget_y_offset(kind, scale),
            });
        }
        next_x += kind.size(scale).0.round() as i32 + gap;
    }

    None
}

/* [REFACTOR] Route all widget movement through the same anchor model so grouped drag, independent drag, and save-time compaction use one position source of truth. */
fn apply_drag_position(config: &mut AppConfig, kind: WidgetWindowKind, position: WindowPosition) {
    if !kind.enabled(config) {
        return;
    }
    let Some(offset) = grouped_widget_offset(config, kind) else {
        return;
    };
    apply_group_anchor(
        config,
        WindowPosition {
            x: position.x - offset.x,
            y: position.y - offset.y,
        },
    );
}

fn compact_widget_positions(mut config: AppConfig) -> AppConfig {
    let Some(anchor) = enabled_widget_anchor(&config) else {
        return config;
    };
    apply_group_anchor(&mut config, anchor);

    config
}

/* [REFACTOR] Split the old shared transparent dashboard into one fixed-size window per widget so X11 never has to clear removed sibling gauges from a single alpha surface. */
fn sync_widget_windows(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    for kind in WidgetWindowKind::ALL {
        if kind.enabled(config) {
            ensure_widget_window(app, kind, config)?;
        } else if let Some(window) = app.get_webview_window(kind.label()) {
            window.close().map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn ensure_widget_window(
    app: &AppHandle,
    kind: WidgetWindowKind,
    config: &AppConfig,
) -> Result<(), String> {
    let widget = kind.widget_config(config);
    let (width, height) = kind.size(config.widget_scale);
    let position = Position::Logical(LogicalPosition::new(
        f64::from(widget.position.x),
        f64::from(widget.position.y),
    ));
    let size = Size::Logical(LogicalSize::new(width, height));

    if let Some(window) = app.get_webview_window(kind.label()) {
        window
            .set_position(position)
            .map_err(|error| error.to_string())?;
        window.set_size(size).map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, kind.label(), WebviewUrl::App("index.html".into()))
        .title(kind.title())
        .position(f64::from(widget.position.x), f64::from(widget.position.y))
        .inner_size(width, height)
        .min_inner_size(width, height)
        .max_inner_size(width, height)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .initialization_script(&format!(
            "window.__TOKEN_DASHBOARD_WIDGET__ = '{}';",
            kind.provider()
        ))
        .icon(thumbnail_icon()?)
        .map_err(|error| error.to_string())?
        .build()
        .map_err(|error| error.to_string())?;
    let window = app
        .get_webview_window(kind.label())
        .ok_or_else(|| format!("{} window was not created", kind.label()))?;
    window.show().map_err(|error| error.to_string())
}

fn parse_provider_kind(provider: &str) -> Result<ProviderKind, String> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "claude" => Ok(ProviderKind::Claude),
        "codex" => Ok(ProviderKind::Codex),
        _ => Err(format!("unsupported usage provider: {provider}")),
    }
}

fn parse_widget_window_kind(provider: &str) -> Result<WidgetWindowKind, String> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "claude" => Ok(WidgetWindowKind::Claude),
        "codex" => Ok(WidgetWindowKind::Codex),
        "pomodoro" => Ok(WidgetWindowKind::Pomodoro),
        _ => Err(format!("unsupported widget provider: {provider}")),
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("config.json"))
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        if std::env::var("XDG_SESSION_TYPE")
            .map(|session| session.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
        {
            eprintln!(
                "token-dashboard warning: Ubuntu X11 is the supported Linux target; Wayland is not a v1 target."
            );
        }
    }

    tauri::Builder::default()
        .setup(|app| {
            let path = config_path(app.handle()).map_err(std::io::Error::other)?;
            let config = compact_widget_positions(load_or_create_config(&path)?);
            let click_through_state = Arc::new(AtomicBool::new(config.click_through));
            app.manage(click_through_state);
            app.manage(Mutex::new(config.clone()));
            if config.autostart {
                set_autostart(true).map_err(std::io::Error::other)?;
            }
            app.manage(Mutex::new(DashboardRuntime::new(
                config.endpoints.clone(),
                token_dashboard::http::ReqwestUsageHttpClient::default(),
                token_dashboard::refresh::ReqwestRefreshHttpClient::default(),
                token_dashboard::dashboard::DefaultCredentialSource,
            )));
            sync_widget_windows(app.handle(), &config).map_err(std::io::Error::other)?;
            apply_click_through(
                app.handle(),
                app.state::<Arc<AtomicBool>>()
                    .inner()
                    .load(Ordering::Relaxed),
            )
            .map_err(std::io::Error::other)?;
            if app
                .state::<Arc<AtomicBool>>()
                .inner()
                .load(Ordering::Relaxed)
            {
                ensure_settings_window(app.handle()).map_err(std::io::Error::other)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            usage_snapshots,
            usage_snapshot,
            get_app_settings,
            save_app_settings,
            move_widget_windows,
            open_settings_window,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Token Dashboard");
}

#[cfg(test)]
mod tests {
    use super::*;
    use token_dashboard::config::AppConfig;

    #[test]
    fn compacts_enabled_widgets_without_middle_gaps() {
        let mut config = AppConfig::default();
        config.widget_scale = 1.0;
        config.widgets.codex.enabled = false;

        let compacted = compact_widget_positions(config);

        assert_eq!(compacted.widgets.claude.position.x, 120);
        assert_eq!(compacted.widgets.claude.position.y, 80);
        assert_eq!(compacted.widgets.pomodoro.position.x, 300);
        assert_eq!(compacted.widgets.pomodoro.position.y, 80);
    }

    #[test]
    fn preserves_anchor_of_first_enabled_widget() {
        let mut config = AppConfig::default();
        config.widget_scale = 1.0;
        config.widgets.claude.enabled = false;

        let compacted = compact_widget_positions(config);

        assert_eq!(compacted.widgets.codex.position.x, 280);
        assert_eq!(compacted.widgets.codex.position.y, 80);
        assert_eq!(compacted.widgets.pomodoro.position.x, 460);
        assert_eq!(compacted.widgets.pomodoro.position.y, 80);
    }
}
