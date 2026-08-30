#!/usr/bin/env bash
# Build a release .app and install it to ~/Applications for daily use.
set -euo pipefail

APP_NAME="Zashiki Warashi"
BUNDLE_REL="src-tauri/target/release/bundle/macos/${APP_NAME}.app"
INSTALL_DIR="${HOME}/Applications"
INSTALL_PATH="${INSTALL_DIR}/${APP_NAME}.app"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: tauri:install is macOS-only (got $(uname -s))" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

echo "==> Building ${APP_NAME}.app (release, app bundle only)…"
npx tauri build --bundles app

if [[ ! -d "${BUNDLE_REL}" ]]; then
  echo "error: expected bundle missing: ${BUNDLE_REL}" >&2
  exit 1
fi

mkdir -p "${INSTALL_DIR}"

# Quit a running instance so we are not replacing a live bundle.
if pgrep -xq "${APP_NAME}" >/dev/null 2>&1 || pgrep -xq "zashiki-warashi" >/dev/null 2>&1; then
  echo "==> Quitting running ${APP_NAME}…"
  osascript -e "tell application \"${APP_NAME}\" to quit" 2>/dev/null || true
  # Fallback if the process name differs from the product name.
  pkill -x "${APP_NAME}" 2>/dev/null || true
  pkill -x "zashiki-warashi" 2>/dev/null || true
  sleep 1
fi

# Replace correctly: never cp -R over an existing .app (stale binary risk).
if [[ -d "${INSTALL_PATH}" ]]; then
  OLD="${TMPDIR:-/tmp}/${APP_NAME}-old-$$.app"
  echo "==> Moving previous install aside…"
  mv "${INSTALL_PATH}" "${OLD}"
  rm -rf "${OLD}"
fi

echo "==> Installing to ${INSTALL_PATH}…"
cp -R "${BUNDLE_REL}" "${INSTALL_PATH}"
xattr -dr com.apple.quarantine "${INSTALL_PATH}" 2>/dev/null || true

echo "Installed ${APP_NAME}.app → ${INSTALL_PATH}"
echo "Open from Spotlight, Dock, or Launchpad. Do not run alongside tauri:dev (shared SQLite)."
