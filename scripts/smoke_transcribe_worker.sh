#!/usr/bin/env bash
# Fail if the signed sidecar cannot load its Python runtime (Team ID mismatch).
# Does not wait for the Whisper model to download.
set -euo pipefail

app="${1:-}"
if [[ -z "$app" || ! -d "$app" ]]; then
  echo "usage: $0 /path/to/BellaNote.app" >&2
  exit 1
fi
app="$(cd "$app" && pwd)"
worker="$app/Contents/MacOS/transcribe-worker"
if [[ ! -x "$worker" ]]; then
  echo "missing executable sidecar: $worker" >&2
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
out="$tmp/stdout.txt"
err="$tmp/stderr.txt"
download_root="$tmp/whisper-models"
# Keep Hub caches inside tmp so a partial model download does not fill the runner home.
export HF_HOME="$tmp/hf"
export HUGGINGFACE_HUB_CACHE="$tmp/hf/hub"
export TRANSFORMERS_CACHE="$tmp/hf/transformers"
mkdir -p "$download_root" "$HUGGINGFACE_HUB_CACHE" "$TRANSFORMERS_CACHE"
touch "$out" "$err"

set +e
"$worker" --model small.en --download-root "$download_root" >"$out" 2>"$err" &
pid=$!
sleep 4
kill "$pid" 2>/dev/null
wait "$pid" 2>/dev/null
set -e

if grep -Eq "different Team IDs|Failed to load Python" "$err"; then
  echo "transcribe-worker failed to load Python:" >&2
  cat "$err" >&2
  exit 1
fi

if grep -Eq "different Team IDs|Failed to load Python" "$out"; then
  echo "transcribe-worker failed to load Python (stdout):" >&2
  cat "$out" >&2
  exit 1
fi

echo "transcribe-worker started without a Python Team ID crash"
echo "stdout:"; cat "$out" || true
echo "stderr:"; cat "$err" || true
