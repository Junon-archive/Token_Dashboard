# Token Dashboard — Codex Project Instructions

## Source of truth

- The primary product specification is `SPEC.md`.
- If implementation details conflict with `SPEC.md`, follow `SPEC.md` unless the user explicitly overrides it.
- Do not expand v1 scope beyond the spec.
- Do not add Windows support, account systems, telemetry, central servers, web scraping, usage history graphs, or self-updating installers.

## Product summary

Build a Tauri 2 desktop floating widget app for macOS Apple Silicon and Ubuntu X11.

The app displays:
- Claude Code usage gauge
- Codex CLI usage gauge
- Pomodoro timer gauge

Core requirements:
- Tokens must stay local.
- Never persist copied tokens in app config.
- Usage providers read tokens from the original CLI-owned locations.
- Claude and Codex providers must degrade safely to STALE/AUTH_ERROR/NOT_LOGGED_IN instead of crashing.
- Pomodoro must work even when all network and token usage providers fail.
- UI must follow the design-reference gauge style described in `SPEC.md`.

## Implementation priorities

Implement in this order:

1. M1 data layer:
   - Rust UsageProvider trait
   - ClaudeProvider
   - CodexProvider
   - token source readers
   - snapshot normalization
   - state machine
   - fixtures and unit tests

2. M2 single widget UI:
   - transparent Tauri window
   - Claude mock/state rendering
   - 7 logical state visual mapping
   - platform window behavior

3. M3 full widgets and settings:
   - Codex widget
   - Pomodoro widget
   - config persistence
   - notifications
   - click-through
   - autostart
   - settings window

4. M4 packaging:
   - GitHub Actions
   - macOS dmg
   - Ubuntu deb/AppImage
   - README and release notes

## Security and privacy rules

- Do not log access tokens, refresh tokens, ID tokens, API keys, authorization headers, or full auth JSON.
- Mask secrets in all debug output.
- Do not send tokens anywhere except the required usage/refresh endpoints.
- Do not introduce telemetry.
- Do not add a central server.
- Do not scrape web pages for usage data.
- Config files must never contain tokens.

## Testing rules

- Add or update tests with every behavior change.
- CI must not call real Claude/Codex APIs.
- Real API smoke tests must be local-only scripts.
- Required tests include:
  - UsageSnapshot conversion fixtures
  - RFC3339 `+00:00` and `Z` parsing
  - epoch reset conversion
  - schema mismatch -> STALE
  - backoff sequence
  - token masking
  - 7 logical states mapped to UI classes
  - Pomodoro working with network disabled and token files missing

## Platform rules

macOS:
- Claude token source is Keychain service `Claude Code-credentials`.
- Use Security framework first, `security` CLI fallback second.
- Tauri transparent window requires app-level `macOSPrivateApi = true` and the Rust `macos-private-api` feature.

Ubuntu X11:
- Claude token path: `~/.claude/.credentials.json`
- Codex token path: `~/.codex/auth.json`
- Warn if token file permissions are broader than 600.
- Ensure `WEBKIT_DISABLE_DMABUF_RENDERER=1` is set automatically.
- Wayland is not a v1 target.

## Git rules

- Use repository: `https://github.com/Junon-archive/Token_Dashboard.git`
- Work in small commits.
- Commit only after tests/lint relevant to the change pass.
- Never commit real tokens, local auth files, build artifacts, or `.env` secrets.
- Commit messages should be concise and conventional:
  - `feat: ...`
  - `fix: ...`
  - `test: ...`
  - `docs: ...`
  - `ci: ...`
  - `chore: ...`

## Subagent orchestration

- Use main-planner for milestone planning and task sequencing.
- Use architect before large structural changes.
- Use api-researcher for undocumented usage endpoint investigation.
- Use security-privacy-specialist before token/auth/logging changes.
- Use ui-ux-specialist before gauge rendering changes.
- Use platform-specialist for macOS/Linux windowing and packaging quirks.
- Use test-specialist before declaring a milestone complete.
- Use git-specialist only after implementation and tests are complete.

Do not let multiple write-capable agents edit the same files at the same time.
Read-only agents should report findings, not modify files.