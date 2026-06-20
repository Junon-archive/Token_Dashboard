# Release Notes

## v0.1.0

Initial unsigned desktop release target.

### Supported Platforms

- macOS Apple Silicon
- Ubuntu 22.04 X11

### Included

- Claude Code usage gauge
- Codex CLI usage gauge
- Pomodoro gauge and controls
- Settings window opened from widget right-click
- Widget visibility, polling interval, widget scale, Pomodoro duration, endpoint override, autostart, and Quit settings
- Local-only smoke test script

### Privacy

- Tokens stay in the original CLI-owned locations.
- Refreshed access tokens are memory-only.
- App config must not contain tokens.
- No telemetry, central server, usage history upload, or web scraping.

### Known Limitations

- Unsigned macOS app; Gatekeeper approval is required.
- Ubuntu X11 only; Wayland is not a v1 target.
- No Windows support.
- No multi-account support.
- No visible notification or click-through settings in v1.
- No self-updating installer.

### Before Publishing

- Verify macOS Keychain access on Apple Silicon.
- Verify Ubuntu `.deb` and AppImage install/run behavior on X11.
- Verify generated package icons from `assets/Thumbnail.png`.
- Verify autostart from installed artifacts on macOS and Ubuntu.

