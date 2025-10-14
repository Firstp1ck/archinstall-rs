#!/usr/bin/env bash
set -euo pipefail

# Ensure 256-color/truecolor for best visuals
export TERM=${TERM:-xterm-256color}
export COLORTERM=${COLORTERM:-truecolor}

# Require a graphical session (terminal emulator). Do not run on raw TTY.
if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  echo "Error: No graphical session detected (DISPLAY/WAYLAND_DISPLAY not set)." >&2
  echo "Please run inside a terminal emulator or use the ISO boot helper." >&2
  exit 1
fi

BIN_DIR=$(cd "$(dirname "$0")" && pwd)
APP="$BIN_DIR/target/release/archinstall-rs"

if [[ -x "$APP" ]]; then
  exec "$APP" "$@"
fi

echo "Binary not found at $APP. Building and running via cargo..."
cargo build --release

if [[ -x "$APP" ]]; then
  exec "$APP" "$@"
fi

# Fallback: run through cargo so it always launches even if the binary path differs
exec cargo run --release -- "$@"


