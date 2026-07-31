# TODO — Biofoundry

Deferred from the original and automation design plans, in rough order of
design interest. See [`game_design.md`](game_design.md) for the game as built.

## Creatures and species

- **Slime Janitor** — waste and spoilage as a resource sink, with feeding troughs as distributed food access. Deepens the logistics-vs-abstraction question the food grid abstracts away today.
- **Bat Courier** — terrain-ignoring hauling as counter-play against warren topology.
- **Goblin Engineer** — the middle tier of the evolution line; hobgoblin and overseer shipped, engineer did not.

## Colony pressure

- **Morale and overcrowding** — the second colony-sim pressure axis beyond hunger. Species already carry morale in the design but nothing drives it.
- **Food variety** — raw versus cooked multipliers and spoilage over time.

## Endgame

- **Multi-outpost worm transit** — the original "living train line": creatures enter the worm's mouth and exit at a remote outpost. The single feature that would pull the game furthest back toward the automation genre.
- **Worm Shrine pause-feeding toggle** — small QoL on the endgame calorie draw.
- Audit the worm's offering so the endgame stresses the whole factory (food *and* ingot chains) rather than only the kitchen.

## Open design questions

- Is equipment persistent per creature through desertion and death? A deserting miner walking off with the pickaxe is thematically right and mechanically cruel.
- Are mine reserves finite-but-large or infinite-with-falloff? Currently finite; switch if replacing mines reads as busywork.
- Should the blacksmith take a small sporewood fuel cost if the early game feels too frictionless?
- Should stockpiles gain a per-resource cap or a "don't haul" toggle, if playtests show the confusion is real? No zone-designation system unless the need is proven.
