#!/usr/bin/env bash
# Rasterize Media/Logo.svg into Tauri, installer, and in-app icon files.
#
# The dock/taskbar mark is the speech-bubble logo on a fully transparent
# square — no nested glass plate. A rounded fill inside the 1024 canvas
# reads as a shrunken glyph on a gray tile once macOS applies its own
# squircle to the bundled .icns (dev mode shows the PNG alpha as-is).
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

write_png() {
  local dest="$1"
  shift
  local out="$tmp/write-png.png"
  magick "$@" \
    -strip \
    -define png:exclude-chunks=bKGD,gAMA,cHRM,tEXt,zTXt,date \
    -depth 8 \
    PNG32:"$out"
  mkdir -p "$(dirname "$dest")"
  mv -f "$out" "$dest"
}

# Keep the portrait mark sharp, then fit it to the full 1024 canvas.
magick -background none "$src" -resize x2400 "$tmp/mark.png"

write_png "$icon_src" \
  -size 1024x1024 xc:none \
  \( "$tmp/mark.png" -resize 1024x1024 \) \
  -gravity center -compose over -composite

npx tauri icon "$icon_src" -o "$icons"
# Desktop-only product — drop mobile icon trees the CLI still emits.
rm -rf "$icons/ios" "$icons/android"

# `tauri icon` writes a PNG bKGD chunk (white). Finder/Dock use that as a
# fill behind transparent pixels in the bundled app. Strip it from every
# PNG the CLI emitted, then mint the .icns with iconutil.
shopt -s nullglob
for png in "$icons"/*.png; do
  write_png "$png" "$png"
done
shopt -u nullglob

write_png "$public/favicon-32.png" "$icon_src" -resize 32x32
write_png "$public/apple-touch-icon.png" "$icon_src" -resize 192x192
magick "$icon_src" -strip -define icon:auto-resize=256,128,64,48,32,16 "$public/favicon.ico"

if command -v iconutil >/dev/null 2>&1; then
  iconset="$tmp/AppIcon.iconset"
  mkdir -p "$iconset"
  while read -r size name; do
    write_png "$iconset/${name}.png" "$icon_src" -resize "${size}x${size}"
  done <<'SIZES'
16 icon_16x16
32 icon_16x16@2x
32 icon_32x32
64 icon_32x32@2x
128 icon_128x128
256 icon_128x128@2x
256 icon_256x256
512 icon_256x256@2x
512 icon_512x512
1024 icon_512x512@2x
SIZES
  iconutil --convert icns --output "$icons/icon.icns" "$iconset"
fi

# NSIS installer chrome (Windows BMP3) stays on the charcoal plate — that
# art is a filled banner, not a dock tile.
magick -size 150x57 "xc:${bg}" \
  \( "$tmp/mark.png" -resize x40 \) \
  -gravity west -geometry +10+0 -composite \
  -type TrueColor BMP3:"$icons/nsis-header.bmp"

magick -size 164x314 "xc:${bg}" \
  \( "$tmp/mark.png" -resize x168 \) \
  -gravity center -composite \
  -type TrueColor BMP3:"$icons/nsis-sidebar.bmp"
