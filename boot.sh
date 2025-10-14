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
echo "Starting archinstall-rs… see $LOG_FILE for details" >&2

# Default wrapper and flags
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
WRAPPER="$SCRIPT_DIR/run-tui.sh"
# No flags by default; set INSTALLER_FLAGS explicitly when desired (e.g., "--dry-run --debug")
INSTALLER_FLAGS=${INSTALLER_FLAGS:-}

# Optional sizing knobs (columns/rows) and font size; override when invoking boot.sh
AI_TERM_COLS=${AI_TERM_COLS:-160}
AI_TERM_ROWS=${AI_TERM_ROWS:-48}
AI_FONT_SIZE=${AI_FONT_SIZE:-12}

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

# Ensure the prebuilt binary exists; download latest release if missing.
ensure_binary() {
  local -a CANDIDATES=(
    "/target/release/archinstall-rs"
    "$SCRIPT_DIR/archinstall-rs"
    "$SCRIPT_DIR/target/release/archinstall-rs"
    "$SCRIPT_DIR/../target/release/archinstall-rs"
    "/usr/local/archinstall-rs/target/release/archinstall-rs"
    "/usr/local/archinstall-rs/archinstall-rs"
  )
  for app in "${CANDIDATES[@]}"; do
    if [[ -x "$app" ]]; then
      log "Found archinstall binary: $app"
      return 0
    fi
  done
  echo "Binary not found locally. Downloading latest release…" >&2
  local url="https://github.com/Firstp1ck/archinstall-rs/releases/latest/download/archinstall-rs"
  local dst="$SCRIPT_DIR/archinstall-rs"
  if command -v wget >/dev/null 2>&1; then
    qrun wget -O "$dst" "$url" || true
  elif command -v curl >/dev/null 2>&1; then
    qrun curl -fL -o "$dst" "$url" || true
  else
    echo "Error: neither wget nor curl available to download the binary" >&2
    return 1
  fi
  if [[ -f "$dst" ]]; then
    chmod +x "$dst" || true
    log "Downloaded archinstall binary to $dst"
    return 0
  fi
  echo "Error: failed to download archinstall binary" >&2
  return 1
}

# Try to ensure the binary before launching any terminal
ensure_binary || true

maybe_launch_graphical() {
  for term in foot kitty alacritty xterm; do
    if command -v "$term" >/dev/null 2>&1; then
      log "Launching $term for installer..."
      echo "Launching $term…" >&2
      case "$term" in
        kitty)
          ( kitty \
              --override font_family='JetBrains Mono' \
              --override initial_window_width="${AI_TERM_COLS}"c \
              --override initial_window_height="${AI_TERM_ROWS}"c \
              -e bash -lc "$WRAPPER $INSTALLER_FLAGS" ) && exit 0 || true
          ;;
        alacritty)
          tmpcfg="/tmp/alacritty-archinstall.yml"
          printf '%s\n' \
"font:" \
"  normal:" \
"    family: JetBrains Mono" \
"  size: ${AI_FONT_SIZE}" \
"window:" \
"  dimensions:" \
"    columns: ${AI_TERM_COLS}" \
"    lines: ${AI_TERM_ROWS}" > "$tmpcfg"
          ( alacritty --config-file "$tmpcfg" -e bash -lc "$WRAPPER $INSTALLER_FLAGS" ) && exit 0 || true
          ;;
        xterm)
          ( xterm -fa 'JetBrains Mono' -fs "${AI_FONT_SIZE}" -geometry "${AI_TERM_COLS}"x"${AI_TERM_ROWS}" -e bash -lc "$WRAPPER $INSTALLER_FLAGS" ) && exit 0 || true
          ;;
        *)
          ( "$term" -e bash -lc "$WRAPPER $INSTALLER_FLAGS" ) && exit 0 || true
          ;;
      esac
    fi
  done
  return 1
}

if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  maybe_launch_graphical || true
else
  log "No graphical session detected. Installing a minimal environment..."
  echo "No GUI session: preparing minimal environment (this may take ~10s)…" >&2
  if command -v pacman >/dev/null 2>&1; then
    echo "Syncing package database…" >&2
    qrun pacman -Sy --noconfirm || true
    # Install minimal terminal stacks and JetBrains Mono font
    echo "Installing cage, foot, seatd, JetBrains Mono…" >&2
    qrun pacman -S --needed --noconfirm cage foot seatd ttf-jetbrains-mono || true
    if command -v seatd >/dev/null 2>&1; then
      echo "Starting seatd.service…" >&2
      qrun systemctl start seatd.service || log "WARN: seatd not started; continuing without it"
    else
      log "INFO: seatd not installed; continuing without it"
    fi
    if command -v cage >/dev/null 2>&1 && command -v foot >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      log "Starting Wayland (cage) with foot..."
      echo "Launching Wayland/foot…" >&2
      exec cage -s -- foot -e bash -lc "$WRAPPER $INSTALLER_FLAGS"
    fi

    qrun pacman -S --needed --noconfirm xorg-server xorg-xinit xterm ttf-jetbrains-mono || true
    if command -v xinit >/dev/null 2>&1 && command -v xterm >/dev/null 2>&1; then
      export TERM=${TERM:-xterm-256color}
      export COLORTERM=${COLORTERM:-truecolor}
      log "Starting Xorg (xinit) with xterm (JetBrains Mono)..."
      echo "Launching Xorg/xterm…" >&2
      exec xinit /usr/bin/xterm -fa 'JetBrains Mono' -fs "${AI_FONT_SIZE}" -geometry "${AI_TERM_COLS}"x"${AI_TERM_ROWS}" -e bash -lc "$WRAPPER $INSTALLER_FLAGS" -- :1
    fi
  fi
fi

log "ERROR: Unable to start a graphical terminal emulator for the installer."
log "Hint: Run with DEBUG=1 QUIET=0 for verbose output. Log: $LOG_FILE"
echo "Error: Unable to start a graphical terminal emulator. See log: $LOG_FILE" >&2
exit 1


