# Troubleshooting

## The app does not show Claude or Codex usage

Confirm the corresponding CLI is logged in first.

Claude Code on Ubuntu:

```bash
ls -l ~/.claude/.credentials.json
```

Codex CLI on Ubuntu:

```bash
ls -l ~/.codex/auth.json
```

Do not print these files. They may contain tokens.

If Claude shows an auth problem after a long idle period, reauthenticate in Claude Code and restart Token Dashboard.

Codex must be logged in with ChatGPT auth mode. If Codex shows `NOT_LOGGED_IN`, re-run the Codex CLI login flow and confirm the auth file exists without printing its contents.

## AUTH_ERROR, STALE, or RATE_LIMITED

- `AUTH_ERROR`: the owning CLI credential is present but cannot currently authenticate. Re-run that CLI's login flow.
- `STALE`: the app kept the last safe status because the provider could not return a fresh valid snapshot.
- `RATE_LIMITED`: the provider is temporarily rejecting requests. Wait for the retry window.

Provider failures should not break Pomodoro.

## Ubuntu token-file permission warning

Claude and Codex token files should not be readable by other users:

```bash
chmod 600 ~/.claude/.credentials.json
chmod 600 ~/.codex/auth.json
```

## Ubuntu widget does not render correctly

Ubuntu 22.04 X11 is the supported Linux target. Wayland is not a v1 target.

The app sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` automatically on Linux to avoid known WebKitGTK transparent-window rendering issues. If artifacts still appear, confirm you are running an X11 session:

```bash
echo "$XDG_SESSION_TYPE"
```

Expected:

```text
x11
```

## Ubuntu AppImage does not launch

Install FUSE compatibility if your distribution does not include it:

```bash
sudo apt install libfuse2
```

## macOS says the app is damaged or from an unidentified developer

The v0.1.0 artifacts are unsigned. Use one of these options:

1. Right-click the app and choose Open.
2. Or remove quarantine from the downloaded app:

```bash
xattr -dr com.apple.quarantine "/Applications/Token Dashboard.app"
```

macOS may also ask for Keychain access the first time the app reads Claude Code credentials.

## Settings cannot be found

Right-click any widget to open Settings. Use the Settings `Quit` button to exit the app.

If autostart is enabled, Linux uses an XDG autostart `.desktop` entry and macOS uses a LaunchAgent plist.

If no widget is visible, terminate the process from a terminal and restart:

```bash
pgrep -af token-dashboard
```

Then kill the matching process ID only if it is the Token Dashboard app.

## Local smoke tests

Smoke tests can call real usage endpoints and must be run locally only:

```bash
TOKEN_DASHBOARD_ALLOW_REAL_API=1 ./scripts/local-smoke.sh --provider all
```

The smoke script refuses to run in CI and should only print normalized, sanitized snapshots.
