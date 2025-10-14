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
  if [[ "$QUIET" == "1" ]]; then
    printf '%s\n' "$*" >>"$LOG_FILE"
  else
    printf '%s\n' "$*" | tee -a "$LOG_FILE" >/dev/null
  fi
}

qrun() {
  if [[ "$QUIET" == "1" ]]; then
    "$@" >>"$LOG_FILE" 2>&1
  else
    "$@" 2>&1 | tee -a "$LOG_FILE"
  fi
}

log "==== archinstall-rs boot helper starting at $(date -Iseconds) ===="
log "ENV: TERM=${TERM:-} COLORTERM=${COLORTERM:-} DISPLAY=${DISPLAY:-} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-}"

# Default wrapper and flags
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
WRAPPER="$SCRIPT_DIR/run-tui.sh"
INSTALLER_FLAGS=${INSTALLER_FLAGS:---dry-run --debug}

# Locate wrapper from common locations
for cand in \
  "$WRAPPER" \
  "$SCRIPT_DIR/bin/run-tui.sh" \
  "/usr/local/archinstall-rs/run-tui.sh" \
  "$SCRIPT_DIR/assets/iso/run-tui.sh" \
; do
  if [[ -f "$cand" ]]; then
    WRAPPER="$cand"
    break
  fi
done
log "Using installer wrapper: $WRAPPER"

maybe_launch_graphical() {
  for term in foot kitty alacritty xterm; do
    if command -v "$term" >/dev/null 2>&1; then
      log "Launching $term for installer..."
      ( "$term" -e bash -lc "$WRAPPER $INSTALLER_FLAGS" ) && exit 0 || true
    fi
  done
  return 1
}

if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  maybe_launch_graphical || true
else
  log "No graphical session detected. Installing a minimal environment..."
  if command -v pacman >/dev/null 2>&1; then
    qrun pacman -Sy --noconfirm || true
    qrun pacman -S --needed --noconfirm cage foot seatd || true
    if command -v seatd >/dev/null 2>&1; then
      qrun systemctl start seatd.service || log "WARN: seatd not started; continuing without it"
    else
      log "INFO: seatd not installed; continuing without it"
    fi
    if command -v cage >/dev/null 2>&1 && command -v foot >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      log "Starting Wayland (cage) with foot..."
      exec cage -s -- foot -e bash -lc "$WRAPPER $INSTALLER_FLAGS"
    fi

    qrun pacman -S --needed --noconfirm xorg-server xorg-xinit xterm || true
    if command -v xinit >/dev/null 2>&1 && command -v xterm >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      log "Starting Xorg (xinit) with xterm..."
      exec xinit /usr/bin/xterm -fa Monospace -fs 12 -e bash -lc "$WRAPPER $INSTALLER_FLAGS" -- :1
    fi
  fi
fi

log "ERROR: Unable to start a graphical terminal emulator for the installer."
log "Hint: Run with DEBUG=1 QUIET=0 for verbose output. Log: $LOG_FILE"
echo "Error: Unable to start a graphical terminal emulator. See log: $LOG_FILE" >&2
exit 1


