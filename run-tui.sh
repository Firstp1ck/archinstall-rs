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
# Prefer a prebuilt binary; search common locations. Do NOT use cargo.
CANDIDATES=(
  "/target/release/archinstall-rs"                    # absolute, if mounted at /
  "$BIN_DIR/archinstall-rs"                           # bundled next to script
  "$BIN_DIR/target/release/archinstall-rs"            # repo-local build
  "$BIN_DIR/../target/release/archinstall-rs"         # parent repo build
  "/usr/local/archinstall-rs/target/release/archinstall-rs" # ISO installed path
  "/usr/local/archinstall-rs/archinstall-rs"          # bundled in /usr/local
)

for APP in "${CANDIDATES[@]}"; do
  if [[ -x "$APP" ]]; then
    exec "$APP" "$@"
  fi
done

echo "Error: prebuilt binary not found in any of the expected locations:" >&2
printf ' - %s\n' "${CANDIDATES[@]}" >&2
echo "Please place the built binary in one of the paths above and retry." >&2
exit 127


