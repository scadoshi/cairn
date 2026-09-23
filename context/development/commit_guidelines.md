# Commit Guidelines

- Concise, one-line messages (multi-line only when many changes)
- Group related files logically
- No emojis
- Use `git diff` to understand changes before committing
- **Never** include AI-agent signatures in commits
    - No "Co-Authored-By: Claude..."
    - No "Generated with [Claude Code]..."
    - No "Written with the help of ..."
- Never push without being asked

## CI: how your commits get checked (run these BEFORE you push)

`.github/workflows/ci.yml` runs `test` and `lint` on every push to `main` and
every pull request, on macOS runners (cairn ships to macOS and iOS, and the
default feature pulls dioxus-desktop). Reproduce the gate locally first.

### 1. Format with nightly

`rustfmt.toml` enables `imports_granularity = "Crate"`, an unstable option, so
stable `cargo fmt` silently skips it: your code looks formatted locally but
fails CI.

```bash
cargo +nightly fmt        # not `cargo fmt`; stable can't apply the Crate imports rule
```

### 2. Clippy, warnings are errors

```bash
cargo clippy --all-targets -- -D warnings
```

Lint levels live in `Cargo.toml` `[lints.clippy]`. The panic family (`unwrap`,
`expect`, `panic`, indexing, slicing) is denied outside tests; `clippy.toml`
allows it inside them.

### 3. Hook order

```bash
dx check
```

Not in CI, because it needs the Dioxus CLI installed on the runner. Run it locally before pushing UI changes. It catches the one class of bug neither clippy nor the tests can see: a hook called conditionally, in a loop, or outside a component body. Dioxus panics at runtime on a hook count that changes between renders, which takes the whole screen down with "Unable to retrieve the hook that was initialized at this index". That shipped to a phone once already.

### 4. Tests

```bash
cargo test                # domain tests, offline, no env needed
```
