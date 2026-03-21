use super::{AppState, PopupKind};
use crate::app::config::presets::{ConfigPresetTableRow, search_blob_for_row};

impl AppState {
    #[allow(dead_code)]
    pub fn init_save_configuration(&mut self) {
        // TODO: Implement Save/Load configuration actions UI (TOML IO).
    }

    pub fn open_config_load_popup(&mut self) {
        self.popup_kind = Some(PopupKind::ConfigLoadSelect);
        self.popup_open = true;

        let cwd_path = Self::cwd_config_path();
        let cwd_row = ConfigPresetTableRow {
            path: cwd_path.clone(),
            country: "(local)".into(),
            language: "—".into(),
            desktop: "—".into(),
            additional: if cwd_path.exists() {
                format!("Saved config at {}", cwd_path.display())
            } else {
                format!(
                    "No file yet — will be created on save ({})",
                    cwd_path.display()
                )
            },
        };

        let mut rows: Vec<ConfigPresetTableRow> = vec![cwd_row];
        let mut items: Vec<String> = vec![search_blob_for_row(&rows[0])];

        for r in &self.config_preset_rows {
            items.push(search_blob_for_row(r));
            rows.push(r.clone());
        }

        self.config_popup_rows = rows;
        self.popup_items = items;
        self.popup_search_query.clear();
        self.popup_in_search = false;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
    }

    fn cwd_config_path() -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("archinstall-rs.config.toml")
    }

    /// Load a config from path and return the user-facing message (success or warning).
    pub fn load_config_and_message(&mut self, path: &std::path::Path) -> String {
        match self.load_config_from_path(path) {
            Ok(()) => {
                let mut msg = String::from(
                    "Configuration loaded successfully.\n\nNote: Please re-enter the following sensitive fields:\n  - Disk encryption password\n  - Root password\n  - User passwords",
                );
                if !self.last_load_missing_sections.is_empty() {
                    msg.push_str("\n\nMissing sections (not found in file):\n");
                    for (i, section) in self.last_load_missing_sections.iter().enumerate() {
                        if i > 0 {
                            msg.push('\n');
                        }
                        msg.push_str(&format!("  - {section}"));
                    }
                }
                msg
            }
            Err(_e) => format!("Failed to load configuration from {}", path.display()),
        }
    }
}
