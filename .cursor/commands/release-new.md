# release-new

Create **release notes** for **archinstall-rs** for the given version.

- Output file: **`Documents/RELEASE_v{version}.md`** (e.g. `RELEASE_v0.2.0.md`).
- Infer **`{version}`** from `Cargo.toml` / user input; compare changes since the **previous** tag or release notes file.
- User-friendly, **short**, bullets for fixes/features; mention risky areas (partitioning, encryption, bootloader) if touched.
- Suggest verify steps: `cargo build --release`, smoke test `cargo run -- --dry-run` on ISO when relevant.
