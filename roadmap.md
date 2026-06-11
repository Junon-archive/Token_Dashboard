# Token Dashboard Roadmap

## Current Status
- Current milestone: M2 — Single Widget UI
- Current task: Prepare M2 implementation start
- Last completed task: Closed out M1 after verifying Claude/Codex smoke and parsing fix
- Last command run: `TOKEN_DASHBOARD_ALLOW_REAL_API=1 /home/junon/Token_Dashboard/scripts/local-smoke.sh --provider claude`
- Last test result: Passed — 49 tests; Claude smoke returned NORMAL, Codex remained verified
- Next recommended command: `git status`
- Blocking issue: macOS Keychain Security framework first path is not implemented on Ubuntu, but M1 is complete
- Updated at: 2026-06-11 05:47 UTC

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
- [ ] Implement transparent Tauri window
- [ ] Implement Claude widget rendering
- [ ] Implement design tokens
- [ ] Implement 7 logical state visual mapping
- [ ] Add mock provider UI tests
- [ ] Verify Linux X11 behavior where possible
- [ ] Commit M2

### M3 — Three Widgets and Settings
- [ ] Add Codex widget
- [ ] Add Pomodoro widget
- [ ] Add settings window
- [ ] Add config persistence
- [ ] Add notification thresholds
- [ ] Add click-through
- [ ] Add autostart
- [ ] Add Pomodoro isolation test
- [ ] Commit M3

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

## Known Issues
| Issue | Severity | Status | Next Action |
|---|---|---|---|
| Codex direct usage endpoint path is not confirmed from PoC | High | Resolved by source analysis | Default to `/backend-api/wham/usage`; still requires local-only real API smoke |
| Codex refresh host/client metadata is not confirmed | High | Resolved by source analysis | `auth.openai.com/oauth/token` and client metadata from codex-check 1.3.13; refresh is memory-only and fixture-tested |
| Claude refresh host/client metadata is undocumented | High | Resolved by local CLI static analysis | `platform.claude.com/v1/oauth/token` and client metadata from installed Claude CLI 2.1.170; refresh is memory-only and fixture-tested |
| Claude local smoke returns AUTH_ERROR | Low | Resolved | Fixed numeric `expiresAt` parsing; smoke now returns `NORMAL` |
| Codex 20-poll smoke is complete | Low | Done | Recorded as WARN throughout; no 429 observed in the captured run |
| macOS Keychain cannot be verified on current Ubuntu environment | Medium | Open | Implement macOS-gated source with `security` fallback and document manual verification |
| 20-poll no-429 run is not completed | Medium | Open | Add local-only script/checklist; do not run in CI |

## Resume Instructions
If the session is interrupted, resume by:
1. Read this roadmap.md.
2. Read SPEC.md.
3. Check git status.
4. Run the last relevant test command.
5. Continue from "Next recommended command".
