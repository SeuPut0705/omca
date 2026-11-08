#!/usr/bin/env bash
# Regenerate every app icon from assets/app-icon/craft-anything-master.png (the canonical artwork,
# a generated 1024 px square), clipped to the rounded tile. craft-anything.svg is its hand-drawn
# vector twin, used where a vector is needed (the in-app icon, hicolor scalable).
#
# Needs: resvg (brew install resvg / cargo install resvg). On macOS, iconutil also writes the
# .icns. The outputs are committed, so building and packaging never need these tools.
#
#   packaging/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
MASTER="$DIR/craft-anything-master.png"
VECTOR="$DIR/craft-anything.svg"
ID="com.seuput.omca"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

command -v resvg >/dev/null || { echo "error: resvg not found (brew install resvg)" >&2; exit 1; }

# The master clipped to a full-bleed 512 tile with 110-unit corners, used as-is on Windows.
# macOS icons follow Apple's grid: the tile is 824/1024 of the canvas with a transparent margin,
# so widen the viewBox around it.
tile() {
  printf '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="%s"><clipPath id="t"><rect width="512" height="512" rx="110"/></clipPath><image clip-path="url(#t)" width="512" height="512" xlink:href="%s"/></svg>' "$1" "$MASTER"
}
SVG="$TMP/tile.svg"
MAC="$TMP/mac.svg"
tile "0 0 512 512" >"$SVG"
tile "-62.136 -62.136 636.272 636.272" >"$MAC"

render() { resvg -w "$2" -h "$2" "$1" "$3" </dev/null; }

render "$SVG" 1024 "$DIR/craft-anything-1024.png"

# Linux hicolor theme (the 256 one is also the runtime icon on Windows and Linux).
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  render "$SVG" "$s" "$DIR/hicolor/${s}x${s}/apps/$ID.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
cp "$VECTOR" "$DIR/hicolor/scalable/apps/$ID.svg"

# Windows .ico.
ICO_PNGS=()
for s in 16 20 24 32 40 48 64 128 256; do
  render "$SVG" "$s" "$TMP/ico-$s.png"
  ICO_PNGS+=("$TMP/ico-$s.png")
done
(cd "$ROOT" && cargo run -q -p xtask -- ico "$DIR/craft-anything.ico" "${ICO_PNGS[@]}")

# macOS .icns.
if command -v iconutil >/dev/null; then
  SET="$TMP/craft-anything.iconset"
  mkdir -p "$SET"
  for s in 16 32 128 256 512; do
    render "$MAC" "$s" "$SET/icon_${s}x${s}.png"
    render "$MAC" $((s * 2)) "$SET/icon_${s}x${s}@2x.png"
  done
  iconutil -c icns -o "$DIR/craft-anything.icns" "$SET"
else
  echo "warning: iconutil not found (macOS only); craft-anything.icns not regenerated" >&2
fi
echo "icons written to $DIR"
