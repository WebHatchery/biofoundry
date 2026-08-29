# Production Plan

## Working rule

Complete the work in priority order. Each work package ends with focused tests,
updated canonical captures where its screen changed, and a successful
`publish.ps1`. Do not start by adding content; first prove the existing game can
communicate and complete its critical path.

## P0 — Establish the playable baseline

**Outcome:** the team has evidence of the current first-session experience and
can reproduce campaign blockers.

- [ ] Run a fresh campaign using pointer/touch controls only and record every
  keyboard-only action, unclear goal, dead end, and unreadable panel.
- [ ] Record actual timing for the critical-path beats on the shipping build.
- [ ] Verify New Warren, Continue, save/load, campaign completion, endless
  continuation, and return-to-menu behavior in Windows and WebGL.
- [x] Classify every implemented system as critical-path, optional depth, or
  post-campaign.
- [x] Turn findings into small implementation issues with reproduction steps
  and acceptance criteria.

**Acceptance:** a baseline report contains the run outcome, beat times,
blockers, confusion points, viewport, input method, and build revision.

## P1 — Make the complete game touch-first

**Outcome:** every required campaign and recovery action works without a
physical keyboard or right mouse button.

- [ ] Add or verify direct drag camera movement and touch-compatible zoom.
- [ ] Remove keyboard-only wording from tutorial and player-facing HUD copy.
- [ ] Keep visible Save and Load controls; make shortcut text secondary.
- [ ] Verify all overlays, inspection actions, placement, digging, job changes,
  crafting, shrine controls, and completion choices with tap/click.
- [ ] Test UI hit targets and overlap at the supported minimum viewport.
- [ ] Add regression coverage for important UI intents where practical.

**Acceptance:** a pointer/touch-only campaign reaches Worm Awakened and returns
to the menu or continues, with no inaccessible required action.

## P2 — Clarify goals and progressive disclosure

**Outcome:** the player always sees one primary objective and encounters new
systems when they become relevant.

- [x] Add a persistent campaign objective card with current progress and the
  next unmet requirement.
- [ ] Rework the tutorial around the five journey beats in `PLAYABLE_SLICE.md`.
- [ ] Make every locked building or action state its exact prerequisite.
- [ ] Delay or collapse optional systems until the critical path establishes
  food, mining, smithing, and the worm goal.
- [ ] Ensure every tutorial instruction names the exact visible control or
  direct gesture required next.
- [ ] Provide a visible way to revisit essential help after tutorial steps
  complete or are skipped.

**Acceptance:** at least four of five first-time players can state their next
goal at each facilitator checkpoint without being told.

## P3 — Make the economy diagnosable

**Outcome:** players can explain why production is failing and select a useful
response.

- [ ] Prioritize cooked food trend, reserve, and time-to-empty in the main HUD.
- [ ] Standardize workstation status across Mine, Cook Pot, Blacksmith, Kiln,
  Smelter, and Worm Shrine.
- [ ] Surface missing labor, missing input, output blockage, invalid route, and
  reserve-policy pauses in plain language.
- [ ] Show before/after throughput when the Iron Pickaxe is equipped.
- [ ] Forecast famine and raid pressure early enough for a meaningful response.
- [ ] Audit status colors for non-color cues and text contrast.

**Acceptance:** at least four of five first-time players correctly diagnose one
seeded food-chain stall and one ore-chain stall without facilitator help.

## P4 — Tune a fair 30–45 minute arc

**Outcome:** the campaign escalates predictably, teaches recovery, and reaches
its climax before fatigue.

- [ ] Tune the opening so the first food pressure occurs after the player has
  seen all relevant controls and has time to respond.
- [ ] Tune the secured-warren goal as the end of onboarding, not a surprise
  accounting check.
- [ ] Ensure the first raid arrives with a readable warning and viable defense.
- [ ] Remove grind or idle waiting between secured warren, factory production,
  and Worm Shrine offerings.
- [ ] Protect essential food and construction reserves from optional industry.
- [ ] Verify optional systems improve or diversify the run rather than becoming
  mandatory hidden taxes.
- [ ] Keep deterministic campaign tests as broad guardrails; use human sessions
  for fun, comprehension, and pacing decisions.

**Acceptance:** at least three of five first-time players finish in 25–55
minutes, and at least four of five recover from the taught food crisis.

## P5 — Deliver the campaign climax and recovery shell

**Outcome:** completion feels authored, and interruptions or failed runs have a
clear path forward.

- [ ] Strengthen the Worm Awakened audiovisual sequence and world-state change.
- [ ] Present elapsed time and a concise campaign summary.
- [ ] Offer visible Continue in Endless Mode and Return to Menu actions.
- [ ] Add autosaves at safe beats while retaining visible manual Save/Load.
- [ ] Detect non-viable colonies where possible and offer restart/load guidance.
- [ ] Verify save behavior across refresh, relaunch, and WebGL storage limits.
- [ ] Handle incompatible or damaged saves without overwriting them silently.

**Acceptance:** completion, continuation, return, reload, and recovery flows all
work in packaged Windows and WebGL builds.

## P6 — Polish, verify, and release Phase One

**Outcome:** the playable slice is stable enough to hand to players without a
developer present.

- [ ] Resolve all open severity-0 and severity-1 playtest findings.
- [ ] Meet the comprehension and completion targets in `PLAYTEST_PLAN.md`.
- [ ] Run format, unit/integration tests, clippy with warnings denied, and the
  required no-parameter `publish.ps1` path.
- [ ] Capture every changed canonical verification scene directly into
  `docs/verification/`, replacing equivalent older images.
- [ ] Verify readable layout at representative desktop and browser sizes.
- [ ] Update `game_page.json`, catalog thumbnail, and public copy to match the
  shipped experience.
- [ ] Reconcile this document so completed work and deferred work are explicit.

**Acceptance:** every item in the release gate below is true on the candidate
commit.

## Phase One release gate

- [ ] The full campaign is completable with visible tap/click controls alone.
- [ ] A first-time player can identify the next objective throughout the run.
- [ ] The food and ore chains expose actionable stall reasons.
- [ ] The opening crisis is forecast and recoverable.
- [ ] Worm Awakened is reachable in the target session length.
- [ ] Completion and endless/menu choices are explicit.
- [ ] Save, load, restart, and incompatible-save behavior are verified.
- [ ] No severity-0 or severity-1 issues remain.
- [ ] No `.rs` file exceeds 800 physical lines.
- [ ] `publish.ps1` passes with no parameters.
- [ ] Canonical captures and player-facing store copy match the candidate.

## Deferred until after Phase One

- additional biomes, maps, species, buildings, resources, and equipment tiers;
- campaign narrative or dialogue beyond concise objective framing;
- meta-progression, achievements, multiplayer, modding, and localization;
- expansion of worm outposts beyond a clear post-campaign activity;
- systemic redesigns not supported by observed player problems.
