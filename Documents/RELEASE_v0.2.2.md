## archinstall-rs v0.2.2

Release date: 2026-03-21

### Highlights
- **Load configuration**: Presets are listed in a **searchable table** (country, language, desktop, notes) driven by `configs/examples/manifest.toml`, with discovery from env overrides, the directory next to the binary, and cwd ancestors.
- **Examples & releases**: More **locale** and **popular** example TOMLs under `configs/examples/`; CI still ships **`config-examples.tar.gz`** (and the install script unpacks it). Many refreshed examples use **GRUB** and document **AUR** helpers where relevant—always confirm bootloader and packages match your hardware before installing.
- **Install log**: Child **stdout** is parsed with **carriage-return** handling so spinner-style progress **updates in place** instead of flooding the log.

### Improvements & fixes
- **Config load**: Missing `root_password_hash` no longer triggers a bogus “missing section” warning; set the root password in the TUI when you rely on plaintext flow.
- **Docs**: README slimmed toward pointers; configuration, install, usage, and development docs expanded (preset layout, mirrors, desktop packages).

### Risky / review before relying on it
- **Install pipeline** (`flow` + stdout pumping) changed; do a **real or VM install** after upgrading, not only dry-run, if you care about long-running command output.
- **Preset content** (bootloader, AUR, disk hints) is **opinionated**; loading a preset does not replace double-checking **partitioning**, **encryption**, and **bootloader** choices in the TUI.

### Breaking changes
- None intended for normal TUI users. Paths and labels for bundled examples moved under `configs/examples/`; old “load from cwd only” behavior is replaced by the preset popup plus optional local `archinstall-rs.config.toml`.

---

**Before tagging v0.2.2**: set `version = "0.2.2"` in `Cargo.toml` and update `Cargo.lock` if needed.

**Suggested verification**
- `cargo build --release`
- `cargo test`
- On an Arch ISO or test VM: `cargo run -- --dry-run` (and a short non-destructive flow) after loading a preset you care about
