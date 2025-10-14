#!/usr/bin/env bash
set -euo pipefail

# Ensure 256-color/truecolor for best visuals
export TERM=${TERM:-xterm-256color}
export COLORTERM=${COLORTERM:-truecolor}

BIN_DIR=$(cd "$(dirname "$0")" && pwd)
APP="$BIN_DIR/target/release/archinstall-rs"

if [[ ! -x "$APP" ]]; then
  echo "Building release binary..."
  cargo build --release
fi

exec "$APP" "$@"


