# Validation

Validated on Linux with Rust 1.95.0, using the exact revisions in
`compatibility.json`.

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Pass |
| `cargo test --workspace --locked --offline` | Runtime unit tests pass |
| `python3 tools/test_template.py` | Bundle compiles in the tiny fixture; original method calls/result preserved, shared runtime, AST audit, unchanged bytes/mtimes pass |

These are compilation/test checks, not interactive gameplay checks. No game was
launched and no Minecraft resources or player data were read or changed.
