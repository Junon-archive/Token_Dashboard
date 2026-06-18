# Token Dashboard Roadmap

## Current Status
- Current milestone: M4 — Packaging and Release
- Current task: Start packaging/release preparation from committed M3 baseline
- Last completed task: Committed M3 widgets/settings checkpoint as `27fa9e1 feat: complete M3 widgets and settings`
- Last command run: `git commit -m "feat: complete M3 widgets and settings"`, `git status --short`, `git log --oneline --decorate -3`
- Last test result: Passed before commit — frontend notifications/Pomodoro/settings/widget tests and Rust 58 lib tests, 4 smoke tests, 5 contract tests
- Next recommended command: Review packaging targets and start M4 with CI/package metadata planning
- Blocking issue: None for M3. macOS Keychain Security framework first path and OS notification display remain unverified on Ubuntu and should be handled during M4 packaging validation.
- Git status note: `.codex/` remains local untracked tooling config and should not be committed. The screenshot reference file is local input and is not required for runtime.
- Updated at: 2026-06-15 10:25 UTC
- Updated at: 2026-06-16 00:00 UTC
- Updated at: 2026-06-16 01:00 UTC
- Updated at: 2026-06-17 00:30 UTC
- Updated at: 2026-06-17 03:30 UTC
- Updated at: 2026-06-18 08:10 UTC
- Updated at: 2026-06-18 08:40 UTC
- Updated at: 2026-06-18 09:20 UTC
- Updated at: 2026-06-18 09:35 UTC
- Updated at: 2026-06-18 09:50 UTC
- Updated at: 2026-06-18 10:05 UTC

## Source Documents Read
- [x] SPEC.md
- [x] for_specification PoC files
- [x] design reference files

