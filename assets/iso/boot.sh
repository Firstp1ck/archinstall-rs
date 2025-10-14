#!/usr/bin/env bash
set -euo pipefail

# Minimal boot helper for a custom ISO. Goals:
# - If a graphical session is available, launch a terminal emulator and run the TUI wrapper.
# - Otherwise, print SSH instructions for running from a remote terminal emulator.

ROOT_DIR=/usr/local/archinstall-rs
WRAPPER="$ROOT_DIR/run-tui.sh"

maybe_launch_graphical() {
  # Prefer foot, then kitty, then alacritty, then xterm as last resort
  for term in foot kitty alacritty xterm; do
    if command -v "$term" >/dev/null 2>&1; then
      echo "Launching $term for installer..."
      # Try Wayland first with foot/kitty; X11 otherwise
      ( "$term" -e bash -lc "$WRAPPER" ) && exit 0 || true
    fi
  done
  return 1
}

if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  maybe_launch_graphical || true
else
  echo "No graphical session detected. Installing a minimal environment..."
  # Sync package database (best-effort) and install minimal stacks
  if command -v pacman >/dev/null 2>&1; then
    pacman -Sy --noconfirm >/dev/null 2>&1 || true
    # Wayland minimal: cage + foot (+ seatd for DRM access)
    pacman -S --needed --noconfirm cage foot seatd >/dev/null 2>&1 || true
    if command -v seatd >/dev/null 2>&1; then
      systemctl start seatd.service >/dev/null 2>&1 || true
    fi
    if command -v cage >/dev/null 2>&1 && command -v foot >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      echo "Starting Wayland (cage) with foot..."
      exec cage -s -- foot -e bash -lc "$WRAPPER"
    fi

    # Fallback: Xorg + xterm via xinit
    pacman -S --needed --noconfirm xorg-server xorg-xinit xterm >/dev/null 2>&1 || true
    if command -v xinit >/dev/null 2>&1 && command -v xterm >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      echo "Starting Xorg (xinit) with xterm..."
      exec xinit /usr/bin/xterm -fa Monospace -fs 12 -e bash -lc "$WRAPPER" -- :1
    fi
  fi
fi

# Last resort: run the TUI in the current console
export TERM=${TERM:-xterm-256color}
export COLORTERM=${COLORTERM:-truecolor}
exec "$WRAPPER"


