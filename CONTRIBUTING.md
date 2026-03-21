# Contributing to archinstall-rs

Thank you for your interest in contributing! This document explains how to set up your environment, the development workflow, coding standards, and how to submit changes. By participating, you agree to uphold our Code of Conduct.

- See `CODE_OF_CONDUCT.md` for expected behavior in all project spaces.

## Getting Started

### Prerequisites
- Rust 1.70 or later (2024 edition)
- Cargo package manager
- Recommended: `rustup`, `rustfmt`, `clippy`

### Clone and Build
```bash
# Fork and clone your fork
git clone https://github.com/<your-username>/archinstall-rs.git
cd archinstall-rs

# Add upstream remote (optional but recommended)
git remote add upstream https://github.com/<upstream-owner>/archinstall-rs.git

# Debug build
cargo build

# Run
cargo run
```

## Development Workflow

### Branching
Create topic branches off `main`:
- feature/<short-description> for features
- fix/<short-description> for bug fixes
- chore/<short-description> for non-functional changes

Keep branches focused and small. Rebase on top of the latest `main` when needed:
```bash
git fetch upstream
git rebase upstream/main
```

### Commit Messages
Use Conventional Commits where possible:
- feat: add disks screen navigation improvements
- fix: correct partition plan free-space threshold
- docs: expand README build instructions
- refactor: extract popup rendering helpers
- test: add config load/save roundtrip tests

### Code Style and Quality
From project documentation conventions, expanded:
1. Follow Rust best practices and idioms
2. Maintain the existing code style
3. Add tests for new functionality where practical
4. Update documentation as needed (`README.md`, `Documents/`, and inline docs)
5. Keep commits atomic and well-described
6. Ensure all tests pass before submitting PR

Run the standard checks locally:
```bash
# Compile quick sanity
cargo check

# Test suite
cargo test

# Format (CI usually enforces this)
cargo fmt --all --check

# Lint (treat warnings as errors locally)
cargo clippy -- -D warnings
```

### Running the App During Development
```bash
# Build and run in debug
cargo run
```
The app is a TUI; see [Documents/USAGE.md](Documents/USAGE.md) for navigation and flow.

## Architecture Primer (Where to Make Changes)
High-level structure (full tree and feature checklist: [Documents/DEVELOPMENT.md](Documents/DEVELOPMENT.md)):
- `src/main.rs`: entry point
- `src/app/`: installation sections and install flow (`install/`, `config/`, screen modules)
- `src/input/`: input handling (screens, popups, command line)
- `src/render/`: section and popup rendering, theme
- `src/common/`: shared UI utilities and popups
- `src/core/`: state, types, storage planner, services

### Adding a New Screen or Feature (UI)
Adapted from [Documents/DEVELOPMENT.md](Documents/DEVELOPMENT.md) “Adding a new feature”:
1. Add `Screen::MyFeature` (and `PopupKind` if needed) in `src/core/types.rs`.
2. Register menu entries and state in `src/core/state.rs`.
3. Add `src/app/my_feature.rs` (draw) and route it in `src/render/sections/content.rs`.
4. Add input under `src/input/screens/`, export in `mod.rs`, wire `dispatcher.rs`.
5. Optional: popups in `src/common/popups.rs` and `src/render/popup/`.
6. Optional: config in `src/app/config/{types,io,view}.rs`.
7. Optional: install steps in `src/core/services/` and `src/app/install/flow.rs`.
8. Add tests and update `Documents/` when user-facing behavior changes.

### Non-UI/Logic Changes
- Configuration I/O: `src/app/config/`.
- Global state: `src/core/state.rs`; shared types: `src/core/types.rs`.
- Reusable UI helpers: `src/common/`.

## Tests
Tests are encouraged for:
- Config serialization/deserialization and defaults
- Non-UI logic (e.g., command planning, validation)
- Small units around input handling functions

Run tests with:
```bash
cargo test
```

## Documentation
- Update `README.md` and the relevant file under `Documents/` when user-facing behavior changes
- Prefer concise inline documentation where code is non-obvious
- Add screenshots under `Images/` if you change TUI flows significantly

## Pull Requests
Before opening a PR:
- Rebase onto the latest `main`
- Ensure `cargo test`, `cargo fmt --all --check`, and `cargo clippy` pass locally
- Include a clear description, motivation, and scope
- Add screenshots/GIFs for UI changes where useful
- Note any breaking changes or migration steps

PR review expectations:
- Maintainers may request changes for clarity, safety, or scope
- Small, focused PRs are reviewed faster

## Issues and Bug Reports
From [Documents/TROUBLESHOOTING.md](Documents/TROUBLESHOOTING.md), expanded:
When reporting a bug, include:
- Clear problem description and expected vs. actual behavior
- Steps to reproduce
- System information (live ISO version, firmware type, hardware summary)
- Relevant configuration files (redact secrets)
- Logs or error messages if available

For feature requests:
- Describe the use case and why it’s valuable
- Outline a proposed UX (screen placement, inputs)
- Consider how it interacts with existing configuration and installation flow

## Security
Please do not open public issues for security vulnerabilities. Report privately to: `firstpick1992@proton.me`.

## Code of Conduct
This project follows the Contributor Covenant. See `CODE_OF_CONDUCT.md`.

## License
By contributing, you agree that your contributions will be licensed under the MIT License (see `LICENSE`).

---
Made with ❤️ and 🦀 for the Arch Linux community.
