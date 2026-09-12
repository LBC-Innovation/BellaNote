#!/usr/bin/env bash
# Re-sign transcribe-worker with sidecar entitlements, then re-seal the .app
# without --deep so the main binary keeps mic/screen entitlements only.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
app="${1:-}"
if [[ -z "$app" || ! -d "$app" ]]; then
  echo "usage: $0 /path/to/BellaNote.app" >&2
  exit 1
fi
app="$(cd "$app" && pwd)"

worker="$app/Contents/MacOS/transcribe-worker"
main="$app/Contents/MacOS/bellanote"
sidecar_ent="$root/src-tauri/entitlements-sidecar.plist"
app_ent="$root/src-tauri/entitlements.plist"

if [[ ! -f "$worker" ]]; then
  echo "missing sidecar: $worker" >&2
  exit 1
fi
if [[ ! -f "$main" ]]; then
  echo "missing app binary: $main" >&2
  exit 1
fi

identity="${APPLE_SIGNING_IDENTITY:-}"
if [[ -z "$identity" ]]; then
  identity="-"
  echo "No APPLE_SIGNING_IDENTITY; ad-hoc signing the sidecar with entitlements."
fi

# --timestamp must not sit between --entitlements and its path, or codesign
# treats "--timestamp" as the entitlements file ("cannot read entitlement data").
sign=(codesign --force --options runtime)
if [[ "$identity" != "-" ]]; then
  sign+=(--timestamp)
fi

"${sign[@]}" --entitlements "$sidecar_ent" --sign "$identity" "$worker"
"${sign[@]}" --entitlements "$app_ent" --sign "$identity" "$app"

worker_ent="$(codesign -d --entitlements - "$worker" 2>&1)"
main_ent="$(codesign -d --entitlements - "$main" 2>&1)"

echo "$worker_ent" | grep -q "com.apple.security.cs.disable-library-validation" || {
  echo "sidecar is missing disable-library-validation" >&2
  echo "$worker_ent" >&2
  exit 1
}
if echo "$worker_ent" | grep -Eq "com.apple.security.device.microphone|com.apple.security.device.audio-input|com.apple.security.screen-capture"; then
  echo "sidecar still has capture entitlements; expected a worker-only plist" >&2
  echo "$worker_ent" >&2
  exit 1
fi
if echo "$main_ent" | grep -q "com.apple.security.cs.disable-library-validation"; then
  echo "main app must not have disable-library-validation" >&2
  echo "$main_ent" >&2
  exit 1
fi
if echo "$main_ent" | grep -Eq "allow-unsigned-executable-memory|allow-jit|allow-dyld-environment-variables"; then
  echo "main app must not have RWX/JIT/DYLD entitlements" >&2
  echo "$main_ent" >&2
  exit 1
fi

echo "sidecar entitlements ok: $worker"
echo "app entitlements ok: $main"
