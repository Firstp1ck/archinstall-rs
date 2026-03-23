use crate::app::AppState;
use std::path::Path;
use std::process::Command;

const SECURE_BOOT_EFIVAR: &str =
    "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c";
const SETUP_MODE_EFIVAR: &str =
    "/sys/firmware/efi/efivars/SetupMode-8be4df61-93ca-11d2-aa0d-00e098032b8c";

impl AppState {
    /// Detect Secure Boot from EFI variable storage. Safe to call repeatedly.
    pub fn detect_secure_boot_state(&mut self) {
        if let Some(v) = self.secure_boot_override {
            self.secure_boot_enabled = v;
            self.secure_boot_known = true;
            self.debug_log(&format!(
                "secure_boot: using override value secure_boot_enabled={}",
                self.secure_boot_enabled
            ));
            return;
        }
        if !self.is_uefi() {
            self.secure_boot_enabled = false;
            self.secure_boot_known = true;
            self.debug_log("secure_boot: host not in UEFI mode, secure_boot_enabled=false");
            return;
        }
        ensure_efivarfs_mounted();
        self.secure_boot_setup_mode = read_efivar_bool(SETUP_MODE_EFIVAR).unwrap_or(false);
        if self.secure_boot_setup_mode {
            self.debug_log("secure_boot: firmware is in Setup Mode (no keys enrolled)");
        }
        if let Some(v) = read_efivar_bool(SECURE_BOOT_EFIVAR) {
            self.secure_boot_enabled = v;
            self.secure_boot_known = true;
            self.debug_log(&format!(
                "secure_boot: detected from efivar secure_boot_enabled={} setup_mode={}",
                self.secure_boot_enabled, self.secure_boot_setup_mode
            ));
            return;
        }
        if let Some(v) = detect_secure_boot_from_bootctl() {
            self.secure_boot_enabled = v;
            self.secure_boot_known = true;
            self.debug_log(&format!(
                "secure_boot: detected from bootctl secure_boot_enabled={}",
                self.secure_boot_enabled
            ));
            return;
        }
        if let Some(v) = detect_secure_boot_from_mokutil() {
            self.secure_boot_enabled = v;
            self.secure_boot_known = true;
            self.debug_log(&format!(
                "secure_boot: detected from mokutil secure_boot_enabled={}",
                self.secure_boot_enabled
            ));
            return;
        }
        self.secure_boot_enabled = false;
        self.secure_boot_known = false;
        self.debug_log(&format!(
            "secure_boot: status unknown after probes, defaulting secure_boot_enabled={}",
            self.secure_boot_enabled
        ));
    }

    pub fn is_secure_boot_enabled(&self) -> bool {
        self.secure_boot_override
            .unwrap_or(self.secure_boot_enabled)
    }

    pub fn secure_boot_status_text(&self) -> &'static str {
        if self.is_secure_boot_enabled() {
            "Enabled"
        } else if self.secure_boot_setup_mode {
            "Disabled (Setup Mode - no keys enrolled)"
        } else if self.secure_boot_known {
            "Disabled"
        } else {
            "Unknown"
        }
    }

    /// Enforce UKI when EFISTUB is selected under Secure Boot.
    pub fn apply_secure_boot_uki_policy(&mut self) {
        if self.bootloader_index == 2 && self.is_secure_boot_enabled() && !self.uki_enabled {
            self.uki_enabled = true;
            self.info_message =
                "Secure Boot detected: UKI enabled automatically for Efistub (experimental)."
                    .into();
            self.debug_log("secure_boot policy: forced uki_enabled=true for EFISTUB");
        }
    }

    pub fn is_uki_forced_for_efistub(&self) -> bool {
        self.bootloader_index == 2 && self.is_secure_boot_enabled()
    }
}

fn ensure_efivarfs_mounted() {
    let efivars = Path::new("/sys/firmware/efi/efivars");
    if efivars.is_dir() && std::fs::read_dir(efivars).map_or(true, |mut d| d.next().is_none()) {
        let _ = Command::new("mount")
            .args(["-t", "efivarfs", "efivarfs", "/sys/firmware/efi/efivars"])
            .output();
    }
}

fn read_efivar_bool(path: &str) -> Option<bool> {
    if !Path::new(path).exists() {
        return None;
    }
    // EFI var format: first 4 bytes are attributes, payload starts at byte 4.
    let bytes = std::fs::read(path).ok()?;
    let value = *bytes.get(4)?;
    Some(value == 1)
}

fn detect_secure_boot_from_bootctl() -> Option<bool> {
    let out = Command::new("bootctl")
        .args(["status", "--no-pager"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    if text.contains("secure boot: enabled") {
        Some(true)
    } else if text.contains("secure boot: disabled") {
        Some(false)
    } else {
        None
    }
}

fn detect_secure_boot_from_mokutil() -> Option<bool> {
    let out = Command::new("mokutil").arg("--sb-state").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    if text.contains("secureboot enabled") {
        Some(true)
    } else if text.contains("secureboot disabled") {
        Some(false)
    } else {
        None
    }
}
