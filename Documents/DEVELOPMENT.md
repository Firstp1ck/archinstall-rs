# Development

## Building

```bash
cargo build      # debug (faster iteration)
cargo run        # run debug build
cargo check      # quick compile check
cargo test
cargo fmt
cargo clippy
```

## Project layout

```
archinstall-rs/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs
│   ├── runner.rs               # TUI run loop and helpers
│   ├── app/                    # Installation sections and install flow
│   │   ├── install/            # Flow builder and install UI
│   │   ├── config/             # TOML save/load/types and summary view
│   │   └── *.rs                # Screens (disks, bootloader, etc.)
│   ├── common/                 # Shared UI utilities and popups
│   ├── core/                   # Core plumbing and services
│   │   ├── services/           # partitioning, mounting, bootloader, ...
│   │   ├── storage/            # StoragePlan / planner (partition + mount + fstab)
│   │   └── state.rs            # Global app state
│   ├── input/                  # Input handling (screens, popups, cmdline)
│   └── render/                 # Rendering (sections, popups, theme)
├── assets/
│   └── limine/                 # Limine assets (experimental)
├── boot.sh                     # Minimal GUI bootstrap helper
├── run-tui.sh                  # Wrapper to launch prebuilt binary in a terminal
├── archinstall-rs.config.toml  # Example configuration (dev machine snapshot)
├── configs/examples/           # Portable TOML presets (locales + popular stacks)
├── Cargo.toml
└── README.md
```

## Principles

- **Modularity**: Each installation section is its own module where practical.
- **Separation of concerns**: UI, orchestration, and data stay distinct.
- **Type safety**: Prefer encoding invariants in types.
- **Errors**: Surface actionable messages in the TUI where possible.

## Adding a new feature (checklist)

1. **Screen type**  
   Add a variant to `Screen` in `src/core/types.rs` (and a `PopupKind::...` variant if you add popups).

2. **Menu and state**  
   In `src/core/state.rs`, add a `MenuEntry` for the new screen in `AppState::new()` and any per-screen fields on `AppState`.

3. **Renderer**  
   Add `src/app/my_feature.rs` with something like `pub fn draw_my_feature(frame: &mut Frame, app: &mut AppState, area: Rect)` and route it from `src/render/sections/content.rs`.

4. **Info panel (optional)**  
   Extend `src/render/sections/info.rs` for summary text when the screen is active.

5. **Input**  
   Add `src/input/screens/my_feature.rs` with handlers (`move_*`, `change_*`, `handle_enter_*`), export in `src/input/screens/mod.rs`, and wire `src/input/screens/dispatcher.rs` (`move_screen_up/down`, `change_value`, `handle_enter`).

6. **Popups (optional)**  
   Add helpers in `src/common/popups.rs` and handle selections in `apply_popup_selection`. Prefer `src/render/popup/general.rs` or add a focused renderer under `src/render/popup/`.

7. **Persistence (optional)**  
   Extend `src/app/config/types.rs`, `src/app/config/io.rs`, and `src/app/config/view.rs` for save/load and summary.

8. **Install plan (optional)**  
   Add a service under `src/core/services/` if system changes are required, and extend `src/app/install/flow.rs` (`build_install_plan`) at the right stage.

9. **Tests and docs**  
   Add tests where feasible and update user-facing docs under `Documents/` when behavior changes.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for workflow, PR expectations, and code style.
