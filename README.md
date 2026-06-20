# Token Dashboard

Token Dashboard is an unofficial local-only Tauri 2 floating widget for:

- Claude Code usage
- Codex CLI usage
- Pomodoro timing

It is designed for macOS Apple Silicon and Ubuntu X11. Tokens stay in the original CLI-owned locations and are never copied into the app config.

## Status

This project is preparing a `v0.1.0` release. The app currently ships unsigned artifacts through GitHub Releases.

## Privacy

- No telemetry.
- No central server.
- No usage history upload.
- No web scraping for usage data.
- The app reads Claude/Codex credentials from the original CLI-owned locations only.
- Refreshed access tokens are kept in memory only.
- Config files must never contain access tokens, refresh tokens, ID tokens, API keys, or authorization headers.

This is not affiliated with Anthropic or OpenAI. Claude and Codex names are used only to identify the local data source shown in the widget.

## Requirements

### macOS

- Apple Silicon Mac.
- Claude Code login must already exist in Keychain service `Claude Code-credentials`.
- Codex CLI login must already exist in the normal Codex CLI auth location.
- The app is unsigned, so macOS Gatekeeper may require manual approval.

### Ubuntu

- Ubuntu 22.04 X11 is the v1 target.
- Wayland is not a v1 target; XWayland behavior is not guaranteed.
- Claude Code login: `~/.claude/.credentials.json`.
- Codex CLI login: `~/.codex/auth.json`.
- Codex auth must be ChatGPT login mode.
- Token files should not be broader than `600`; the app warns safely when permissions are loose.

## Install

Download the latest draft/release artifact from GitHub Releases:

- macOS: `.dmg`
- Ubuntu: `.deb` or `.AppImage`

For Ubuntu `.deb`:

```bash
sudo apt install ./token-dashboard_0.1.0_amd64.deb
```

For Ubuntu `.AppImage`:

```bash
chmod +x ./token-dashboard_0.1.0_amd64.AppImage
./token-dashboard_0.1.0_amd64.AppImage
```

For macOS unsigned builds, see [docs/troubleshooting.md](docs/troubleshooting.md).

## Use

- The app opens floating widgets for Claude, Codex, and Pomodoro.
- Right-click a widget to open Settings.
- Drag a widget to move the grouped row.
- Use Settings to choose visible widgets, polling interval, widget scale, Pomodoro durations, autostart, and Quit.
- Use the Settings `Quit` button to terminate the app because widgets are frameless and skipped from the taskbar.
- Notifications and click-through are not exposed in the v1 settings surface.

## Settings

- `Widgets`: show or hide Claude, Codex, and Pomodoro widgets.
- `Polling interval`: usage provider polling interval in seconds. Values below the app minimum are clamped.
- `Widget scale`: visual size of the floating widgets.
- `Autostart`: starts Token Dashboard on login.
- `Pomodoro`: focus and break minute settings.
- `Advanced endpoints`: optional usage endpoint overrides for undocumented API changes.
- `Quit`: exits the app.

Autostart uses an XDG autostart `.desktop` file on Linux and a LaunchAgent plist on macOS.

## Limitations

- No Windows support.
- No Wayland-native support in v1.
- No multi-account support.
- No system tray.
- No notification surface in v1.
- No self-updating installer.

## Local Development

CI-safe tests do not call real Claude or Codex APIs.

```bash
npm install
npm run build
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

Run the desktop app locally:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin token-dashboard
```

Run real local smoke checks only when you explicitly allow API calls:

```bash
TOKEN_DASHBOARD_ALLOW_REAL_API=1 ./scripts/local-smoke.sh --provider all
```

Never run local smoke tests in CI and never paste auth file contents into bug reports.

## Build Packages Locally

Install the Tauri CLI, then build the platform-specific bundles:

```bash
cargo install tauri-cli --version "^2"
cargo tauri build --bundles deb,appimage
```

On macOS:

```bash
cargo tauri build --target aarch64-apple-darwin --bundles dmg
```

GitHub Actions builds release artifacts from `v*` tags and creates a draft release.
