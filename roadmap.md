# Token Dashboard Roadmap

## Current Status
- Current milestone: M1 — Data Layer
- Current task: Push current safe M1 checkpoints
- Last completed task: Committed config guardrails checkpoint
- Last command run: `cargo test --manifest-path src-tauri/Cargo.toml`
- Last test result: Passed — 41 tests
- Next recommended command: `git push -u origin main`
- Blocking issue: Codex direct usage endpoint/path and refresh host remain unverified; implement overrideable structure and local-only smoke script first
- Updated at: 2026-06-11 00:00 KST

## Source Documents Read
- [x] SPEC.md
- [x] for_specification PoC files
- [x] design reference files

## Decisions
| Date | Decision | Reason | Files/Sections |
|---|---|---|---|
| 2026-06-11 | Start with a GUI-free Rust data-layer crate under `src-tauri` | M1 requires provider parsing/state logic before UI; Tauri can attach to the same crate in M2 | SPEC.md §§3-5, §11 |
| 2026-06-11 | Keep Codex usage endpoint path configurable and do not hardcode unverified refresh behavior | PoC used `codex-check`; direct path/refresh host are not confirmed | SPEC.md §§2.4, 4.3, R-3 |
| 2026-06-11 | Automatic tests use fixtures only; real API checks are local-only scripts | CI must not call real APIs or print auth material | SPEC.md §§2, 10 |

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
- [ ] Implement refresh/cache policy where safe
- [x] Implement state machine
- [x] Implement backoff
- [x] Implement token masking
- [x] Add fixture tests
- [x] Add local-only smoke script
- [ ] Run tests
- [ ] Commit M1

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

## Known Issues
| Issue | Severity | Status | Next Action |
|---|---|---|---|
| Codex direct usage endpoint path is not confirmed from PoC | High | Open | Keep endpoint override, parser fixtures, and local-only smoke script; investigate source references if added |
| Codex/Claude refresh host allowlist is not confirmed | High | Open | Do not persist refreshed tokens; defer real refresh calls until host is documented |
| macOS Keychain cannot be verified on current Ubuntu environment | Medium | Open | Implement macOS-gated source with `security` fallback and document manual verification |
| 20-poll no-429 run is not completed | Medium | Open | Add local-only script/checklist; do not run in CI |

## Resume Instructions
If the session is interrupted, resume by:
1. Read this roadmap.md.
2. Read SPEC.md.
3. Check git status.
4. Run the last relevant test command.
5. Continue from "Next recommended command".
