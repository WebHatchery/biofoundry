# Playable Slice Specification

## Campaign contract

A fresh Biofoundry campaign is a 30–45 minute management arc. The player
stabilizes the food grid, expands into ore, closes the equipment feedback loop,
survives pressure, and awakens the Colossal Worm. The campaign ending clearly
states that the goal is complete and offers visible controls to continue in
endless play or return to the menu.

The current broad simulation remains the implementation baseline. Phase One
curates how and when the player meets it.

## Core loop

1. Read current demand and shortages.
2. Change job assignments, placement, digging, or production queues.
3. Watch creatures carry out the plan in the world.
4. Diagnose stalled or inefficient work.
5. Expand capacity and unlock a stronger response.
6. Withstand a pressure event and repeat at a larger scale.

The shortest useful decision cycle should resolve within 10–30 seconds. Large
campaign investments may take minutes, but they must expose progress and a
reason when paused.

## First-session journey

### Beat 1: Enter the warren — 0–2 minutes

The title screen offers a clear New Warren action and a disabled or available
Continue action. Starting a game immediately frames the campaign goal and
highlights the food ledger.

The player learns to pan and zoom through direct touch gestures or visible
controls. No tutorial instruction may require WASD, function keys, right-click,
or another keyboard/mouse-specific action.

**Player understanding:** creatures do the work; food keeps all work running.

### Beat 2: Stabilize food — 2–8 minutes

The player places a Mushroom Farm, sees construction delivery, and changes job
counts to support hauling and cooking. The HUD distinguishes raw ingredients,
cooked reserves, production, upkeep, and estimated time to empty.

The opening food crisis is forecast before it becomes severe. Recovery guidance
points at the exact visible control that can help without solving the economy
automatically.

**Player understanding:** farms create input, carriers move it, cooks multiply
it, and every additional creature increases demand.

### Beat 3: Automate extraction — 8–15 minutes

The starting Mine demonstrates worker slots, buffer, rate, and hauling. The
player places a Blacksmith and assigns a Smith. Inspect panels tell the player
whether each building lacks labor, input, output room, or a route.

The player queues an Iron Pickaxe and sees it reach a miner. The upgraded rate
must be visible before and after so the feedback loop is felt rather than only
described.

**Player understanding:** ore becomes ingots, and ingots improve the creatures
that make the factory run.

### Beat 4: Secure the warren — 15–25 minutes

The player reaches the food-and-ore security threshold while responding to a
raid or comparable pressure event. Threat arrival is telegraphed and the game
names the available defense lever. A setback can cost resources or workers but
must not leave an unwinnable state without clearly offering restart/load.

Optional capture, study, breeding, specialist, morale, spoilage, and outpost
systems are introduced only when their prerequisites and immediate use are
clear. They must not compete visually with the main goal before that point.

**Player understanding:** surplus creates room to grow; growth creates new
upkeep and risk.

### Beat 5: Awaken the worm — 25–45 minutes

Forging the campaign threshold reveals the Worm Shrine as the explicit final
objective. Its panel shows construction cost, food and ingot offering progress,
reserves, pause state, and predicted blockers. The worm awakening supplies the
largest audiovisual payoff in the build.

The completion overlay reports elapsed time and offers visible Continue and
Return to Menu controls. Endless-only worm transit and outposts may follow the
ending; they are not part of the Phase One critical path.

**Player understanding:** the entire living factory was built to sustain this
final demand.

## Critical path

The required critical path is:

`New Warren → food stability → working Mine → Blacksmith → Iron Pickaxe →
secured warren → Worm Shrine → completed offerings → Worm Awakened`

Every prerequisite must be discoverable in the game. A locked action shows its
requirement; a completed requirement immediately communicates the unlock.

## Controls and accessibility

- New game, continue, pause/settings, save, load, tutorial progress, all job
  changes, building placement, digging, inspection, crafting, breeding,
  offering controls, completion choices, and recovery actions require visible
  tap/click targets.
- Camera movement requires a direct drag gesture that works for mouse and touch;
  zoom requires visible controls or a supported pinch gesture.
- Player-facing text names visible controls or direct gestures first. Shortcut
  labels may follow only when useful.
- Touch targets must remain usable at common desktop browser sizes and must not
  overlap at the smallest supported viewport.
- Status cannot rely on color alone; pair color with shape, icon, or text.
- Modal overlays trap world input and always expose a visible exit action.

## Economy legibility

The main HUD prioritizes, in order:

1. campaign objective and next requirement;
2. cooked food trend, reserve, and time to empty;
3. current warning or pressure event;
4. workforce assignment and idle capacity;
5. contextual building or creature details.

Advanced ledgers may be expandable. The opening screen must not give equal
weight to every resource in the prototype.

Every workstation reports one of these states in plain language:

- working, with rate and progress;
- no eligible worker;
- waiting for input;
- output full or waiting for haul;
- no valid route;
- paused by reserve policy;
- locked, with the exact unlock requirement.

## Failure and recovery

Phase One favors setbacks over abrupt game over. Food shortage reduces output
before desertion, raids steal resources before destroying the run, and reserve
policies protect essential construction and feeding.

The game must:

- warn before time-to-empty crosses the critical threshold;
- identify the resource or routing cause of a stall;
- preserve at least one viable recovery action during the taught crisis;
- autosave at safe campaign beats and expose a visible manual Save/Load path;
- distinguish “difficult but recoverable” from “run cannot progress”;
- offer restart and load choices if the colony becomes non-viable.

## Content included in Phase One

### Required for the critical path

- Goblins and the Miner, Carrier, Cook, Smith, Guard, and Idle jobs;
- Mushroom Farm, Cook Pot, Ore Mine, Blacksmith, Stockpile, and Worm Shrine;
- raw food, cooked food, ore, ingots, and the Iron Pickaxe;
- digging and construction delivery;
- opening food pressure and a raid;
- save/load, tutorial, campaign goals, and completion flow.

### Optional depth, retained when it supports clarity

- Beetle Hauler and Salamander Smelter;
- charcoal production and remaining equipment;
- traps, study, breeding, hobgoblins, and overseers;
- spoilage, waste, feeding troughs, janitors, bats, engineers, and morale;
- worm outposts and transit after campaign completion.

Optional systems can remain in the build, but they may be progressively
revealed or moved behind endless play. None may become an undocumented
requirement for campaign completion.

## Presentation requirements

- The title screen states the fantasy and primary action within one glance.
- Construction, crafting, unlock, raid, famine recovery, and worm awakening
  each have distinct audiovisual feedback.
- Creatures and their carried resource are readable at normal play zoom.
- Selection and placement validity remain clear over every terrain type.
- The worm awakening is a real climax, not only a text notification.

## Save compatibility

Phase One development should preserve current saves when practical. When a
schema change cannot be migrated safely, increment the data version, fail with
a clear player-facing explanation, and never silently corrupt or overwrite the
old save.
