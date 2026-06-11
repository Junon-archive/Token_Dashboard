#!/usr/bin/env bash
set -euo pipefail

if [[ "${TOKEN_DASHBOARD_ALLOW_REAL_API:-}" != "1" ]]; then
  echo "Refusing to run real API smoke test. Set TOKEN_DASHBOARD_ALLOW_REAL_API=1 locally."
  exit 2
fi

if [[ "${CI:-}" == "true" ]]; then
  echo "Refusing to run real API smoke test in CI."
  exit 2
fi

echo "Local-only smoke placeholder."
echo "This script prints only normalized snapshots and sanitized status fields."
echo "Do not print auth files, Authorization headers, access tokens, refresh tokens, ID tokens, or API keys."

cargo run --manifest-path src-tauri/Cargo.toml --bin local_smoke -- "$@"
