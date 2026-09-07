#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="${1:-${CARGO_BUILD_TARGET:-}}"
if [[ -z "$target" ]]; then
  target="$(rustc -vV | awk '/^host:/{print $2}')"
fi

out_dir="$root/src-tauri/binaries"
out="$out_dir/transcribe-worker-$target"
if [[ "$target" == *windows* ]]; then
  out="${out}.exe"
fi
script="$root/scripts/transcribe_worker.py"
spec="$root/scripts/transcribe_worker.spec"
req="$root/requirements-sidecar.txt"
work="$root/src-tauri/.cache/pyinstaller"

mkdir -p "$out_dir" "$work"

if [[ -f "$out" && "$(wc -c < "$out")" -gt 1000000 && "$out" -nt "$script" && "$out" -nt "$spec" && "$out" -nt "$req" && "$out" -nt "$root/requirements.txt" ]]; then
  echo "whisper sidecar is up to date: $out"
  exit 0
fi

python="${ECHO_PYTHON:-}"
if [[ -z "$python" ]]; then
  if [[ -x "$root/.venv/bin/python3" ]]; then
    python="$root/.venv/bin/python3"
  elif [[ -f "$root/.venv/Scripts/python.exe" ]]; then
    python="$root/.venv/Scripts/python.exe"
  elif command -v python3 >/dev/null 2>&1; then
    python="$(command -v python3)"
  else
    python="$(command -v python)"
  fi
fi

"$python" -m pip install -q -r "$req"
"$python" -m PyInstaller \
  --noconfirm \
  --clean \
  --workpath "$work" \
  --distpath "$work/dist" \
  "$spec"

src="$work/dist/transcribe-worker"
if [[ -f "${src}.exe" ]]; then
  src="${src}.exe"
fi
if [[ ! -f "$src" ]]; then
  echo "PyInstaller did not write $src" >&2
  exit 1
fi

mkdir -p "$out_dir"
cp "$src" "$out"
if [[ "$out" != *.exe ]]; then
  chmod +x "$out"
fi
echo "wrote $out"
