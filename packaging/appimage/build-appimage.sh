#!/usr/bin/env bash
# Builds an AppImage for gtop-gui using appimagetool.
# Requirements: appimagetool in PATH, built release binary at target/release/gtop-gui

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="${VERSION:-0.1.0}"
APPDIR="$REPO_ROOT/target/appimage/gtop-gui.AppDir"

echo "[+] Building release binary..."
cargo build --release --package gtop-gui --manifest-path "$REPO_ROOT/Cargo.toml"

echo "[+] Creating AppDir structure..."
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/icons/hicolor/scalable/apps"

cp "$REPO_ROOT/target/release/gtop-gui" "$APPDIR/usr/bin/gtop-gui"
cp "$REPO_ROOT/assets/icons/gtop.svg"   "$APPDIR/usr/share/icons/hicolor/scalable/apps/gtop.svg"
cp "$REPO_ROOT/packaging/appimage/gtop-gui.desktop" "$APPDIR/gtop-gui.desktop"
cp "$REPO_ROOT/packaging/appimage/AppRun" "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"

# Desktop file also at root for appimagetool
ln -sf gtop.svg "$APPDIR/.DirIcon" 2>/dev/null || true

echo "[+] Running appimagetool..."
ARCH="$(uname -m)" appimagetool "$APPDIR" "$REPO_ROOT/target/gtop-gui-$VERSION-$ARCH.AppImage"

echo "[+] Done: target/gtop-gui-$VERSION-$ARCH.AppImage"
