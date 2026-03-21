use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const EXAMPLES_SUBDIR: &str = "configs/examples";

#[derive(Clone, Debug)]
pub struct ConfigPresetTableRow {
    pub path: PathBuf,
    pub country: String,
    pub language: String,
    pub desktop: String,
    pub additional: String,
}

#[derive(Deserialize)]
struct ManifestRoot {
    preset: Vec<ManifestPreset>,
}

#[derive(Deserialize)]
struct ManifestPreset {
    file: String,
    country: String,
    language: String,
    desktop: String,
    additional: String,
}

/// Return unique directories that may contain example `.toml` presets.
/// Precedence: env overrides, exe-relative, cwd ancestor walk.
fn discover_example_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut try_add = |p: PathBuf| {
        if p.is_dir() && !roots.contains(&p) {
            roots.push(p);
        }
    };

    if let Ok(val) = std::env::var("ARCHINSTALL_RS_EXAMPLES") {
        try_add(PathBuf::from(val));
    }

    if let Ok(val) = std::env::var("ARCHINSTALL_RS_REPO") {
        try_add(PathBuf::from(val).join(EXAMPLES_SUBDIR));
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        try_add(parent.join(EXAMPLES_SUBDIR));
    }

    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join(EXAMPLES_SUBDIR);
            if candidate.is_dir() {
                try_add(candidate);
                break;
            }
            match dir.parent() {
                Some(p) if p != dir => dir = p,
                _ => break,
            }
        }
    }

    roots
}

fn load_manifest_maps(roots: &[PathBuf]) -> HashMap<String, ManifestPreset> {
    let mut map: HashMap<String, ManifestPreset> = HashMap::new();
    for root in roots {
        let mf = root.join("manifest.toml");
        if !mf.is_file() {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&mf)
            && let Ok(parsed) = toml::from_str::<ManifestRoot>(&text)
        {
            for p in parsed.preset {
                map.entry(p.file.clone()).or_insert(p);
            }
        }
    }
    map
}

/// Extract a human-readable fallback description from a `.toml` preset file.
fn label_for_preset(path: &Path) -> String {
    if let Ok(text) = std::fs::read_to_string(path)
        && let Some(line) = text.lines().next()
    {
        let trimmed = line.trim();
        if let Some(comment) = trimmed.strip_prefix('#') {
            let label = comment.trim();
            if !label.is_empty() {
                let capped = if label.len() > 80 {
                    &label[..80]
                } else {
                    label
                };
                return capped.to_string();
            }
        }
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Discover example presets with table metadata from `manifest.toml` when present.
pub fn list_example_preset_rows() -> Vec<ConfigPresetTableRow> {
    let roots = discover_example_roots();
    let manifest = load_manifest_maps(&roots);
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut rows: Vec<ConfigPresetTableRow> = Vec::new();

    for root in roots {
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if !p.is_file() {
                    continue;
                }
                if p.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                if p.file_name().is_some_and(|n| n == "manifest.toml") {
                    continue;
                }
                let canonical = p.canonicalize().unwrap_or_else(|_| p.clone());
                if seen.contains(&canonical) {
                    continue;
                }
                seen.push(canonical);

                let fname = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let row = if let Some(m) = manifest.get(&fname) {
                    ConfigPresetTableRow {
                        path: p.clone(),
                        country: m.country.clone(),
                        language: m.language.clone(),
                        desktop: m.desktop.clone(),
                        additional: m.additional.clone(),
                    }
                } else {
                    let add = label_for_preset(&p);
                    ConfigPresetTableRow {
                        path: p,
                        country: "—".into(),
                        language: "—".into(),
                        desktop: "—".into(),
                        additional: add,
                    }
                };
                rows.push(row);
            }
        }
    }

    rows.sort_by(|a, b| {
        a.country
            .to_lowercase()
            .cmp(&b.country.to_lowercase())
            .then_with(|| a.desktop.to_lowercase().cmp(&b.desktop.to_lowercase()))
            .then_with(|| {
                a.path
                    .file_name()
                    .unwrap_or_default()
                    .cmp(b.path.file_name().unwrap_or_default())
            })
    });
    rows
}

/// Search text for the popup filter (all columns, lowercased match).
pub fn search_blob_for_row(row: &ConfigPresetTableRow) -> String {
    format!(
        "{} {} {} {}",
        row.country, row.language, row.desktop, row.additional
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discover_from_temp_tree() {
        let dir = tempfile::tempdir().unwrap();
        let examples = dir.path().join("configs/examples");
        fs::create_dir_all(&examples).unwrap();
        fs::write(
            examples.join("test-preset.toml"),
            "# My test preset\n[locales]\n",
        )
        .unwrap();

        // SAFETY: test is single-threaded; no other thread reads this var concurrently.
        unsafe {
            std::env::set_var("ARCHINSTALL_RS_EXAMPLES", examples.to_str().unwrap());
        }
        let rows = list_example_preset_rows();
        unsafe {
            std::env::remove_var("ARCHINSTALL_RS_EXAMPLES");
        }

        assert!(
            rows.iter().any(|r| r.additional == "My test preset"),
            "expected fallback additional from comment, got: {rows:?}"
        );
    }

    #[test]
    fn label_falls_back_to_stem() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("no-comment.toml");
        fs::write(&f, "[locales]\nkeyboard_layout = \"us\"\n").unwrap();
        assert_eq!(label_for_preset(&f), "no-comment");
    }
}
