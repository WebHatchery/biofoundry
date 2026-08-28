# Biofoundry

Biofoundry is a touch-first colony-management game about building a living
factory underground. Every production line is staffed by hungry creatures,
so food is both the colony's lifeline and its power grid.

The current build contains a broad simulation prototype: colony jobs,
building and digging, food and ore chains, equipment, specialist creatures,
raids, progression, saves, and the Colossal Worm campaign. Development is now
focused on turning those systems into a coherent, learnable game.

## Phase One

Phase One is the first production milestone. It targets a polished 30–45
minute campaign that a new player can start, understand, complete, and recover
from using visible pointer or touch controls alone.

The documentation set is indexed at
[`docs/phase-one/README.md`](docs/phase-one/README.md):

- product vision and audience;
- playable-slice specification;
- prioritized production plan and acceptance criteria;
- playtest and quality plan.

These documents replace the prototype-era design history and completed task
ledger. Runtime facts and balance values remain owned by `assets/data/*.json`.

## Run

```powershell
cargo run
```

## Validate and publish

```powershell
cargo test
cargo clippy --all-targets --all-features -- -D warnings
.\publish.ps1
```

`publish.ps1` is the required project validation path. It builds the Windows
and WebGL releases, packages them, and updates the preview and catalog.

## Project map

- `assets/data/` — data-driven balance, species, buildings, equipment,
  unlocks, and tutorial content.
- `src/simulation/` — deterministic fixed-timestep simulation services.
- `src/state/` — persistent game and world state.
- `src/ui/` — player-facing drawing and UI intents.
- `docs/phase-one/` — current product and production documentation.
- `docs/verification/` — canonical UI verification captures.

Repository rules live in `AGENTS.md` and `CODE_STANDARDS.md`; they are
governance documents rather than game-design documentation.
