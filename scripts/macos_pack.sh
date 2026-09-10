#!/usr/bin/env bash
# Local Mac packaging: Tauri bundle, then sidecar entitlements, then a smoke start.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [[ "$(uname -s)" != "Darwin" ]]; then
  exec npx tauri build --bundles dmg,app "$@"
fi

npx tauri build --bundles dmg,app "$@"

shopt -s nullglob
apps=(
  "$root"/src-tauri/target/aarch64-apple-darwin/release/bundle/macos/*.app
  "$root"/src-tauri/target/release/bundle/macos/*.app
)
if [[ ${#apps[@]} -eq 0 ]]; then
  echo "No .app bundle found after tauri build" >&2
  exit 1
fi
app="${apps[0]}"
bash "$root/scripts/macos_resign_sidecar.sh" "$app"
bash "$root/scripts/smoke_transcribe_worker.sh" "$app"
echo "signed sidecar smoke passed: $app"
