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
# Candidate binary locations (bundled first, then workspace targets)
CANDIDATES=(
  "$BIN_DIR/archinstall-rs"
  "$BIN_DIR/bin/archinstall-rs"
  "$BIN_DIR/target/release/archinstall-rs"
  "$BIN_DIR/../target/release/archinstall-rs"
)

for APP in "${CANDIDATES[@]}"; do
  if [[ -x "$APP" ]]; then
    exec "$APP" "$@"
  fi
done

echo "No bundled binary found. Attempting to build via cargo..."
if command -v cargo >/dev/null 2>&1; then
  cargo build --release || true
  for APP in "${CANDIDATES[@]}"; do
    if [[ -x "$APP" ]]; then
      exec "$APP" "$@"
    fi
  done
  # Last resort: cargo run
  exec cargo run --release -- "$@"
else
  echo "Error: cargo is not available and no prebuilt binary was found." >&2
  echo "Please provide a bundled archinstall-rs binary in one of: ${CANDIDATES[*]}" >&2
  exit 127
fi


