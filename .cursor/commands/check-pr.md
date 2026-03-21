# check-pr

Review the **last commits** from the specified author (or the PR branch vs **`main`**) for **archinstall-rs**.

1. Use `git log` / `git show` scoped to the PR (e.g. vs `origin/main` or `main`).
2. Explain **what changed** and **why** in plain terms; call out installer-risk areas: partitioning, LUKS, mounts, `pacstrap`, fstab, bootloader, config I/O.
3. If you spot **critical logic errors** (data loss risk, wrong device targets, unsafe commands), explain and suggest **concrete fixes** with file/module references under `src/`.
