# TODO — Biofoundry

This list contains outstanding implementation work, split into tasks suitable
for an AI coding agent. Completed phases, settled design decisions, and
playtest-only questions are intentionally omitted. See
[`game_design.md`](game_design.md) for the game as built.

## Creatures and species

### Slime Janitor

- [ ] Add waste and spoilage state, configuration, production, storage, and decay.
- [ ] Add the Slime Janitor species and its unlock/recruitment path.
- [ ] Add feeding trough buildings and janitor behavior for distributing food.
- [ ] Integrate trough feeding and waste handling with creature routing, UI, and saves.
- [ ] Add simulation, persistence, data, and UI tests for the new food loop.

### Bat Courier

- [ ] Add the Bat Courier species, upkeep, carrying capacity, and unlock/recruitment path.
- [ ] Add terrain-ignoring movement with valid pickup and drop-off constraints.
- [ ] Include bats in hauling priorities and food/industry load-shedding.
- [ ] Add courier status/readouts, save support, and routing/throughput tests.

### Goblin Engineer

- [ ] Add the missing Engineer species data, progression threshold, and Breeding Pit action.
- [ ] Add Engineer job modifiers and auto-assignment/equipment compatibility.
- [ ] Add breeding feedback, inspection text, and status presentation.
- [ ] Add simulation, data, persistence, and UI tests for the evolution tier.

## Colony pressure

### Morale and overcrowding

- [ ] Add persisted morale state and balance configuration for each creature.
- [ ] Calculate overcrowding from population and usable warren capacity.
- [ ] Apply morale and overcrowding to work speed, job behavior, and desertion pressure.
- [ ] Add HUD/status indicators and clear recovery behavior.
- [ ] Add deterministic simulation, save/load, and UI coverage.

### Food variety

- [ ] Track raw ingredients and cooked food as separate stockpile resources.
- [ ] Add raw/cooked recipe multipliers and configurable spoilage timers.
- [ ] Update hauling, cooking, consumption, economy meters, and save compatibility.
- [ ] Add UI/status feedback for freshness and the resulting production trade-offs.
- [ ] Add economy, persistence, and regression tests for both food paths.

## Endgame

### Multi-outpost worm transit

- [ ] Add outpost data, placement rules, storage, and activation state.
- [ ] Add worm transit endpoints and route validation between the shrine and outposts.
- [ ] Implement creature and cargo transfer, transit timing, and failure/recovery states.
- [ ] Add touch controls, transit status, inspection text, and save/load support.
- [ ] Add deterministic simulation and UI tests for local and remote logistics.

### Worm Shrine pause-feeding toggle

- [ ] Add a persisted pause/resume flag to the Worm Shrine state.
- [ ] Add a visible, tappable inspection control and paused status indicator.
- [ ] Make shrine feeding honor the toggle while retaining the food-reserve guard.
- [ ] Add simulation, persistence, and UI tests for pausing and resuming offerings.

### Mixed Worm Shrine offerings

- [ ] Add configurable food-and-ingot offering costs and reservation rules.
- [ ] Update shrine accounting, progress, and awakening conditions for both resources.
- [ ] Update economy meters, inspection text, and campaign feedback.
- [ ] Add economy, save/load, and regression tests for partial and completed offerings.
