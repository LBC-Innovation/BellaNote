#!/usr/bin/env bash
# After Tauri signs the .app (without notarizing): resign sidecar, smoke,
# rebuild the DMG from the resigned app, notarize, staple, upload.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="${1:-aarch64-apple-darwin}"
bundle="$root/src-tauri/target/$target/release/bundle"
macos_dir="$bundle/macos"
dmg_dir="$bundle/dmg"

shopt -s nullglob
apps=("$macos_dir"/*.app)
if [[ ${#apps[@]} -eq 0 ]]; then
  echo "No .app under $macos_dir" >&2
  exit 1
fi
app="${apps[0]}"

bash "$root/scripts/macos_import_signing_cert.sh"
bash "$root/scripts/macos_resign_sidecar.sh" "$app"
bash "$root/scripts/smoke_transcribe_worker.sh" "$app"

version="$(python3 -c "import json; print(json.load(open('$root/src-tauri/tauri.conf.json'))['version'])")"
volname="$(python3 -c "import json; print(json.load(open('$root/src-tauri/tauri.conf.json'))['productName'])")"
dmg="$dmg_dir/${volname}_${version}_aarch64.dmg"
mkdir -p "$dmg_dir"

# Host disk is usually fine; reclaim still drops large intermediates we no longer need.
echo "Disk before reclaim:"
df -h "$root" "$TMPDIR" 2>/dev/null || df -h
release_dir="$root/src-tauri/target/$target/release"
rm -rf \
  "$release_dir/deps" \
  "$release_dir/build" \
  "$release_dir/incremental" \
  "$release_dir/.fingerprint" \
  "$release_dir/bellanote" \
  "$release_dir/bellanote.d" \
  "$macos_dir"/*.app.tar.gz \
  "$root/src-tauri/.cache" \
  "$root/src-tauri/binaries" \
  "$root/node_modules" \
  "${HOME}/.cache/huggingface" \
  "${HOME}/Library/Caches/huggingface" \
  "${CARGO_HOME:-$HOME/.cargo}/registry/src" \
  "${CARGO_HOME:-$HOME/.cargo}/git/checkouts"
echo "Disk after reclaim:"
df -h "$root" "$TMPDIR" 2>/dev/null || df -h

# hdiutil's default -srcfolder auto-size often undersizes a large .app (Whisper
# sidecar). That fails as "No space left on device" on /Volumes/$volname even
# when the host still has tens of GB free. Create an explicitly sized RW image,
# copy with ditto, then convert to compressed UDZO.
if [[ -d "/Volumes/$volname" ]]; then
  hdiutil detach "/Volumes/$volname" -force || true
fi

app_kb="$(du -sk "$app" | awk '{print $1}')"
# 30% headroom + 64MB for HFS+ overhead / catalog.
size_kb="$((app_kb * 13 / 10 + 64 * 1024))"
rw_dmg="${TMPDIR:-/tmp}/bellanote-rw-$$.dmg"
rm -f "$rw_dmg" "$dmg"

echo "Creating DMG: app=$(du -sh "$app" | awk '{print $1}') rw_image=$((size_kb / 1024))m"
hdiutil create -ov -size "${size_kb}k" -fs HFS+ -volname "$volname" "$rw_dmg"
attach_out="$(hdiutil attach -readwrite -noverify -noautoopen "$rw_dmg")"
echo "$attach_out"
device="$(echo "$attach_out" | awk '/^\/dev\// {dev=$1} END {print dev}')"
volume="/Volumes/$volname"
if [[ -z "$device" || ! -d "$volume" ]]; then
  echo "Failed to mount $rw_dmg" >&2
  exit 1
fi

cleanup_dmg() {
  hdiutil detach "$device" -force 2>/dev/null || true
  rm -f "$rw_dmg"
}
trap cleanup_dmg EXIT

ditto "$app" "$volume/$(basename "$app")"
ln -s /Applications "$volume/Applications"
sync
hdiutil detach "$device"
device=""
trap - EXIT

hdiutil convert "$rw_dmg" -format UDZO -imagekey zlib-level=9 -o "$dmg"
rm -f "$rw_dmg"


if [[ -z "${APPLE_ID:-}" || -z "${APPLE_PASSWORD:-}" || -z "${APPLE_TEAM_ID:-}" || -z "${APPLE_SIGNING_IDENTITY:-}" || "${APPLE_SIGNING_IDENTITY}" == "-" ]]; then
  echo "Apple notarization skipped (missing identity or notary secrets); DMG at $dmg"
  exit 0
fi

xcrun notarytool submit "$dmg" \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" \
  --wait
xcrun stapler staple "$dmg"
xcrun stapler staple "$app" || true

if [[ -n "${GITHUB_TOKEN:-}" ]]; then
  tag="v${version}"
  if gh release view "$tag" >/dev/null 2>&1; then
    gh release upload "$tag" "$dmg" --clobber
  else
    echo "Release $tag does not exist yet; DMG left at $dmg"
  fi
fi

echo "notarized DMG: $dmg"
