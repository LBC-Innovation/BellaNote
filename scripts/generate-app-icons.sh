#!/usr/bin/env bash
# Rasterize Media/Logo.svg into Tauri, installer, and in-app icon files.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
src="$root/Media/Logo.svg"
icon_src="$root/Media/logo-app-icon.png"
icons="$root/src-tauri/icons"
public="$root/public"
bg="#12161c"

if [[ ! -f "$src" ]]; then
  echo "missing source logo: $src" >&2
  exit 1
fi

if ! command -v magick >/dev/null 2>&1; then
  echo "ImageMagick (magick) is required to generate icons" >&2
  exit 1
fi

mkdir -p "$public" "$icons"
cp "$src" "$public/logo.svg"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Keep the portrait mark sharp, then place it on a rounded glass plate.
magick -background none "$src" -resize x2400 "$tmp/mark.png"

# Transparent canvas, then a rounded glass plate, then the centered mark.
# A single -composite with three images treats the third as a mask — layer them in two steps.
magick -size 1024x1024 xc:none \
  -fill "rgba(18,22,28,0.16)" \
  -stroke "rgba(184,252,226,0.48)" \
  -strokewidth 10 \
  -draw "roundrectangle 56,56 967,967 280,280" \
  PNG32:"$tmp/plate.png"

magick "$tmp/plate.png" \
  \( "$tmp/mark.png" -resize x640 \) \
  -gravity center -compose over -composite \
  -depth 8 PNG32:"$icon_src"

npx tauri icon "$icon_src" -o "$icons"
# Desktop-only product — drop mobile icon trees the CLI still emits.
rm -rf "$icons/ios" "$icons/android"

magick "$icon_src" -resize 32x32 "$public/favicon-32.png"
magick "$icon_src" -resize 192x192 "$public/apple-touch-icon.png"
magick "$icon_src" -define icon:auto-resize=256,128,64,48,32,16 "$public/favicon.ico"

# NSIS installer chrome (Windows BMP3).
magick -size 150x57 "xc:${bg}" \
  \( "$tmp/mark.png" -resize x40 \) \
  -gravity west -geometry +10+0 -composite \
  -type TrueColor BMP3:"$icons/nsis-header.bmp"

magick -size 164x314 "xc:${bg}" \
  \( "$tmp/mark.png" -resize x168 \) \
  -gravity center -composite \
  -type TrueColor BMP3:"$icons/nsis-sidebar.bmp"
