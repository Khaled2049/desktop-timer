#!/bin/sh
# Installs the latest Desktop Timer release into Applications.
set -eu

REPO="Khaled2049/desktop-timer"
APP="Desktop Timer.app"
INSTALL_DIR="${DESKTOP_TIMER_INSTALL_DIR:-/Applications}"

fail() {
  echo "Error: $1" >&2
  exit 1
}

[ "$(uname -s)" = "Darwin" ] || fail "Desktop Timer only runs on macOS."
[ "$(uname -m)" = "arm64" ] || fail "This download only supports Apple Silicon Macs."
[ -d "$INSTALL_DIR" ] && [ -w "$INSTALL_DIR" ] || fail "Cannot write to $INSTALL_DIR."

WORK="$(mktemp -d)"
MOUNT="$WORK/mount"
cleanup() {
  [ -d "$MOUNT" ] && hdiutil detach "$MOUNT" -quiet 2>/dev/null || true
  rm -rf "$WORK"
}
trap cleanup EXIT

echo "Finding the latest release..."
ASSETS="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
  sed -n 's/.*"browser_download_url": *"\([^"]*\)".*/\1/p')"
DMG_URL="$(echo "$ASSETS" | grep '\.dmg$' | head -n 1)"
SUMS_URL="$(echo "$ASSETS" | grep '/SHA256SUMS\.txt$' | head -n 1 || true)"
[ -n "$DMG_URL" ] || fail "Could not find a DMG in the latest release."
DMG="$WORK/$(basename "$DMG_URL")"

echo "Downloading $(basename "$DMG_URL")..."
curl -fsSL -o "$DMG" "$DMG_URL"

if [ -n "$SUMS_URL" ]; then
  curl -fsSL -o "$WORK/SHA256SUMS.txt" "$SUMS_URL"
  (cd "$WORK" && shasum -a 256 -c SHA256SUMS.txt >/dev/null) || fail "The download did not match its checksum."
fi

mkdir "$MOUNT"
hdiutil attach "$DMG" -nobrowse -readonly -mountpoint "$MOUNT" -quiet
[ -d "$MOUNT/$APP" ] || fail "The DMG does not contain $APP."

# Stop an installed copy so its files can be replaced.
pkill -f "$INSTALL_DIR/$APP/" 2>/dev/null || true
rm -rf "$INSTALL_DIR/$APP"
ditto "$MOUNT/$APP" "$INSTALL_DIR/$APP"

VERSION="$(defaults read "$INSTALL_DIR/$APP/Contents/Info" CFBundleShortVersionString)"
echo "Installed Desktop Timer $VERSION in $INSTALL_DIR."
echo "Start it with: open \"$INSTALL_DIR/$APP\""
