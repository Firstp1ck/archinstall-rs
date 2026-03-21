<!-- Thank you for contributing to **archinstall-rs**!

**References:**
- [CONTRIBUTING.md](../CONTRIBUTING.md) — contribution guidelines
- [README.md](../README.md) — features, roadmap, usage
- [Documents/arch_manual.md](../Documents/arch_manual.md) — installer-oriented notes (when applicable)

Please skim these before submitting your PR.
-->

## Summary
Briefly describe the problem and how your change solves it. (bullets welcome)

## Type of change
- [ ] feat (new feature)
- [ ] fix (bug fix)
- [ ] docs (documentation only)
- [ ] refactor (no functional change)
- [ ] perf (performance)
- [ ] test (add/update tests)
- [ ] chore (build/infra/CI)
- [ ] ui (visual/interaction changes in the TUI)
- [ ] breaking change (incompatible behavior or config)

## Related issues
Closes #

## How to test
Exact steps and commands. Prefer **`--dry-run`** when exercising install flows.

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test -- --test-threads=1
cargo run -- --dry-run
```

On a live Arch ISO (when your change touches partitioning, mounts, or install): describe the disk layout and menu path you used.

## Screenshots / recordings (if UI changes)
Before/after terminal screenshots or a short recording. Update `Images/` if you add promotional shots.

## Checklist

**Code quality**
- [ ] `cargo check` succeeds
- [ ] `cargo fmt --all` produces no diff
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` is clean
- [ ] `cargo test -- --test-threads=1` passes
- [ ] New public API or non-obvious behavior has rustdoc where it helps reviewers

**Testing**
- [ ] Added or updated tests when behavior is non-trivial (storage planner, command redaction, parsing, etc.)
- [ ] For bug fixes: reproduced first, then verified the fix (ideally with a test)

**Documentation**
- [ ] Updated [README.md](../README.md) if user-visible behavior, options, or roadmap claims changed
- [ ] Updated [Documents/](../Documents/) (e.g. `arch_manual.md`, plans) if installer steps or constraints changed

**Installer safety**
- [ ] Respects **`--dry-run`** where applicable (no destructive commands without user confirmation)
- [ ] Partitioning / LUKS / mount / `pacstrap` paths reviewed for wrong-device or ordering mistakes
- [ ] No intentional breaking changes — or they are called out under **Breaking changes** below

**Scope**
- [ ] Not a third-party packaging-only change (AUR/distros); those belong in their respective packaging repos

## Notes for reviewers
Tricky areas, assumptions, edge cases, or follow-ups.

## Breaking changes
Migration steps (e.g. config TOML keys, CLI flags, default behavior).

## Additional context
Logs (redact secrets), discussion links, design notes.
