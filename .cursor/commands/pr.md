# pr

Create a PR description for **archinstall-rs** using `@.github/PULL_REQUEST_TEMPLATE.md`.

- Fill every relevant section; use **archinstall-rs** naming (binary `archinstall-rs`, not other projects).
- Testing examples: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo run -- --dry-run` on a live ISO when applicable.
- Save the completed description to **`dev/PR/<branch-name>.md`** where `<branch-name>` is `git branch --show-current`.
