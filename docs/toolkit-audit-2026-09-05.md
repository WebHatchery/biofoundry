# Toolkit audit — 5 September 2026

The catalogue review reported no specific Biofoundry migration. Current source inspection confirms shared implementations for the reviewed responsibilities:

- `src/data.rs`: labeled embedded JSON and DataRegistry content loading.
- `src/game/persistence.rs`: shared versioned slots, migration callback, backup, restore and quarantine. Typed session validation and schema conversion stay game-owned.
- `src/game.rs` and `src/game/input.rs`: toolkit Camera2D, camera bounds and TouchGesture; game input ownership stays local.
- `src/state/world.rs` and `src/state.rs`: shared grids and SeededRng.
- `src/audio.rs`: SoundManager and shared settings/persistence.
- `src/ui/hud/widgets.rs`: toolkit surfaces, measured centered text, pointer and touch targets.

The last word of a building name is intentionally used as a compact control label, not as a wrapping algorithm. Diagnostic text-file output is an audit artifact, not save/content loading. No local generic JSON, wrapping, RNG or particle implementation requiring replacement was found in the inspected production sources.

Fixed an unrelated strict-Clippy finding by passing the existing HudOptions to the command strip instead of splitting its fields into extra arguments.

Validation: 517 passing tests/checks; formatting; all-target/all-feature Clippy with warnings denied; all project Rust files at most 800 lines; default `publish.ps1` Windows and WebGL packaging, seven registered assets, Preview deployment and Project Roost tracking passed.
