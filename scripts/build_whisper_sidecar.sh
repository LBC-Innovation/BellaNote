#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="${1:-${CARGO_BUILD_TARGET:-}}"
if [[ -z "$target" ]]; then
  target="$(rustc -vV | awk '/^host:/{print $2}')"
fi

out_dir="$root/src-tauri/binaries"
out="$out_dir/transcribe-worker-$target"
script="$root/scripts/transcribe_worker.py"
spec="$root/scripts/transcribe_worker.spec"
req="$root/requirements-sidecar.txt"
work="$root/src-tauri/.cache/pyinstaller"

mkdir -p "$out_dir" "$work"

if [[ -f "$out" && "$(wc -c < "$out")" -gt 1000000 && "$out" -nt "$script" && "$out" -nt "$spec" && "$out" -nt "$req" && "$out" -nt "$root/requirements.txt" ]]; then
  echo "whisper sidecar is up to date: $out"
  exit 0
fi

python="${ECHO_PYTHON:-$root/.venv/bin/python3}"
if [[ ! -x "$python" ]]; then
  python="$(command -v python3)"
fi

"$python" -m pip install -q -r "$req"
"$python" -m PyInstaller \
  --noconfirm \
  --clean \
  --workpath "$work" \
  --distpath "$work/dist" \
  "$spec"

mkdir -p "$out_dir"
cp "$work/dist/transcribe-worker" "$out"
chmod +x "$out"
echo "wrote $out"
