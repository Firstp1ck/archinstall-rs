# pr-update

For **archinstall-rs** (Rust TUI Arch installer), from the repo root:

1. `git fetch origin` and diff **against `main`**: `git log origin/main..HEAD --oneline` and `git diff origin/main...HEAD --stat` (use `main` if that is your default base).
2. Update **`dev/PR/<current-branch>.md`** (`git branch --show-current`): append only **new** bullets/commits not already reflected there.
3. Keep additions **short, concise, clear**. **Do not remove** existing PR file content.

Focus areas when summarizing: `src/app/`, `src/core/` (storage, partitioning, mount, install flow), `src/input/`, `src/render/`, `tests/`, `Documents/`.
