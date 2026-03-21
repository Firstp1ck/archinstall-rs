use std::path::{Path, PathBuf};

const EXAMPLES_SUBDIR: &str = "configs/examples";

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

/// Extract a human-readable label from a `.toml` preset file.
/// Uses the first `# ...` comment line if present, otherwise the file stem.
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

/// Discover example presets and return `(label, path)` pairs sorted by label.
pub fn list_example_presets() -> Vec<(String, PathBuf)> {
    let roots = discover_example_roots();
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut presets: Vec<(String, PathBuf)> = Vec::new();
    for root in roots {
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "toml") && p.is_file() {
                    let canonical = p.canonicalize().unwrap_or_else(|_| p.clone());
                    if !seen.contains(&canonical) {
                        let label = label_for_preset(&p);
                        presets.push((label, p));
                        seen.push(canonical);
                    }
                }
            }
        }
    }
    presets.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    presets
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

        // Override env so the function finds our temp dir
        // SAFETY: test is single-threaded; no other thread reads this var concurrently.
        unsafe {
            std::env::set_var("ARCHINSTALL_RS_EXAMPLES", examples.to_str().unwrap());
        }
        let presets = list_example_presets();
        unsafe {
            std::env::remove_var("ARCHINSTALL_RS_EXAMPLES");
        }

        assert!(
            presets.iter().any(|(label, _)| label == "My test preset"),
            "expected to find test preset, got: {presets:?}"
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
