#!/usr/bin/env bash
set -euo pipefail

# Ensure 256-color/truecolor for best visuals and prefer JetBrains Mono in terminals that support it
export TERM=${TERM:-xterm-256color}
export COLORTERM=${COLORTERM:-truecolor}
export KITTY_OVERRIDE="font_family=JetBrains Mono"

# Require a graphical session (terminal emulator). Do not run on raw TTY.
if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  echo "Error: No graphical session detected (DISPLAY/WAYLAND_DISPLAY not set)." >&2
  echo "Please run inside a terminal emulator or use the ISO boot helper." >&2
  exit 1
fi

BIN_DIR=$(cd "$(dirname "$0")" && pwd)
# Strictly use the prebuilt binary at an absolute path
APP="/target/release/archinstall-rs"
if [[ -x "$APP" ]]; then
  exec "$APP" "$@"
fi
echo "Error: prebuilt binary not found or not executable: $APP" >&2
echo "Please ensure the binary exists at $APP before launching." >&2
exit 127


