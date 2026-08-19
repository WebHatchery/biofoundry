# TODO — Biofoundry

This list records the completed implementation work that was formerly
outstanding, split into tasks suitable for an AI coding agent. Completed
phases, settled design decisions, and playtest-only questions are intentionally
omitted. See [`game_design.md`](game_design.md) for the game as built.

## Creatures and species

### Slime Janitor

- [x] Add waste and spoilage state, configuration, production, storage, and decay.
- [x] Add the Slime Janitor species and its unlock/recruitment path.
- [x] Add feeding trough buildings and janitor behavior for distributing food.
- [x] Integrate trough feeding and waste handling with creature routing, UI, and saves.
- [x] Add simulation, persistence, data, and UI tests for the new food loop.

### Bat Courier

- [x] Add the Bat Courier species, upkeep, carrying capacity, and unlock/recruitment path.
- [x] Add terrain-ignoring movement with valid pickup and drop-off constraints.
- [x] Include bats in hauling priorities and food/industry load-shedding.
- [x] Add courier status/readouts, save support, and routing/throughput tests.

### Goblin Engineer

- [x] Add the missing Engineer species data, progression threshold, and Breeding Pit action.
- [x] Add Engineer job modifiers and auto-assignment/equipment compatibility.
- [x] Add breeding feedback, inspection text, and status presentation.
- [x] Add simulation, data, persistence, and UI tests for the evolution tier.

## Colony pressure

### Morale and overcrowding

- [x] Add persisted morale state and balance configuration for each creature.
- [x] Calculate overcrowding from population and usable warren capacity.
- [x] Apply morale and overcrowding to work speed, job behavior, and desertion pressure.
- [x] Add HUD/status indicators and clear recovery behavior.
- [x] Add deterministic simulation, save/load, and UI coverage.

### Food variety

- [x] Track raw ingredients and cooked food as separate stockpile resources.
- [x] Add raw/cooked recipe multipliers and configurable spoilage timers.
- [x] Update hauling, cooking, consumption, economy meters, and save compatibility.
- [x] Add UI/status feedback for freshness and the resulting production trade-offs.
- [x] Add economy, persistence, and regression tests for both food paths.

## Endgame

### Multi-outpost worm transit

- [x] Add outpost data, placement rules, storage, and activation state.
- [x] Add worm transit endpoints and route validation between the shrine and outposts.
- [x] Implement creature and cargo transfer, transit timing, and failure/recovery states.
- [x] Add touch controls, transit status, inspection text, and save/load support.
- [x] Add deterministic simulation and UI tests for local and remote logistics.

### Worm Shrine pause-feeding toggle

- [x] Add a persisted pause/resume flag to the Worm Shrine state.
- [x] Add a visible, tappable inspection control and paused status indicator.
- [x] Make shrine feeding honor the toggle while retaining the food-reserve guard.
- [x] Add simulation, persistence, and UI tests for pausing and resuming offerings.

### Mixed Worm Shrine offerings

- [x] Add configurable food-and-ingot offering costs and reservation rules.
- [x] Update shrine accounting, progress, and awakening conditions for both resources.
- [x] Update economy meters, inspection text, and campaign feedback.
- [x] Add economy, save/load, and regression tests for partial and completed offerings.