## Decisions
| Date | Decision | Reason | Files/Sections |
|---|---|---|---|
| 2026-06-11 | Start with a GUI-free Rust data-layer crate under `src-tauri` | M1 requires provider parsing/state logic before UI; Tauri can attach to the same crate in M2 | SPEC.md §§3-5, §11 |
| 2026-06-11 | Keep Codex usage endpoint path configurable until source-backed | PoC used `codex-check`; direct path/refresh host were initially not confirmed | SPEC.md §§2.4, 4.3, R-3 |
| 2026-06-11 | Set Codex usage default to `chatgpt.com/backend-api/wham/usage` and document refresh host `auth.openai.com/oauth/token` | Confirmed by static analysis of public `codex-check@1.3.13`; app keeps refresh memory-only and allowlisted | `src-tauri/src/config.rs`, `src-tauri/src/providers/codex.rs`, `src-tauri/src/refresh.rs` |
| 2026-06-11 | Automatic tests use fixtures only; real API checks are local-only scripts | CI must not call real APIs or print auth material | SPEC.md §§2, 10 |
| 2026-06-11 | Add source-backed refresh retry for Claude and Codex, but keep refreshed tokens memory-only | Claude CLI 2.1.170 and codex-check 1.3.13 expose refresh endpoint/client metadata; SPEC forbids writing CLI-owned auth files | `src-tauri/src/refresh.rs`, `src-tauri/src/providers/claude.rs`, `src-tauri/src/providers/codex.rs` |
| 2026-06-11 | Reject token-like material in app config on load/write | Unknown config keys are preserved only when they do not contain token-like keys or Bearer strings | `src-tauri/src/config.rs` |
| 2026-06-11 | Keep hover disk/glow effects disabled on Linux transparent WebKit, but expose last update age on hover | Manual M2 checks showed persistent transparent-window artifacts from disk/shadow hover effects; UI-5 still needs a hover-accessible update age | `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/src/widget.js`, `frontend/tests/widget.test.mjs` |
| 2026-06-11 | Generalize the usage widget renderer before adding the full Codex widget | This is safe while M2 visual verification is blocked because it is frontend-only, provider-neutral, and covered by Node DOM tests | `frontend/src/widget.js`, `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs` |
| 2026-06-11 | Add Codex as a mock dashboard widget before real provider polling | Keeps M3 UI work moving without reading tokens or calling real APIs from the app shell | `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`, `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs` |
| 2026-06-13 | Accept screenshot-style block tick marks for the gauge | Manual review found the exact design-reference line ticks too plain on Linux desktop backgrounds; rounded block ticks with 45-degree state-colored major marks look closer to the provided reference | `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs` |
| 2026-06-13 | Expose frontend-safe provider DTO instead of internal `UsageSnapshot` | Real provider wiring must not expose raw provider errors, extras, endpoints, token paths, account IDs, or auth material to the webview | `src-tauri/src/dashboard.rs`, `src-tauri/src/main.rs` |
| 2026-06-13 | Restrict token-bearing Codex usage endpoint to `/backend-api/wham/usage` | Real dashboard polling should send bearer tokens only to the confirmed usage endpoint, not arbitrary `chatgpt.com/backend-api/*` paths | `src-tauri/src/config.rs`, `src-tauri/src/providers/codex.rs`, `src-tauri/tests/m1_contract.rs` |
| 2026-06-13 | Enable Tauri global API for the widget webview | The frontend uses `window.__TAURI__.core.invoke`; without `withGlobalTauri`, the app rendered browser fallback mock data instead of real provider snapshots | `src-tauri/tauri.conf.json`, `frontend/src/main.js` |
| 2026-06-13 | Keep Pomodoro timer frontend-local in the first slice | SPEC separates Pomodoro from provider polling; the first widget can render and tick locally while Rust notification commands/settings persistence remain later M3 tasks | `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css` |
| 2026-06-13 | Avoid 1-second full DOM rerenders in transparent WebKit | Manual check showed repeated opacity accumulation/reset artifacts when the Pomodoro shell replaced the entire dashboard every second; minute-level redraw matches the displayed minute precision | `frontend/src/main.js`, `frontend/tests/widget.test.mjs` |
| 2026-06-15 | Treat Claude refresh `400 invalid_grant` as `AUTH_ERROR` | Current local Claude credential has an expired/invalid access token and a refresh token rejected by Claude's OAuth endpoint; UI should show an auth problem rather than generic stale/network | `src-tauri/src/refresh.rs` |
| 2026-06-15 | Codex WARN/yellow can be caused by the secondary 7-day window | Codex smoke showed primary 5-hour usage at 9% but secondary 7-day usage at 80%; the state machine uses the max of primary/secondary usage, so WARN is expected | `src-tauri/src/providers/codex.rs`, `src-tauri/src/state.rs` |
| 2026-06-15 | Keep Pomodoro controls frontend-local and hover-only | This preserves Pomodoro isolation, avoids settings/notification scope creep, and keeps the transparent widget mostly draggable while exposing controls only when needed | `frontend/src/pomodoro.js`, `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css` |
| 2026-06-15 | Use 12 major ticks only for Pomodoro | Pomodoro benefits from clock-like 5-minute divisions; Claude/Codex retain the accepted 8 major usage ticks | `frontend/src/widget.js`, `frontend/tests/widget.test.mjs` |
| 2026-06-15 | Avoid full-dashboard timer redraws in the transparent window | Linux transparent WebKit can leave stale glyph/control paint when DOM subtrees are replaced; timer updates now mutate text/classes/arc attributes in place and use a small numeric paint plate | `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs` |
| 2026-06-15 | Keep Pomodoro minute editing frontend-local for now | Center-click minute setting is useful before persistent settings; the value changes the current phase only, clamps to 1-180 minutes, and resets that phase paused until config persistence is added | `frontend/src/pomodoro.js`, `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/tests/pomodoro.test.mjs` |
| 2026-06-16 | Make Pomodoro timing follow the active phase duration and redraw faster | A 5-minute timer should shrink over 5 minutes and a 1-minute timer should complete over 60 seconds, so Pomodoro now uses phase-duration progress with a sub-second UI cadence | `frontend/src/pomodoro.js`, `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/pomodoro.test.mjs`, `frontend/tests/widget.test.mjs` |
| 2026-06-16 | Add a blinking Pomodoro end state before phase handoff | Users need a visible completion cue, so a finished timer now blinks for 30 seconds, can be acknowledged by clicking, and hands off the next phase paused instead of auto-starting | `frontend/src/pomodoro.js`, `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/pomodoro.test.mjs`, `frontend/tests/widget.test.mjs` |
| 2026-06-16 | Keep Pomodoro hover active across the button gap | The controls sit below the gauge, so the hover region now bridges the gap to keep the buttons clickable instead of collapsing as the pointer moves downward | `frontend/src/main.js`, `frontend/tests/widget.test.mjs` |
| 2026-06-16 | Keep the Pomodoro toggle button node fixed | Transparent WebKit can become unstable when the control is recreated, so the toggle now stays mounted and only its label/class changes in place | `frontend/src/main.js`, `frontend/tests/widget.test.mjs` |
| 2026-06-16 | Move Pomodoro controls out of hover overlay into a fixed action row | The button row now lives in the widget flow below the gauge instead of depending on hover overlays, which removes the unstable hover/click boundary | `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs` |
| 2026-06-16 | Make the Pomodoro control row an opaque paint island | Transparent WebKit can leave old button paint visible through translucent controls, so the row and buttons now use opaque backgrounds with paint containment and isolation | `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs` |
| 2026-06-16 | Top-align dashboard widgets after adding the Pomodoro button row | Pomodoro is taller than Claude/Codex after the fixed controls row, so the dashboard now aligns widget tops to keep the three circular gauges level | `frontend/src/styles.css`, `frontend/tests/widget.test.mjs` |
| 2026-06-16 | Add settings before click-through | Once click-through is enabled the widget cannot receive right-click or button events, so settings access and persistence must exist first as the recovery path foundation | `src-tauri/src/main.rs`, `settings.html`, `frontend/src/settings.js`, `src-tauri/src/config.rs` |
| 2026-06-16 | Keep settings window opaque and separate from the transparent widget | Settings should not participate in the Linux transparent WebKit repaint path, and it needs normal decorations/taskbar behavior for recoverability | `src-tauri/src/main.rs`, `settings.html`, `frontend/src/settings.css` |
| 2026-06-16 | Implement notification thresholds as pure snapshot logic first | The threshold rules must be CI-safe and token-free; actual OS notification display remains a thin dispatch layer and a platform smoke concern | `frontend/src/notifications.js`, `frontend/tests/notifications.test.mjs`, `frontend/src/main.js` |
| 2026-06-16 | Open settings automatically when click-through is persisted on | Click-through intentionally prevents right-click, drag, and Pomodoro button events on the widget, so startup must provide a visible recovery path to turn it off | `src-tauri/src/main.rs` |
| 2026-06-16 | Implement autostart with OS-native files instead of a new plugin | Avoids adding dependency/network risk during M3; Linux uses XDG autostart desktop entries and macOS uses LaunchAgent plist files | `src-tauri/src/autostart.rs`, `src-tauri/src/main.rs` |
| 2026-06-17 | Re-read settings from the dashboard runtime instead of assuming save implies application | Settings are edited in a separate window, so the transparent widget webview must poll persisted settings and apply widget visibility/Pomodoro duration changes in place | `frontend/src/main.js`, `frontend/src/pomodoro.js` |
| 2026-06-17 | Keep settings window open while click-through is enabled | If the settings window can close while click-through is on, the user loses the right-click recovery path; close requests now refocus the settings window until click-through is disabled | `src-tauri/src/main.rs`, `frontend/src/settings.js` |
| 2026-06-17 | Reconcile widget visibility in place instead of repainting the full dashboard root | The old full-window clear plate could itself surface as a long rectangular opaque block on Linux X11 transparent WebKit, so widget visibility changes now add/remove/reorder widget sections without a whole-window repaint pass | `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `RENDERING_REFACTOR.md` |
| 2026-06-17 | Split the shared dashboard into one transparent window per widget | The remaining X11/WebKitGTK ghost was tied to removing sibling gauges inside one transparent top-level surface, so Claude/Codex/Pomodoro now live in independent widget windows with provider-scoped frontend bootstrapping | `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `frontend/src/main.js`, `frontend/src/settings.js`, `frontend/tests/widget.test.mjs`, `src-tauri/tests/window_contract.rs`, `RENDERING_REFACTOR.md` |
| 2026-06-18 | Make grouped movement a persisted layout mode and route drag through one Rust position path | The native moved-hook approach was unstable on Linux X11; explicit drag commands keep grouped/independent movement under one config-backed layout model and let widget visibility compaction remain deterministic only when grouping is enabled | `src-tauri/src/main.rs`, `src-tauri/src/config.rs`, `frontend/src/main.js`, `frontend/src/settings.js`, `frontend/tests/settings.test.mjs`, `frontend/tests/widget.test.mjs`, `src-tauri/tests/window_contract.rs` |
| 2026-06-18 | Remove optional ungrouped layout and keep widgets always grouped | Manual verification showed the grouped row is stable while ungrouped mode adds complexity without product value; config normalization now forces grouped layout and settings no longer expose a split-mode toggle | `src-tauri/src/config.rs`, `src-tauri/src/main.rs`, `frontend/src/settings.js`, `frontend/src/main.js`, `frontend/tests/settings.test.mjs`, `roadmap.md` |
| 2026-06-18 | Use opaque gauge disks and avoid stale opacity dimming | Linux WebKitGTK transparent windows can leave rectangular text backing layers when text/disk/arc opacity changes; stale state is now shown via update badge while gauge paint remains opaque | `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `RENDERING_REFACTOR.md` |
| 2026-06-18 | Strengthen M3 pre-commit security guardrails | Config writes now reject invalid endpoint overrides before persistence, token-like keys/values are detected more broadly, runtime token-file permission warnings are sanitized, and PoC local home paths are redacted before M4 docs work | `src-tauri/src/config.rs`, `src-tauri/src/dashboard.rs`, `for_specification/poc-result-ubuntu.md`, `roadmap.md` |

## Milestone Checklist

### M1 — Data Layer
- [x] Read SPEC.md and PoC materials
- [x] Define Rust module structure
- [x] Define UsageProvider trait
- [x] Define UsageSnapshot schema
- [x] Implement Claude token source
- [x] Implement Claude usage parser
- [x] Implement Codex token source
- [x] Investigate Codex direct usage endpoint
- [x] Implement Codex usage parser
- [x] Implement refresh/cache policy where safe
- [x] Implement state machine
- [x] Implement backoff
- [x] Implement token masking
- [x] Add fixture tests
- [x] Add local-only smoke script
- [x] Run tests
- [x] Commit M1

### M2 — Single Widget UI
- [x] Implement transparent Tauri window
- [x] Implement Claude widget rendering
- [x] Implement design tokens
- [x] Implement 7 logical state visual mapping
- [x] Add mock provider UI tests
- [x] Add hover-accessible last update display
- [x] Verify Linux X11 behavior where possible
- [x] Commit M2

### M3 — Three Widgets and Settings
- [x] Generalize usage widget renderer for Claude/Codex provider views
- [x] Add Codex widget shell with mock snapshot
- [x] Wire Claude/Codex widgets to real provider runtime
- [x] Add Pomodoro widget
- [x] Add Pomodoro controls and phase switching
- [x] Add Pomodoro center-minute editing
- [x] Add settings window
- [x] Add config persistence
- [x] Add notification thresholds
- [x] Add click-through
- [x] Add autostart
- [x] Add Pomodoro isolation test
- [x] Commit M3

### M4 — Packaging and Release
- [ ] Add GitHub Actions
- [ ] Build macOS dmg
- [ ] Build Ubuntu deb/AppImage
- [ ] Write README
- [ ] Write troubleshooting docs
- [ ] Verify release artifact behavior
- [ ] Commit M4
- [ ] Push to GitHub

## Work Log
### 2026-06-11 00:00
- Agent: main
- Task: Initial orientation and M1 scaffold planning
- Files changed: `roadmap.md`
- Commands run: `git status --short`, `git remote -v`, `rg --files for_specification`, `sed` reads for SPEC/PoC/design, version checks
- Result: Source documents read; M1 scaffold direction chosen
- Next step: Add Rust data-layer crate, fixtures, and tests

### 2026-06-11 00:00
- Agent: main + planner/architect/api/security read-only subagents
- Task: Implement first M1 data-layer slice
- Files changed: `src-tauri/Cargo.toml`, `src-tauri/src/*`, `src-tauri/tests/*`, `scripts/local-smoke.sh`, `.gitignore`, PoC docs redactions
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml`, sensitive metadata `rg` scan
- Result: 28 Rust tests passed; endpoint host allowlist added; PoC email/account/auth path metadata redacted; last-good stale/rate-limit preservation and memory-only refresh cache tests added
- Next step: Commit first M1 checkpoint, then add provider HTTP runtime and safer refresh-host handling

### 2026-06-11 00:00
- Agent: main + git-specialist
- Task: Commit first safe checkpoints
- Files changed: git history
- Commands run: `git commit -m "feat: add M1 usage data layer scaffold"`, `git commit -m "docs: add project spec and roadmap"`
- Result: Created commits `c7f7ca6` and `b90a0f6`; `.codex/` remains untracked local tooling config
- Next step: Add HTTP client abstraction and provider orchestration without real API tests

### 2026-06-11 00:00
- Agent: main
- Task: Add provider HTTP abstraction
- Files changed: `src-tauri/src/http.rs`, provider modules, runtime, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: 32 tests passed; HTTP clients validate endpoint allowlist before attaching bearer tokens; provider status mapping tested with fixture client only
- Next step: Commit provider HTTP checkpoint, then implement config persistence defaults/clamp/corrupt recovery tests

### 2026-06-11 00:00
- Agent: main
- Task: Add read-only token source readers
- Files changed: `src-tauri/src/token_source.rs`, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: 34 tests passed; synthetic temp-file tests cover Codex and Linux Claude readers without reading real auth files; macOS Claude uses `security` CLI path with no file fallback
- Next step: Commit token source checkpoint, then add config persistence defaults/clamp/corrupt recovery tests

### 2026-06-11 00:00
- Agent: main
- Task: Add refresh policy guardrails
- Files changed: `src-tauri/src/refresh.rs`, `src-tauri/src/lib.rs`, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: 37 tests passed; undocumented refresh endpoints are blocked before network; refresh success writes only memory cache and returns non-warning state
- Next step: Commit refresh policy checkpoint, then add config persistence defaults/clamp/corrupt recovery tests

### 2026-06-11 00:00
- Agent: main
- Task: Add config guardrails
- Files changed: `src-tauri/src/config.rs`, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: 41 tests passed; config clamps polling minimum to 120s, preserves unknown top-level keys, backs up corrupted config, and detects token-like config material
- Next step: Commit config guardrails checkpoint, then run final M1 status scan and decide whether to push current safe commits

### 2026-06-11 00:00
- Agent: main
- Task: Final checkpoint before push
- Files changed: `roadmap.md`
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml`, `git status --short`, `git log --oneline --decorate -10`
- Result: 41 tests passed; six commits exist on `main`; only `.codex/` remains untracked as local tooling config
- Next step: Commit roadmap status and push `main` to `origin`

### 2026-06-11 00:00
- Agent: main
- Task: Confirm Codex direct endpoint from public package source
- Files changed: `src-tauri/src/config.rs`, `src-tauri/src/providers/codex.rs`, `src-tauri/src/refresh.rs`, `src-tauri/tests/fixtures/codex_raw_usage.json`, `src-tauri/tests/m1_contract.rs`, `roadmap.md`
- Commands run: `npm view codex-check repository version dist.tarball`, `npm pack codex-check --pack-destination /tmp`, `rg`/`sed` over `/tmp/codex-check-1.3.13/package/index.mjs`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: 44 tests passed; confirmed usage path `/backend-api/wham/usage`, `ChatGPT-Account-Id` header, and refresh host `auth.openai.com/oauth/token`; no auth files read and no real API calls made
- Next step: Commit and push this endpoint research update

### 2026-06-11 00:00
- Agent: main
- Task: Add local-only smoke runner
- Files changed: `src-tauri/src/bin/local_smoke.rs`, `scripts/local-smoke.sh`, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml`, `./scripts/local-smoke.sh --provider codex`
- Result: 47 tests passed; smoke script refuses to run without `TOKEN_DASHBOARD_ALLOW_REAL_API=1`; no real auth files or APIs were touched
- Next step: Commit smoke runner, then request permission to run one-shot real local smoke

### 2026-06-11 09:38
- Agent: main + api-researcher + security-privacy-specialist
- Task: Add source-backed refresh orchestration and M1 security hardening
- Files changed: `src-tauri/src/config.rs`, `src-tauri/src/token_source.rs`, `src-tauri/src/refresh.rs`, `src-tauri/src/refresh_cache.rs`, `src-tauri/src/providers/claude.rs`, `src-tauri/src/providers/codex.rs`, `src-tauri/src/bin/local_smoke.rs`, `roadmap.md`
- Commands run: `strings` static inspection of installed Claude CLI binary, `rg` over public codex-check source, `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml`, sensitive-pattern `rg --pcre2` scan
- Result: 57 tests passed; Claude/Codex 401 refresh retry path is fixture-tested; refreshed tokens stay memory-only; config rejects token-like material; secret-bearing Debug output is redacted
- Next step: Commit refresh/security hardening, then run guarded one-shot local smoke only with explicit local permission

### 2026-06-11 09:48
- Agent: main
- Task: Run guarded one-shot local smoke
- Files changed: `roadmap.md`, `scripts/local-smoke.sh`
- Commands run: `TOKEN_DASHBOARD_ALLOW_REAL_API=1 ./scripts/local-smoke.sh --provider all`
- Result: No tokens or auth files printed. Codex direct usage returned a WARN snapshot with primary 86% and secondary 47%. Claude degraded to AUTH_ERROR, so the local Claude credential state likely still needs CLI-side re-login or further endpoint compatibility investigation.
- Next step: Commit smoke result; optionally run 20-poll Codex smoke only with explicit local permission

### 2026-06-11 10:34
- Agent: main
- Task: Start long-running Codex smoke
- Files changed: `roadmap.md`
- Commands run: `env TOKEN_DASHBOARD_ALLOW_REAL_API=1 ./scripts/local-smoke.sh --provider codex --polls 20 --interval-sec 180`
- Result: Session `57479` completed; all 20 polls returned WARN snapshots, without printing tokens or auth files
- Next step: Decide whether Claude should be retried after a CLI relogin or whether to proceed to remaining M1 non-account checks

### 2026-06-11 05:47 UTC
- Agent: main
- Task: Diagnose Claude smoke AUTH_ERROR and verify after parser fix
- Files changed: `src-tauri/src/token_source.rs`, `roadmap.md`
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml`, `TOKEN_DASHBOARD_ALLOW_REAL_API=1 /home/junon/Token_Dashboard/scripts/local-smoke.sh --provider claude`
- Result: Claude `expiresAt` parsed as numeric epoch milliseconds; smoke now returns `NORMAL` with populated primary/secondary windows and no token output
- Next step: Commit parser fix and smoke verification, then reassess whether any M1 gaps remain beyond Ubuntu-only platform verification

### 2026-06-11 07:02 UTC
- Agent: main + ui-ux-specialist + platform-specialist + test-specialist
- Task: Start M2 Claude single-widget UI
- Files changed: `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`, `src-tauri/icons/icon.png`, `index.html`, `package.json`, `frontend/src/*`, `frontend/tests/widget.test.mjs`, `scripts/build-frontend.mjs`, `.gitignore`, `roadmap.md`
- Commands run: `npm run build`, `npm test`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added Tauri 2 transparent always-on-top Claude window shell with Linux DMABUF env guard; rendered Claude gauge from mock snapshot; added CI-safe Node DOM/state tests covering 7 logical states, depleted variant, stale badge, lamps, remaining-quota arcs, and Critical-only reduced-motion pulse guard; short GUI smoke ran until timeout without panic after replacing the placeholder icon
- Next step: Perform visual Linux X11 check for transparency/always-on-top/skip-taskbar/drag, then commit M2 checkpoint

### 2026-06-18 08:10 UTC
- Agent: main
- Task: Add persisted grouped/independent widget movement mode and unify drag routing
- Files changed: `src-tauri/src/config.rs`, `src-tauri/src/main.rs`, `frontend/src/main.js`, `frontend/src/settings.js`, `frontend/tests/settings.test.mjs`, `frontend/tests/widget.test.mjs`, `src-tauri/tests/window_contract.rs`, `roadmap.md`
- Commands run: `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Reverted the optional ungrouped mode and fixed the product on a single grouped-layout model; widget dragging still uses the explicit `move_widget_windows` command path, but settings no longer expose a split-mode toggle and config normalization now forces grouped layout for consistent behavior.
- Next step: Run Linux X11 manual verification for grouped drag, widget on/off compaction, first-paint Pomodoro gauge alignment, and absence of transparent-window remnants.

### 2026-06-18 08:40 UTC
- Agent: main
- Task: Finalize Linux X11 rendering stabilization after manual verification
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `RENDERING_REFACTOR.md`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: User verified settings visibility, widget compaction, smooth grouped drag, and no ghosting after widget changes/state changes/movement. Stale opacity/filter effects and the default semi-transparent disk were removed because they could leave rectangular text backing artifacts in transparent WebKitGTK windows. Gauge disks now use opaque `rgb(20, 20, 30)` and stale state remains visible through the update badge.
- Next step: Run one final `git status` review, commit the M3 checkpoint, then start M4 packaging/release work.

### 2026-06-18 09:00 UTC
- Agent: main + security-privacy-specialist + test-specialist
- Task: M3 pre-commit security and test review
- Files changed: `src-tauri/src/config.rs`, `src-tauri/src/dashboard.rs`, `for_specification/poc-result-ubuntu.md`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `git diff --check`
- Result: Added endpoint validation before config persistence, broadened token-material config detection for token-like keys and secret-shaped strings, emitted sanitized runtime warnings for broad token-file permissions, redacted the remaining local Claude credential path in the Ubuntu PoC note, and confirmed there are no CI-safe test gaps blocking the M3 commit.
- Next step: Commit M3, then begin M4 packaging/release preparation.

### 2026-06-18 09:20 UTC
- Agent: main
- Task: Commit M3 checkpoint and prepare M4 handoff
- Files changed: `roadmap.md`
- Commands run: `git commit -m "feat: complete M3 widgets and settings"`, `git status --short`, `git log --oneline --decorate -3`
- Result: M3 was committed as `27fa9e1 feat: complete M3 widgets and settings`. The remaining untracked files are local `.codex/` tooling and local visual/debug reference images that are intentionally not part of the runtime commit.
- Next step: Start M4 packaging and release work: GitHub Actions, Ubuntu deb/AppImage, macOS dmg, README/troubleshooting/release notes, plus platform smoke checks for macOS Keychain and OS notification display.

### 2026-06-18 09:35 UTC
- Agent: main
- Task: Remove Pomodoro paused translucency before M4
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`
- Result: Pomodoro paused state no longer dims the disk, number, arc, or track via opacity changes; it keeps opaque gauge paint to avoid the same Linux transparent WebKit backing-box artifacts already removed from Claude/Codex.
- Next step: Commit this M3 polish fix, then start M4 packaging/release work.

### 2026-06-18 09:50 UTC
- Agent: main
- Task: Stop idle Pomodoro repaint loop in paused state
- Files changed: `frontend/src/main.js`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`
- Result: Confirmed the remaining likely cause was not OS window opacity or stale mapping; Pomodoro stayed `PAUSED` but still called `updatePomodoroWidget()` every 250ms. The periodic repaint now returns immediately while Pomodoro is paused and only continues during running or ending states.
- Next step: Run one Linux X11 visual check for paused Pomodoro remaining opaque over time, then start M4 packaging/release work.

### 2026-06-18 10:05 UTC
- Agent: main
- Task: Prevent stale frontend assets during direct `cargo run`
- Files changed: `src-tauri/build.rs`, `roadmap.md`
- Commands run: `npm run build`, `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Found the visual check was still loading stale `dist` assets because direct `cargo run --manifest-path src-tauri/Cargo.toml` does not execute Tauri CLI `beforeBuildCommand`. The Rust build script now runs `npm run build` when frontend inputs change, so direct Cargo runs use the latest UI assets.
- Next step: Re-run the Linux X11 Pomodoro paused visual check.

### 2026-06-11 07:14 UTC
- Agent: main
- Task: Address M2 visual smoke feedback
- Files changed: `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Removed runtime hover disk/arc styling that made the transparent widget appear opaque after mouseover; added `data-tauri-drag-region="deep"` so Tauri's Linux drag-region handler can move the frameless widget
- Next step: Re-run manual visual check for hover transparency reset and drag movement

### 2026-06-11 07:24 UTC
- Agent: main
- Task: Refine M2 hover and drag behavior after manual check
- Files changed: `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `src-tauri/capabilities/default.json`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, `timeout 5 cargo run --manifest-path src-tauri/Cargo.toml --bin token-dashboard`
- Result: Restored hover visual feedback through `.is-hovered` so it can be explicitly cleared on pointer leave/window blur; added direct `startDragging()` call on left mousedown and granted `core:window:allow-start-dragging`
- Next step: Re-run manual visual check for hover reset and drag movement

### 2026-06-11 07:31 UTC
- Agent: main
- Task: Make hover reset immediate after manual check
- Files changed: `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added mouse leave listeners alongside pointer leave and removed hover background/filter/color transitions, so leaving the widget restores the initial translucent appearance immediately
- Next step: Re-run manual visual check for immediate hover reset

### 2026-06-11 07:36 UTC
- Agent: main
- Task: Remove hover artifacts from transparent window
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Removed hover `box-shadow` and `drop-shadow` effects that were accumulating in the transparent WebKit window; hover now only darkens the disk interior and brightens the number
- Next step: Re-run manual visual check for hover reset without residual shadow

### 2026-06-11 07:41 UTC
- Agent: main
- Task: Disable persistent hover visuals on transparent WebKit
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Removed the remaining `.is-hovered` disk/text styling because even interior background changes persisted visually in the transparent Linux WebKit window; drag behavior remains intact
- Next step: Re-run manual visual check for stable initial translucency while moving the mouse across the widget

### 2026-06-11 07:45 UTC
- Agent: main
- Task: Improve tick mark visibility on light backgrounds
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`
- Result: Increased tick opacity from `.16` to `.34` and stroke width from `1` to `1.35` to keep the gauge texture visible against bright desktop backgrounds
- Next step: Re-run manual visual check on bright and dark backgrounds

### 2026-06-11 07:48 UTC
- Agent: main
- Task: Increase tick mark contrast further
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`
- Result: Increased tick opacity to `.46` and stroke width to `1.5` after manual light-background review
- Next step: Re-run manual visual check on bright and dark backgrounds

### 2026-06-11 07:52 UTC
- Agent: main
- Task: Soften widget disk edge on light backgrounds
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`
- Result: Replaced the hard disk fill and border with a radial gradient that fades the disk edge to transparent without using shadow/filter effects
- Next step: Re-run manual visual check on bright and dark backgrounds

### 2026-06-11 07:57 UTC
- Agent: main
- Task: Make disk edge fade affect backdrop blur
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`
- Result: Replaced the background-only gradient with a radial mask so the disk fill and backdrop blur fade out together at the edge
- Next step: Re-run manual visual check on bright and dark backgrounds

### 2026-06-11 15:02 UTC
- Agent: main + main-planner/ui-ux-specialist/platform-specialist/test-specialist read-only subagents
- Task: Reconcile SPEC, roadmap, and actual M2 implementation state
- Files changed: `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm run build`, `npm test`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`, `timeout 5 cargo run --manifest-path src-tauri/Cargo.toml --bin token-dashboard`, `printenv DISPLAY`, `printenv XDG_SESSION_TYPE`
- Result: Automatic M2/M1 tests pass. Added hover-accessible update age badge while keeping disk/glow hover effects disabled to avoid transparent WebKit artifacts. GUI smoke could not initialize GTK because the current session is `tty` and `DISPLAY` is unset.
- Next step: Run Linux X11 visual verification from a graphical session, then commit the M2 checkpoint.

### 2026-06-11 15:10 UTC
- Agent: main + ui-ux-specialist/test-specialist read-only subagents
- Task: Prepare M3 Codex widget work by generalizing the M2 usage widget renderer only
- Files changed: `frontend/src/widget.js`, `frontend/src/main.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added `renderUsageWidget()` and provider view metadata for Claude/Codex, kept `renderClaudeWidget()` as a compatibility wrapper, added Codex brand CSS tokens, switched the runtime entry to the generic renderer, and expanded Node tests for Codex DOM, provider-neutral state classes, missing secondary window, missing primary window, invalid `fetched_at`, and runtime import binding.
- Git note: This M2 shell plus M3-prep renderer generalization was committed with message `feat: add single widget shell`. `.codex/` remains local tooling config and must stay out of commits.
- Next step: Either run the pending Linux X11 visual verification and commit the M2 checkpoint, or continue with the actual M3 Codex widget using the generic renderer.

### 2026-06-11 15:15 UTC
- Agent: main
- Task: Add Codex widget shell using mock snapshots
- Files changed: `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`, `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added `mock_usage_snapshots()` returning Claude and Codex snapshots, rendered both widgets through `renderUsageDashboard()`, widened the transparent widget window to 340px, added horizontal dashboard layout, and covered dashboard/Codex/Tauri-width behavior in Node tests. No real token files are read and no real APIs are called by this app shell path.
- Git note: This M3 checkpoint was committed with message `feat: add codex widget shell`; `.codex/` remains untracked local tooling config and must stay out of commits.
- Next step: Decide whether to wire the Codex widget to real provider runtime or start Pomodoro.

### 2026-06-13 05:05 UTC
- Agent: main
- Task: Rework gauge tick mark style from screenshot reference
- Files changed: `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Replaced thin radial line ticks with short rounded block ticks, added larger brand-tinted major ticks every 6 marks, and switched tick styling from stroke-based to fill-based to better match the provided gauge screenshot.
- Next step: Re-run manual visual check on bright and dark backgrounds, focusing on whether the new tick marks feel less busy and more integrated with the gauge.

### 2026-06-13 05:12 UTC
- Agent: main
- Task: Improve major tick mark contrast
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`
- Result: Changed every-45-degree major ticks from `color-mix()` to direct provider gauge color `var(--arc)` with `.82` opacity to avoid black/low-contrast rendering in WebKit.
- Next step: Re-run manual visual check for major tick visibility on bright and dark backgrounds.

### 2026-06-13 05:17 UTC
- Agent: main
- Task: Fix major tick color not changing in the app
- Files changed: `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`
- Result: Replaced `var(--arc)` major tick indirection with provider-specific `.widget.claude/.widget.codex` fill rules and moved ticks after rings in SVG draw order so major ticks render above the gauge/track.
- Next step: Re-run manual visual check for orange Claude and teal Codex major ticks.

### 2026-06-13 05:21 UTC
- Agent: main
- Task: Make major ticks follow state gauge color
- Files changed: `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`
- Result: Changed major tick fill to `var(--arc)` so the every-45-degree ticks follow provider color in NORMAL, caution color in WARN/low, and danger color in CRITICAL/depleted states.
- Next step: Re-run manual visual check for NORMAL/WARN/CRITICAL major tick colors.

### 2026-06-13 05:28 UTC
- Agent: main
- Task: Close M2 visual verification and tick polish
- Files changed: `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: User confirmed Linux X11 M2 visual checks passed, including transparent background, disk edge, bright-background tick visibility, frameless/always-on-top/skip-taskbar behavior, drag movement, and hover update badge without residual disk/shadow artifacts. Screenshot-style block ticks with state-colored major marks were accepted.
- Next step: Commit the visual polish, then continue M3 by replacing mock dashboard snapshots with real provider snapshots while keeping token reads local and non-persistent.

### 2026-06-13 05:35 UTC
- Agent: main + main-planner/security-privacy-specialist/test-specialist read-only subagents
- Task: Start M3 real provider snapshot bridge
- Files changed: `src-tauri/src/dashboard.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `src-tauri/src/token_source.rs`, `src-tauri/src/config.rs`, `src-tauri/src/providers/codex.rs`, `src-tauri/tests/m1_contract.rs`, `frontend/src/main.js`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npm test`, `npm run build`
- Result: Replaced `mock_usage_snapshots` with `usage_snapshots`, added a dashboard runtime that reads CLI-owned Claude/Codex token sources, uses existing fixture-tested provider refresh paths with memory-only cache, degrades missing token sources to `NOT_LOGGED_IN`, and exposes only frontend-safe snapshot fields. Tightened Codex token-bearing endpoint allowlist to the confirmed `https://chatgpt.com/backend-api/wham/usage`.
- Next step: Run Linux desktop visual check for the real provider bridge; expected result is two widgets populated from local providers or safe degraded states without exposing secrets. If visual check passes, continue M3 with Pomodoro widget/runtime.

### 2026-06-13 05:55 UTC
- Agent: main
- Task: Add first Pomodoro widget slice
- Files changed: `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `src-tauri/tauri.conf.json`, `index.html`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added a third Pomodoro widget with frontend-local focus timer state, focus/break/paused rendering classes, Pomodoro color tokens, one-number minute display, no secondary ring, and a wider transparent widget window. Added DOM tests that verify Pomodoro still renders when Claude/Codex are stale/auth degraded.
- Next step: Run Linux X11 visual check for the three-widget layout. Then add Pomodoro controls, phase switching, settings persistence, and notification command integration.

### 2026-06-13 06:05 UTC
- Agent: main
- Task: Address Pomodoro visual check feedback and diagnose Claude disconnected state
- Files changed: `frontend/src/main.js`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `TOKEN_DASHBOARD_ALLOW_REAL_API=1 ./scripts/local-smoke.sh --provider claude`, token-free `curl -I` and GET probes to `https://api.anthropic.com/api/oauth/usage`
- Result: User confirmed the three widgets, Pomodoro focus display, and window behavior. Reduced full dashboard rerender from 1s to 60s to avoid transparent WebKit opacity accumulation. Claude local smoke returned `STALE` with network failure; token-free endpoint probes reached Cloudflare but returned `429` with `retry-after` around one hour, so the current Claude disconnect is not caused by CLI auth file changes.
- Next step: Re-run visual check for opacity stability after the 60s repaint change. Re-check Claude after the 429 retry window or add safer status reason DTO if the UI needs to distinguish RATE_LIMITED from generic stale.

### 2026-06-15 09:35 UTC
- Agent: main
- Task: Diagnose persistent Claude usage absence and Codex yellow/WARN color
- Files changed: `src-tauri/src/refresh.rs`, `roadmap.md`
- Commands run: `TOKEN_DASHBOARD_ALLOW_REAL_API=1 ./scripts/local-smoke.sh --provider claude`, sanitized file existence/schema checks for `~/.claude/.credentials.json`, token-free endpoint probe to `https://api.anthropic.com/api/oauth/usage`, token-status-only Claude usage and refresh probes, `cargo test --manifest-path src-tauri/Cargo.toml`, `TOKEN_DASHBOARD_ALLOW_REAL_API=1 ./scripts/local-smoke.sh --provider codex`
- Result: Claude credential file exists with mode `600` and expected access/refresh fields. Token-status-only probes showed usage access token returns `401`, refresh returns `400 invalid_grant`; this means the saved Claude session must be re-authenticated and should not require recurring login once a valid refresh token is restored. Updated refresh failure mapping so `400 invalid_grant` becomes `AUTH_ERROR`. Codex smoke returned `WARN` with primary 9% and secondary 80%, so the yellow color is expected.
- Next step: User should run `claude auth logout` then `claude auth login --claudeai` or the appropriate `--console`/`--sso` variant, then run local Claude smoke again. Continue M3 with Pomodoro controls, phase switching, settings persistence, and notification command integration after auth is restored or accepted as an external account state.

### 2026-06-15 09:45 UTC
- Agent: main + main-planner/ui-ux-specialist/test-specialist read-only subagents
- Task: Add Pomodoro controls and frontend-local phase switching
- Files changed: `frontend/src/pomodoro.js`, `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/pomodoro.test.mjs`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Extracted Pomodoro state machine into a provider/network-independent module; added pause/resume, reset, skip, focus/break auto-rollover, and hover/focus-only controls that opt out of window drag. Added CI-safe tests for pause/resume/reset/skip/rollover and Pomodoro isolation from Tauri/provider APIs.
- Next step: Run Linux X11 visual check for Pomodoro controls, focusing on hover-only toolbar visibility, button clicks not starting window drag, reset/skip/toggle behavior, and no transparent-window repaint artifacts. Then start config persistence for Pomodoro durations and widget settings.

### 2026-06-15 10:00 UTC
- Agent: main
- Task: Address Pomodoro control visual feedback
- Files changed: `frontend/src/main.js`, `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Pomodoro button actions now replace only the Pomodoro widget instead of redrawing the full dashboard, reducing transparent WebKit overlap artifacts. Paused state is visually stronger through dimmed ring/text and highlighted resume button. Pomodoro alone uses 12 major tick marks while Claude/Codex keep 8.
- Next step: Run Linux X11 visual check for Pomodoro button actions, paused-state readability, and Pomodoro-only 12 major ticks.

### 2026-06-16 00:00 UTC
- Agent: main + subagents
- Task: Fix Pomodoro button interaction and continuous timing
- Files changed: `frontend/src/main.js`, `frontend/src/pomodoro.js`, `frontend/src/styles.css`, `frontend/tests/pomodoro.test.mjs`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `git diff -- frontend/src/main.js frontend/src/pomodoro.js frontend/src/styles.css frontend/tests/pomodoro.test.mjs frontend/tests/widget.test.mjs roadmap.md`
- Result: Pomodoro controls now stay inside the hover region, button clicks are handled from the app root, the timer redraws on a sub-second cadence, and gauge progress is based on the active phase duration instead of a fixed dial.
- Next step: Run the app and confirm the three Pomodoro buttons work and the ring shrinks smoothly for short durations.

### 2026-06-16 00:00 UTC
- Agent: main
- Task: Add Pomodoro end-state blink and paused handoff
- Files changed: `frontend/src/main.js`, `frontend/src/pomodoro.js`, `frontend/src/widget.js`, `frontend/src/styles.css`, `frontend/tests/pomodoro.test.mjs`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Completed timers now blink for 30 seconds, clicking the widget acknowledges the end state immediately, the next phase stays paused until the user starts it, and Pomodoro controls are now a fixed opaque action row below the gauge rather than a hover overlay.
- Next step: Run the app and visually verify the blinking end state, the below-gauge button placement, and the paused next-phase handoff.

### 2026-06-16 01:00 UTC
- Agent: main + main-planner read-only subagent
- Task: Add M3 settings window and config persistence foundation
- Files changed: `src-tauri/src/config.rs`, `src-tauri/src/dashboard.rs`, `src-tauri/src/main.rs`, `src-tauri/capabilities/default.json`, `settings.html`, `scripts/build-frontend.mjs`, `frontend/src/settings.js`, `frontend/src/settings.css`, `frontend/tests/settings.test.mjs`, `frontend/src/main.js`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added a decorated opaque settings window at `settings.html`, opened from the widget context menu via `open_settings_window`, added `get_app_settings`/`save_app_settings`, persisted M3 config fields without token material, normalized partial config files, and updated provider runtime endpoints after settings saves.
- Next step: Implement notification thresholds using mock-safe tests, then add click-through only after confirming there is a reliable settings recovery path.

### 2026-06-16 01:15 UTC
- Agent: main + test-specialist read-only subagent
- Task: Add notification threshold logic
- Files changed: `frontend/src/notifications.js`, `frontend/tests/notifications.test.mjs`, `frontend/src/main.js`, `roadmap.md`
- Commands run: `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added CI-safe threshold event logic for Claude/Codex snapshots, defaulted notifications to `[80, 95]`, deduplicated events by provider/window/threshold/reset, rearmed on `resets_at` changes, ignored degraded snapshots, and dispatched through a thin Notification API wrapper without reading tokens or calling real providers in tests.
- Next step: Add click-through only with a settings recovery path; then add autostart and final Pomodoro isolation verification.

### 2026-06-16 01:30 UTC
- Agent: main
- Task: Add click-through and autostart
- Files changed: `src-tauri/src/autostart.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `roadmap.md`
- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml`, `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Added click-through application through Tauri window cursor-event ignoring, opens settings automatically at startup when click-through is enabled as a recovery path, and added OS-native autostart persistence using Linux XDG desktop entries and macOS LaunchAgent plists. Autostart tests use temp paths only and do not touch the real user config.
- Next step: Run Linux X11 manual verification for settings window, click-through recovery, autostart file creation/removal, notification display, and unchanged transparent widget behavior.

### 2026-06-17 00:00 UTC
- Agent: main
- Task: Address settings manual verification feedback
- Files changed: `frontend/src/main.js`, `frontend/src/pomodoro.js`, `frontend/src/settings.js`, `frontend/src/settings.css`, `frontend/tests/pomodoro.test.mjs`, `frontend/tests/settings.test.mjs`, `frontend/tests/widget.test.mjs`, `src-tauri/src/main.rs`, `roadmap.md`
- Commands run: `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `npm run build`
- Result: Settings saves now show visible saving/saved states, the dashboard re-reads settings and applies widget visibility plus Pomodoro focus/break durations within one second, and the settings window refuses to close while click-through is enabled so the user cannot lose the recovery path.
- Next step: Re-run Linux X11 manual verification for save feedback, widget checkbox application, Pomodoro duration application, and click-through recovery behavior.

### 2026-06-17 00:15 UTC
- Agent: main
- Task: Move click-through to experimental and stop rerendering the dashboard on settings changes
- Files changed: `frontend/src/main.js`, `frontend/src/settings.js`, `frontend/src/settings.css`, `frontend/tests/settings.test.mjs`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Widget visibility now toggles in-place via `is-hidden` instead of replacing the root dashboard HTML, Pomodoro durations update from persisted settings without spawning a new widget layer, and click-through is shown under an Experimental section in the settings window.
- Next step: Re-run the Linux X11 smoke once, then commit M3 after confirming the new in-place behavior visually.

### 2026-06-17 00:30 UTC
- Agent: main
- Task: Remount the dashboard root on settings changes and make click-through explicitly experimental
- Files changed: `frontend/src/main.js`, `frontend/src/settings.js`, `frontend/tests/settings.test.mjs`, `frontend/tests/widget.test.mjs`, `roadmap.md`
- Commands run: `node --test frontend/tests/settings.test.mjs frontend/tests/widget.test.mjs`, `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: Settings saves now replace the dashboard root node instead of mutating the existing tree, which avoids stale widget layers in transparent WebKit; the settings window now labels click-through as an experimental feature, and tests pass.
- Next step: Ask for a fresh Linux visual check on widget-checkbox save behavior and confirm the old dashboard layer no longer remains below the new one.

## Known Issues
| Issue | Severity | Status | Next Action |
|---|---|---|---|
| Codex direct usage endpoint path is not confirmed from PoC | High | Resolved by source analysis | Default to `/backend-api/wham/usage`; still requires local-only real API smoke |
| Codex refresh host/client metadata is not confirmed | High | Resolved by source analysis | `auth.openai.com/oauth/token` and client metadata from codex-check 1.3.13; refresh is memory-only and fixture-tested |
| Claude refresh host/client metadata is undocumented | High | Resolved by local CLI static analysis | `platform.claude.com/v1/oauth/token` and client metadata from installed Claude CLI 2.1.170; refresh is memory-only and fixture-tested |
| Claude local smoke returns AUTH_ERROR | Low | Resolved | Fixed numeric `expiresAt` parsing; smoke now returns `NORMAL` |
| Codex 20-poll smoke is complete | Low | Done | Recorded as WARN throughout; no 429 observed in the captured run |
| macOS Keychain cannot be verified on current Ubuntu environment | Medium | Open | Security CLI fallback exists; perform manual macOS Keychain verification during M4 packaging validation. |
| Real provider bridge Linux visual check | Medium | Resolved | User confirmed the Tauri widget now renders real provider values after enabling `withGlobalTauri`; mock fallback no longer masks runtime invoke failures. |
| Pomodoro visual verification | Medium | Resolved | User confirmed three-widget layout, grouped drag behavior, widget on/off compaction, and stable opacity after the per-widget window split. |
| Claude local credential refresh is invalid | Medium | Resolved | User confirmed Claude works after re-authentication; recurring login should not be required while the new refresh token remains valid. |
| Codex widget is yellow/WARN | Low | Expected | Codex primary 5-hour usage is low, but secondary 7-day usage is 80%; state machine uses the maximum usage window, so WARN/yellow is correct. |
| Pomodoro controls visual verification | Medium | Resolved | Tests cover reset/skip/toggle, minute editing, end-state acknowledgement, and provider isolation; user confirmed the current grouped window model is visually stable on Linux X11. |
| Click-through can block settings access | High | Mitigated | When enabled, widget right-click, drag, and Pomodoro buttons are intentionally unavailable; startup opens the settings window automatically and close is prevented while click-through is enabled. |
| M3 settings runtime application is partial | Medium | Resolved | Settings persist and endpoints update immediately; notification thresholds are evaluated on provider polling; click-through/autostart/widget toggles/Pomodoro durations apply after save; widget scale is applied through the runtime settings signature. |
| OS notification display is not manually verified | Medium | Open | Threshold logic and dispatch are tested with a fake Notification API; run a platform smoke during M4 after forcing a mock threshold or reaching a real threshold. |
| M2 Linux X11 visual verification | High | Resolved | User confirmed transparency, layout, tick visibility, frameless/always-on-top/skip-taskbar, drag, and hover update badge behavior on Linux X11. |
| M2/M3 visual tuning intentionally diverges from design-reference token literals | Medium | Resolved | Accepted screenshot-style block ticks, stronger bright-background tick contrast, opaque disks, fixed Pomodoro controls, and disabled hover disk/glow/stale opacity effects as implementation decisions for Linux transparent WebKit. |

## Resume Instructions
If the session is interrupted, resume by:
1. Read this roadmap.md.
2. Read SPEC.md.
3. Check git status.
4. Run the last relevant test command.
5. Continue from "Next recommended command".
