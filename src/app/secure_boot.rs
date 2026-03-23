use crate::app::AppState;
use std::path::Path;

const SECURE_BOOT_EFIVAR: &str =
    "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c";

impl AppState {
    /// Detect Secure Boot from EFI variable storage. Safe to call repeatedly.
    pub fn detect_secure_boot_state(&mut self) {
        if let Some(v) = self.secure_boot_override {
            self.secure_boot_enabled = v;
            self.debug_log(&format!(
                "secure_boot: using override value secure_boot_enabled={}",
                self.secure_boot_enabled
            ));
            return;
        }
        if !self.is_uefi() {
            self.secure_boot_enabled = false;
            self.debug_log("secure_boot: host not in UEFI mode, secure_boot_enabled=false");
            return;
        }
        self.secure_boot_enabled = read_secure_boot_efivar().unwrap_or(false);
        self.debug_log(&format!(
            "secure_boot: detected secure_boot_enabled={}",
            self.secure_boot_enabled
        ));
    }

    pub fn is_secure_boot_enabled(&self) -> bool {
        self.secure_boot_override
            .unwrap_or(self.secure_boot_enabled)
    }

    /// Enforce UKI when EFISTUB is selected under Secure Boot.
    pub fn apply_secure_boot_uki_policy(&mut self) {
        if self.bootloader_index == 2 && self.is_secure_boot_enabled() && !self.uki_enabled {
            self.uki_enabled = true;
            self.info_message =
                "Secure Boot detected: UKI enabled automatically for EFISTUB.".into();
            self.debug_log("secure_boot policy: forced uki_enabled=true for EFISTUB");
        }
    }

    pub fn is_uki_forced_for_efistub(&self) -> bool {
        self.bootloader_index == 2 && self.is_secure_boot_enabled()
    }
}

fn read_secure_boot_efivar() -> Option<bool> {
    if !Path::new(SECURE_BOOT_EFIVAR).exists() {
        return None;
    }
    // EFI var format: first 4 bytes are attributes, payload starts at byte 4.
    let bytes = std::fs::read(SECURE_BOOT_EFIVAR).ok()?;
    let value = *bytes.get(4)?;
    Some(value == 1)
}
