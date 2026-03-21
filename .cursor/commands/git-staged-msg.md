# git-staged-msg

For **archinstall-rs**, create/update commit messages for **currently staged** changes.

- Write **`dev/COMMIT/SHORT.md`** (one-line subject) and **`dev/COMMIT/LONG.md`** (body with bullets), or update them in place.
- Use Conventional Commits prefix: `fix:`, `feat:`, `change:`, `perf:`, `test:`, `chore:`, `refactor:`, `docs:`, `style:`, `build:`, `ci:`, `revert:`.

**Long form** — one bullet per meaningful change, same type prefixes where helpful:

```
<type>: <short summary>
- <type>: point description, short and clear.
- <type>: point description, short and clear.
```

No extra prose outside that structure in the long file unless the user asks.
