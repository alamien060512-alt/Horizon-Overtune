# Overtune for Horizon

A native [Horizon](https://github.com/horizon-rust/horizon) port of
[Overtune](https://modrinth.com/mod/overtune): a Spotify-style music player for
Minecraft's soundtrack. It is a native mod, so it builds into a custom Horizon
executable (see below); it is not a `.hmod` for a running game.

Built from [horizon-native-mod-template](https://github.com/horizon-rust/horizon-native-mod-template).

## What works

| Overtune feature | In this port |
|---|---|
| Play, pause, skip, previous | Yes: keys, `/overtune`, chat feedback |
| Shuffle and repeat (off / all / one) | Yes |
| Library: songs, albums, liked songs | Yes: fixed list of the game's music events and discs; albums are Soundtrack, Nether and Music Discs |
| Likes | Yes, kept between sessions |
| Independent volume | Yes (0-100% in steps), sent to the game's music channel |
| Now-playing toast | Yes, bottom-right HUD box when the song changes or you press a key |
| Vanilla music suppression | Yes while Overtune is on |
| Settings page and rebindable keys | Yes: Options > Mods > Overtune |
| Custom playlists, importing audio files, event-based songs, seek bar, artist pages | **Not ported.** They need an audio file loader and screens that Horizon's native hook points do not offer |

Honest limits:

- **Pause restarts the song.** Horizon's sound engine can stop an instance but not pause one, so resuming starts the song again.
- **The library is a fixed list** of sound events from vanilla's `sounds.json` (the engine cannot list its registry). `Soundtrack` entries such as "Overworld" are the game's own event, which plays one of its random tracks. An event your resources lack fails to start and is skipped; after three failures in a row Overtune pauses itself.
- **Vanilla music may blip.** Vanilla's music manager is stopped on the next tick after it starts a track.
- **Nothing here has been run in a game.** See [VALIDATION.md](VALIDATION.md).

## Controls

Defaults (rebind in Options > Mods > Overtune):

| Key | Action |
|---|---|
| F7 | Play / pause |
| F6 / F8 | Previous / next song |
| F9 | Toggle shuffle |
| F10 | Like song |
| Page Up / Page Down | Volume up / down |

Chat: `/overtune [play|next|prev|shuffle|repeat|like|volume <0-100>|list|select <n>|scope|toast|on|off|status]`.
`list` shows the current library with numbers; `select 3` plays the third.
`scope` cycles All songs, Liked songs and each album.

## Layout

```text
Cargo.toml                          Standalone runtime library + source audit workspace
compatibility.json                  Exact Horizon / horizon-mixins revisions and source hashes
mixins/overtune/
  horizon-mixin.toml                Module and seven HEAD hooks
  runtime.rs                        Player, library, settings, commands, toast layout + tests
  tick.rs      native_tick          Vanilla suppression and playback
  hud.rs       native_hud           Now-playing toast
  key.rs       native_key           Keys
  command.rs   native_command       /overtune
  mods.rs      native_mods          Options > Mods row and keys
  settings.rs, settings_click.rs    Settings page
tools/                              Stage / check / build / test, packaging, audits
```

The adapters are thin; the logic is in `runtime.rs`, which Cargo tests alone
and Horizon also compiles as the module `overtune` in `game.rs`.

## Build

Requirements: Linux, Rust 1.95.0, Python 3.12+, Git and Cargo.

```sh
git clone https://github.com/horizon-rust/horizon ../horizon
git clone https://github.com/horizon-rust/horizon-mixins ../horizon-mixins
git -C ../horizon-mixins checkout "$(python3 -c 'import json; print(json.load(open("compatibility.json"))["horizon_mixins_revision"])')"

python3 tools/horizon.py stage --horizon ../horizon
cargo fetch --locked --manifest-path target/mixins/modded/Cargo.toml
python3 tools/horizon.py check --horizon ../horizon
python3 tools/horizon.py build --horizon ../horizon
./target/horizon-cargo/fast/horizon-client --resources /absolute/path/to/prepared/resources
```

Pinned to Horizon `ba6344a` (Nightly 2026-10-30) and horizon-mixins `d9c83d7`
(0.6.0). Package for the [horizon-mods](https://github.com/horizon-rust/horizon-mods)
registry with `python3 tools/package.py` (`target/overtune-0.1.0.hnmod`).

## Credits and license

Overtune is by its original authors; this is an independent port of its feature
set and contains none of its code or assets. Build tooling is adapted from the
MIT-licensed Horizon native mod template. MIT, see [LICENSE](LICENSE).
