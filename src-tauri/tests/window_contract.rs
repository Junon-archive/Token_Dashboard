use serde_json::Value;
use token_dashboard::config::AppConfig;

#[test]
fn default_widget_positions_stay_stable() {
    let config = AppConfig::default();

    assert!(config.grouped_widgets);
    assert_eq!(config.widgets.claude.position.x, 120);
    assert_eq!(config.widgets.claude.position.y, 80);
    assert_eq!(config.widgets.codex.position.x, 280);
    assert_eq!(config.widgets.codex.position.y, 80);
    assert_eq!(config.widgets.pomodoro.position.x, 440);
    assert_eq!(config.widgets.pomodoro.position.y, 80);
}

#[test]
fn capability_allows_every_widget_window() {
    let capability: Value = serde_json::from_str(include_str!("../capabilities/default.json"))
        .expect("default capability should parse");
    let windows = capability["windows"]
        .as_array()
        .expect("capability windows should be an array");

    for label in [
        "claude-widget",
        "codex-widget",
        "pomodoro-widget",
        "settings",
    ] {
        assert!(
            windows.iter().any(|item| item.as_str() == Some(label)),
            "missing capability window label: {label}"
        );
    }

    let permissions = capability["permissions"]
        .as_array()
        .expect("capability permissions should be an array");
    assert!(permissions
        .iter()
        .any(|item| item.as_str() == Some("core:window:allow-start-dragging")));
}

#[test]
fn tauri_config_uses_dynamic_widget_windows() {
    let config: Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .expect("tauri config should parse");

    assert_eq!(config["app"]["withGlobalTauri"], Value::Bool(true));
    assert_eq!(config["app"]["macOSPrivateApi"], Value::Bool(true));
    assert_eq!(config["app"]["windows"], Value::Array(Vec::new()));
}
