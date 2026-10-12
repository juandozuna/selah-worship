#!/usr/bin/env bash
# Build a user-facing DMG that contains Selah.app (Applications link) and the volunteer PDF.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$(ls -d "$ROOT"/src-tauri/target/release/bundle/macos/*.app 2>/dev/null | head -1 || true)"
PDF="$ROOT/docs/guide/Selah-Volunteer-Guide.pdf"
OUT_DIR="$ROOT/src-tauri/target/release/bundle/dmg"
STAGE="$ROOT/src-tauri/target/release/bundle/dmg-stage"
VERSION="$(node -p "require('$ROOT/package.json').version")"
DMG="$OUT_DIR/Selah_${VERSION}_aarch64.dmg"

if [[ -z "$APP" || ! -d "$APP" ]]; then
  echo "Build the macOS app first: npm run desktop:build" >&2
  exit 1
fi
if [[ ! -f "$PDF" ]]; then
  echo "Missing guide PDF. Run: npm run guide:pdf" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
rm -rf "$STAGE"
mkdir -p "$STAGE"
ditto "$APP" "$STAGE/$(basename "$APP")"
cp "$PDF" "$STAGE/Selah-Volunteer-Guide.pdf"
ln -sf /Applications "$STAGE/Applications"

rm -f "$DMG"
hdiutil create -volname "Selah" -srcfolder "$STAGE" -ov -format UDZO "$DMG"
echo "Created $DMG"
ls -lh "$DMG"
