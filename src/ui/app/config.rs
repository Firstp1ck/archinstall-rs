use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{AppState, CustomRepo, RepoSignOption, RepoSignature, UserAccount};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigLocales {
    pub keyboard_layout: String,
    pub locale_language: String,
    pub locale_encoding: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigMirrors {
    pub regions: Vec<String>,
    pub optional_repos: Vec<String>,
    pub custom_servers: Vec<String>,
    pub custom_repos: Vec<CustomRepoConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomRepoConfig {
    pub name: String,
    pub url: String,
    pub signature: String,
    pub sign_option: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigDisks {
    pub mode: String,
    pub selected_device: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigDiskEncryption {
    pub encryption_type: String,
    pub partition: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigSwap {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigBootloader {
    pub kind: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub locales: ConfigLocales,
    pub mirrors: ConfigMirrors,
    pub disks: ConfigDisks,
    pub disk_encryption: ConfigDiskEncryption,
    pub swap: ConfigSwap,
    pub bootloader: ConfigBootloader,
    pub users: Vec<ConfigUser>,
}

impl AppState {
    fn config_path() -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            .join("archinstall-rs.config.toml")
    }

    fn build_config(&self) -> AppConfig {
        let locales = ConfigLocales {
            keyboard_layout: self.current_keyboard_layout(),
            locale_language: self.current_locale_language(),
            locale_encoding: self.current_locale_encoding(),
        };
        let mirrors = ConfigMirrors {
            regions: self
                .mirrors_regions_selected
                .iter()
                .filter_map(|&i| self.mirrors_regions_options.get(i).cloned())
                .collect(),
            optional_repos: self
                .optional_repos_selected
                .iter()
                .filter_map(|&i| self.optional_repos_options.get(i).cloned())
                .collect(),
            custom_servers: self.mirrors_custom_servers.clone(),
            custom_repos: self
                .custom_repos
                .iter()
                .map(|r: &CustomRepo| CustomRepoConfig {
                    name: r.name.clone(),
                    url: r.url.clone(),
                    signature: match r.signature {
                        RepoSignature::Never => "Never".into(),
                        RepoSignature::Optional => "Optional".into(),
                        RepoSignature::Required => "Required".into(),
                    },
                    sign_option: r.sign_option.map(|s| match s {
                        RepoSignOption::TrustedOnly => "TrustedOnly".into(),
                        RepoSignOption::TrustedAll => "TrustedAll".into(),
                    }),
                })
                .collect(),
        };
        let disks_mode = match self.disks_mode_index {
            0 => "Best-effort default partition layout",
            1 => "Manual Partitioning",
            _ => "Pre-mounted configuration",
        };
        let disks = ConfigDisks { mode: disks_mode.into(), selected_device: self.disks_selected_device.clone() };
        let encryption_type = match self.disk_encryption_type_index { 0 => "None", _ => "LUKS" }.to_string();
        let password_hash = if self.disk_encryption_type_index == 1 && !self.disk_encryption_password.is_empty() {
            let mut hasher = Sha256::new();
            hasher.update(self.disk_encryption_password.as_bytes());
            let result = hasher.finalize();
            Some(format!("{:x}", result))
        } else { None };
        let disk_encryption = ConfigDiskEncryption {
            encryption_type,
            partition: self.disk_encryption_selected_partition.clone(),
            password_hash,
        };
        let swap = ConfigSwap { enabled: self.swap_enabled };
        let bootloader = ConfigBootloader { kind: match self.bootloader_index { 0 => "systemd-boot".into(), 1 => "grub".into(), 2 => "efistub".into(), _ => "limine".into() } };
        let users: Vec<ConfigUser> = self.users.iter().map(|u: &UserAccount| {
            let mut hasher = Sha256::new();
            hasher.update(u.password.as_bytes());
            let result = hasher.finalize();
            ConfigUser { username: u.username.clone(), password_hash: format!("{:x}", result), is_sudo: u.is_sudo }
        }).collect();
        AppConfig { locales, mirrors, disks, disk_encryption, swap, bootloader, users }
    }

    pub fn save_config(&self) {
        let cfg = self.build_config();
        if let Ok(toml) = toml::to_string_pretty(&cfg) {
            let _ = fs::write(Self::config_path(), toml);
        }
    }

    pub fn load_config(&mut self) -> Result<(), ()> {
        // Ensure option lists are available before mapping names back to indices
        let _ = self.load_locales_options();
        let _ = self.load_mirrors_options();

        let path = Self::config_path();
        let text = std::fs::read_to_string(path).map_err(|_| ())?;
        let cfg: AppConfig = toml::from_str(&text).map_err(|_| ())?;
        self.last_load_missing_sections.clear();

        // Locales
        if !cfg.locales.keyboard_layout.is_empty() {
            if let Some(idx) = self.keyboard_layout_options.iter().position(|s| s == &cfg.locales.keyboard_layout) {
                self.keyboard_layout_index = idx;
            }
        } else { self.last_load_missing_sections.push("Locales: keyboard_layout".into()); }
        if !cfg.locales.locale_language.is_empty() {
            if let Some(idx) = self.locale_language_options.iter().position(|s| s == &cfg.locales.locale_language) {
                self.locale_language_index = idx;
            }
        } else { self.last_load_missing_sections.push("Locales: locale_language".into()); }
        if !cfg.locales.locale_encoding.is_empty() {
            if let Some(idx) = self.locale_encoding_options.iter().position(|s| s == &cfg.locales.locale_encoding) {
                self.locale_encoding_index = idx;
            }
        } else { self.last_load_missing_sections.push("Locales: locale_encoding".into()); }

        // Mirrors (best effort)
        self.mirrors_regions_selected.clear();
        if cfg.mirrors.regions.is_empty() { self.last_load_missing_sections.push("Mirrors: regions".into()); }
        for name in cfg.mirrors.regions {
            if let Some(idx) = self.mirrors_regions_options.iter().position(|s| s == &name) {
                self.mirrors_regions_selected.insert(idx);
            }
        }
        self.optional_repos_selected.clear();
        if cfg.mirrors.optional_repos.is_empty() { self.last_load_missing_sections.push("Mirrors: optional_repos".into()); }
        for name in cfg.mirrors.optional_repos {
            if let Some(idx) = self.optional_repos_options.iter().position(|s| s == &name) {
                self.optional_repos_selected.insert(idx);
            }
        }
        if cfg.mirrors.custom_servers.is_empty() { self.last_load_missing_sections.push("Mirrors: custom_servers".into()); }
        self.mirrors_custom_servers = cfg.mirrors.custom_servers;
        self.custom_repos = cfg
            .mirrors
            .custom_repos
            .into_iter()
            .map(|r| super::CustomRepo {
                name: r.name,
                url: r.url,
                signature: match r.signature.as_str() { "Never" => super::RepoSignature::Never, "Optional" => super::RepoSignature::Optional, _ => super::RepoSignature::Required },
                sign_option: r.sign_option.as_deref().map(|s| if s == "TrustedOnly" { super::RepoSignOption::TrustedOnly } else { super::RepoSignOption::TrustedAll }),
            })
            .collect();
        if self.custom_repos.is_empty() { self.last_load_missing_sections.push("Mirrors: custom_repos".into()); }

        // Disks
        self.load_disks_devices();
        let loaded_selected_device = cfg.disks.selected_device.clone();
        if cfg.disks.mode.is_empty() { self.last_load_missing_sections.push("Disks: mode".into()); }
        self.disks_mode_index = match cfg.disks.mode.as_str() {
            "Best-effort default partition layout" => 0,
            "Manual Partitioning" => 1,
            _ => 2,
        };
        self.disks_selected_device = loaded_selected_device.and_then(|p| {
            if self.disks_devices.iter().any(|d| d.path == p) { Some(p) } else { None }
        });
        if self.disks_selected_device.is_none() { self.last_load_missing_sections.push("Disks: selected_device".into()); }

        // Disk encryption
        if cfg.disk_encryption.encryption_type.is_empty() { self.last_load_missing_sections.push("DiskEncryption: encryption_type".into()); }
        self.disk_encryption_type_index = if cfg.disk_encryption.encryption_type == "LUKS" { 1 } else { 0 };
        self.disk_encryption_selected_partition = cfg.disk_encryption.partition;
        if self.disk_encryption_selected_partition.is_none() { self.last_load_missing_sections.push("DiskEncryption: partition".into()); }
        // do not load plaintext password; hash in config is not converted back

        // Swap
        self.swap_enabled = cfg.swap.enabled;

        // Bootloader
        if cfg.bootloader.kind.is_empty() { self.last_load_missing_sections.push("Bootloader: kind".into()); }
        self.bootloader_index = match cfg.bootloader.kind.as_str() {
            "grub" => 1,
            "efistub" => 2,
            "limine" => 3,
            _ => 0, // systemd-boot default
        };
        // Users
        self.users = cfg.users.into_iter().map(|c| UserAccount { username: c.username, password: String::new(), is_sudo: c.is_sudo }).collect();
        if self.users.is_empty() { self.last_load_missing_sections.push("UserAccount: users".into()); }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigUser {
    pub username: String,
    pub password_hash: String,
    pub is_sudo: bool,
}

pub fn draw_configuration(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Configuration",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    let options = vec![
        ("Save Configuration", 0),
        ("Load Configuration", 1),
    ];

    for (label, idx) in options {
        let is_focused_line = app.config_focus_index == idx;
        let is_active_line = is_focused_line && matches!(app.focus, super::Focus::Content);
        let bullet = if is_focused_line { "▶" } else { " " };
        let bullet_style = if is_active_line { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let label_style = if is_active_line { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::White) };
        let line = Line::from(vec![
            Span::styled(format!("{} ", bullet), bullet_style),
            Span::styled(label.to_string(), label_style),
        ]);
        lines.push(line);
    }

    let continue_style = if app.config_focus_index == 2 && matches!(app.focus, super::Focus::Content) {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[ Continue ]", continue_style)));

    let content = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(match app.focus { super::Focus::Content => " Desicion Menu (focused) ", _ => " Desicion Menu " }))
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}



