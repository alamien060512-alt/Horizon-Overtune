# Validation

Linux, Rust 1.95.0, Horizon `ba6344a59cb9e755b730dcf02285fc0d33b97d3c`,
horizon-mixins `d9c83d7096b7ea5b83678da2efdfd6161f74997e`.

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Pass |
| `cargo test --workspace --locked --offline` | 17 runtime tests pass |
| `python3 tools/test_template.py` | Pass (stand-in fixture: seven hooks compile, original body preserved, AST audit) |
| `python3 tools/horizon.py stage` on the pinned export | Pass (plan/apply of all seven hooks) |
| `python3 tools/horizon.py check` / `test` / `build` on the pinned export | **Not completed**: not yet confirmed |
| `python3 tools/test_stage.py` | **Not run** |

No game was launched. Sound event names are unchecked against real resources,
and playback, the HUD toast and keys are untested in game.
