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

bash "$root/scripts/macos_resign_sidecar.sh" "$app"
bash "$root/scripts/smoke_transcribe_worker.sh" "$app"

version="$(python3 -c "import json; print(json.load(open('$root/src-tauri/tauri.conf.json'))['version'])")"
volname="$(python3 -c "import json; print(json.load(open('$root/src-tauri/tauri.conf.json'))['productName'])")"
dmg="$dmg_dir/${volname}_${version}_aarch64.dmg"
mkdir -p "$dmg_dir"

staging="$(mktemp -d)"
trap 'rm -rf "$staging"' EXIT
cp -R "$app" "$staging/"
ln -s /Applications "$staging/Applications"
rm -f "$dmg"
hdiutil create -volname "$volname" -srcfolder "$staging" -ov -format UDZO "$dmg"

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
