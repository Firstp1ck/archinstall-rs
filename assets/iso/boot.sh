#!/usr/bin/env bash
set -euo pipefail

# Enable tracing with DEBUG=1, silence package output with QUIET=1 (default)
[[ -n "${DEBUG:-}" ]] && set -x
QUIET=${QUIET:-1}

# Log file (override with LOG_FILE=/path)
LOG_FILE=${LOG_FILE:-/var/log/archinstall-iso-boot.log}
mkdir -p "$(dirname "$LOG_FILE")" || true
if ! touch "$LOG_FILE" 2>/dev/null; then
  LOG_FILE=/tmp/archinstall-iso-boot.log
  mkdir -p "$(dirname "$LOG_FILE")" || true
  : > "$LOG_FILE"
fi

log() {
  # Always append to log; only print to console in verbose mode
  if [[ "$QUIET" == "1" ]]; then
    printf '%s\n' "$*" >>"$LOG_FILE"
  else
    printf '%s\n' "$*" | tee -a "$LOG_FILE" >/dev/null
  fi
}

qrun() {
  # Run a command, logging stdout/stderr to the logfile. In quiet mode, do not echo to console.
  if [[ "$QUIET" == "1" ]]; then
    "$@" >>"$LOG_FILE" 2>&1
  else
    "$@" 2>&1 | tee -a "$LOG_FILE"
  fi
}

log "==== archinstall-rs ISO boot helper starting at $(date -Iseconds) ===="
log "ENV: TERM=${TERM:-} COLORTERM=${COLORTERM:-} DISPLAY=${DISPLAY:-} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-}"

# Minimal boot helper for a custom ISO. Goals:
# - If a graphical session is available, launch a terminal emulator and run the TUI wrapper.
# - Otherwise, print SSH instructions for running from a remote terminal emulator.

ROOT_DIR=/usr/local/archinstall-rs
WRAPPER="$ROOT_DIR/run-tui.sh"

maybe_launch_graphical() {
  # Prefer foot, then kitty, then alacritty, then xterm as last resort
  for term in foot kitty alacritty xterm; do
    if command -v "$term" >/dev/null 2>&1; then
      log "Launching $term for installer..."
      # Try Wayland first with foot/kitty; X11 otherwise
      ( "$term" -e bash -lc "$WRAPPER" ) && exit 0 || true
    fi
  done
  return 1
}

if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  maybe_launch_graphical || true
else
  log "No graphical session detected. Installing a minimal environment..."
  # Sync package database (best-effort) and install minimal stacks
  if command -v pacman >/dev/null 2>&1; then
    qrun pacman -Sy --noconfirm || true
    # Wayland minimal: cage + foot (+ seatd for DRM access)
    qrun pacman -S --needed --noconfirm cage foot seatd || true
    if command -v seatd >/dev/null 2>&1; then
      qrun systemctl start seatd.service || true
    fi
    if command -v cage >/dev/null 2>&1 && command -v foot >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      log "Starting Wayland (cage) with foot..."
      exec cage -s -- foot -e bash -lc "$WRAPPER"
    fi

    # Fallback: Xorg + xterm via xinit
    qrun pacman -S --needed --noconfirm xorg-server xorg-xinit xterm || true
    if command -v xinit >/dev/null 2>&1 && command -v xterm >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      log "Starting Xorg (xinit) with xterm..."
      exec xinit /usr/bin/xterm -fa Monospace -fs 12 -e bash -lc "$WRAPPER" -- :1
    fi
  fi
fi

# If we reached here, we failed to start any graphical terminal emulator
log "ERROR: Unable to start a graphical terminal emulator for the installer."
log "Hint: Run with DEBUG=1 QUIET=0 for verbose output. Log: $LOG_FILE"
echo "Error: Unable to start a graphical terminal emulator. See log: $LOG_FILE" >&2
exit 1


