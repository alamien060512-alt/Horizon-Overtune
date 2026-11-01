# Overtune for Horizon

A native Horizon port of [Overtune](https://modrinth.com/mod/overtune) — a clean,
Spotify-inspired in-game music player for Minecraft.

## Features

- **In-game music player** — play, pause, skip, shuffle, repeat and seek Minecraft's
  soundtrack without leaving the game.
- **Library browsing** — songs, albums, artists, liked songs and playlists.
- **Custom playlists** — saved locally alongside your settings.
- **Import custom songs** — drop audio files into Overtune's import folder and
  play them alongside Minecraft's original music.
- **Now-playing toast** — a HUD overlay shows the current track.
- **Independent volume** — Overtune's volume is separate from the game's.
- **Event-based music** — custom songs for rain, danger, boss fights, first
  Nether entry, first Elytra flight, credits and more.
- **Vanilla music suppression** — vanilla tracks don't overlap Overtune's.
- **Keybind support** — bind play/pause, skip and other controls.

## Building

This is a native Horizon mod built with [horizon-mixins](https://github.com/horizon-rust/horizon-mixins).
It builds a custom Horizon executable, not a `.hmod` dropped into a running game.

Requirements: Linux, Rust 1.95.0, Python 3.12+, Git and Cargo.

```sh
# Clone Horizon and horizon-mixins beside this mod
git clone https://github.com/horizon-rust/horizon ../horizon
git clone https://github.com/horizon-rust/horizon-mixins ../horizon-mixins
git -C ../horizon-mixins checkout "$(python3 -c 'import json; print(json.load(open("compatibility.json"))["horizon_mixins_revision"])')"

cargo fetch --locked
cargo test --workspace --locked --offline
```

### Stage, check, build

```sh
python3 tools/horizon.py stage --horizon ../horizon
cargo fetch --locked --manifest-path target/mixins/modded/Cargo.toml
python3 tools/horizon.py check --horizon ../horizon
python3 tools/horizon.py build --horizon ../horizon
```

Launch with prepared resources:

```sh
./target/horizon-cargo/fast/horizon-client --resources /absolute/path/to/prepared/resources
```

### Package

```sh
python3 tools/package.py
# Writes target/overtune-0.1.0.hnmod for submission to horizon-mods
```

## Layout

```
Cargo.toml                    Standalone runtime library + audit workspace
compatibility.json            Exact Horizon / horizon-mixins revisions
mixins/overtune/
  horizon-mixin.toml          Module and HEAD hook declarations
  runtime.rs                  Music player logic + unit tests (single source)
  frame.rs                    native_frame hook: music ticking
  hud.rs                      native_hud hook: now-playing overlay
  key.rs                      native_key hook: keybinds
  mods.rs                     native_mods hook: Options > Mods row
  settings.rs                 native_settings hook: settings page buttons
  settings_click.rs           native_settings_click hook: settings clicks
  lifecycle.rs                native_world_lifecycle hook: event-based music
tools/
  horizon.py                  Stage → check → build → test
  package.py                  .hnmod packager
  test_template.py            Tiny fixture test
  test_stage.py               Staging test
proof.py                      In-game proof with horizon-ctl
```

## License

MIT. See [LICENSE](LICENSE).
