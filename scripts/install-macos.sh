#!/usr/bin/env bash
# Build a release .app and install it to /Applications for daily use.
# Uses macOS Authorization Services (Touch ID / password) when privileges are needed —
# not sudo, so the standard auth sheet appears.
set -euo pipefail

APP_NAME="Zashiki Warashi"
BUNDLE_REL="src-tauri/target/release/bundle/macos/${APP_NAME}.app"
INSTALL_DIR="/Applications"
INSTALL_PATH="${INSTALL_DIR}/${APP_NAME}.app"
LEGACY_PATH="${HOME}/Applications/${APP_NAME}.app"
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: tauri:install is macOS-only (got $(uname -s))" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

echo "==> Building ${APP_NAME}.app (release, app bundle only)…"
npx tauri build --bundles app

BUNDLE_ABS="$(cd "$(dirname "${BUNDLE_REL}")" && pwd)/$(basename "${BUNDLE_REL}")"
if [[ ! -d "${BUNDLE_ABS}" ]]; then
  echo "error: expected bundle missing: ${BUNDLE_REL}" >&2
  exit 1
fi

# Quit a running instance so we are not replacing a live bundle.
if pgrep -xq "${APP_NAME}" >/dev/null 2>&1 || pgrep -xq "zashiki-warashi" >/dev/null 2>&1; then
  echo "==> Quitting running ${APP_NAME}…"
  osascript -e "tell application \"${APP_NAME}\" to quit" 2>/dev/null || true
  # Fallback if the process name differs from the product name.
  pkill -x "${APP_NAME}" 2>/dev/null || true
  pkill -x "zashiki-warashi" 2>/dev/null || true
  sleep 1
fi

can_write_unprivileged() {
  if [[ ! -w "${INSTALL_DIR}" ]]; then
    return 1
  fi
  if [[ -e "${INSTALL_PATH}" && ! -w "${INSTALL_PATH}" ]]; then
    return 1
  fi
  return 0
}

install_unprivileged() {
  if [[ -d "${INSTALL_PATH}" ]]; then
    OLD="${TMPDIR:-/tmp}/${APP_NAME}-old-$$.app"
    echo "==> Moving previous install aside…"
    mv "${INSTALL_PATH}" "${OLD}"
    rm -rf "${OLD}"
  fi
  echo "==> Installing to ${INSTALL_PATH}…"
  ditto "${BUNDLE_ABS}" "${INSTALL_PATH}"
  xattr -dr com.apple.quarantine "${INSTALL_PATH}" 2>/dev/null || true
}

# Privileged copy via Authorization Services (Touch ID when enrolled, else password).
# Paths arrive as argv; quoted form of keeps spaces in the app name safe.
install_privileged() {
  local owner
  owner="$(id -un)"
  echo "==> Installing to ${INSTALL_PATH} (administrator privileges required)…"
  if ! osascript \
    -e 'on run argv' \
    -e '  set src to item 1 of argv' \
    -e '  set dest to item 2 of argv' \
    -e '  set ownerName to item 3 of argv' \
    -e '  set cmd to "rm -rf " & quoted form of dest & " && ditto " & quoted form of src & " " & quoted form of dest & " && xattr -dr com.apple.quarantine " & quoted form of dest & " && chown -R " & quoted form of ownerName & " " & quoted form of dest' \
    -e '  do shell script cmd with administrator privileges' \
    -e 'end run' \
    -- "${BUNDLE_ABS}" "${INSTALL_PATH}" "${owner}"; then
    echo "error: install cancelled or authorization failed. ${APP_NAME} must be in /Applications for Spotlight." >&2
    exit 1
  fi
}

if can_write_unprivileged; then
  install_unprivileged
else
  install_privileged
fi

# Drop leftover ~/Applications copy so Launch Services / Spotlight prefer /Applications.
if [[ -d "${LEGACY_PATH}" ]]; then
  echo "==> Removing legacy ${LEGACY_PATH}…"
  rm -rf "${LEGACY_PATH}"
fi

if [[ -x "${LSREGISTER}" ]]; then
  "${LSREGISTER}" -f "${INSTALL_PATH}" >/dev/null 2>&1 || true
fi

echo "Installed ${APP_NAME}.app → ${INSTALL_PATH}"
echo "Open from Spotlight, Dock, or Launchpad. Do not run alongside tauri:dev (shared SQLite)."
