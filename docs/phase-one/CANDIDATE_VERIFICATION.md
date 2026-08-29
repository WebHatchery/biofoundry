# Current Candidate Verification

**Date:** 2026-08-30  
**Source revision:** `145092d`
**Published target:** WebGL Preview at `/games/biofoundry/`  
**Browser viewport:** 1280×720; game canvas 1200×675  
**Input used:** visible pointer controls only

This record supplements the historical [`P0_BASELINE_REPORT.md`](P0_BASELINE_REPORT.md).
It records checks made against the current candidate and does not turn
automated simulation results into first-time-player evidence.

## Verified in the deployed Preview

| Flow | Result | Evidence |
| --- | --- | --- |
| Continue from title | Pass | Visible Continue loaded the saved warren at `00:28`. |
| Manual Save | Pass | Visible Save produced `Warren saved.` and the run reached `00:36` before the check. |
| Menu exit without manual Save | Pass | A fresh Preview warren left through visible Menu without Save; Continue restored the run at `00:01` with the opening tutorial intact. |
| Refresh and Continue | Pass | Reload returned to the title screen with Continue enabled; Continue restored `00:36` and showed `Warren loaded.`. |
| Fresh-tab relaunch and Continue | Pass | A new Preview tab restored the same `00:36` state and objective with visible controls. |
| Minimum-layout spot check | Pass | The 1200×675 canvas and required HUD remained visible in the 1280×720 Preview viewport. |
| Visible-control smoke path | Pass | A fresh Preview warren advanced through visible New Warren, + zoom, and Farm placement without keyboard input. While the 10-ore construction site was pending, the tutorial correctly remained on `2/5 — Stabilize the Food Grid`; successful placement returned to Inspect instead of creating a second site. This is developer smoke evidence, not a qualifying first-time-player session. |

## Automated candidate checks

- `cargo fmt -- --check` — pass.
- `cargo test --all-targets` — 155 unit tests and 2 integration tests pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- `publish.ps1` with no parameters — pass; Windows and WebGL packages
  deployed to Preview.
- Fixed-seed campaign beats — secure `19.4m`, factory `24.4m`, shrine
  `29.4m`, worm `39.5m`; handoff gaps `5.0 / 5.0 / 10.1m`.
- Shrine pacing guardrail — the final offering handoff now remains under
  12 minutes on the fixed-seed campaign, while ten food offerings still map
  cleanly to the ten ingots required for awakening.
- Worm Shrine inspection capture — [ui_shrine.png](../verification/ui_shrine.png)
  shows remaining food/ingot offerings, minimum feed time, and the automatic
  offering state in the final-demand card.
- Optional support capture — [ui_optional.png](../verification/ui_optional.png)
  shows recruit controls labeled with their practical roles and the Engineer
  mine-throughput bonus at the published HUD scale. This improves discoverability
  evidence but does not replace first-time-player testing of optional-system value.
- Completion capture — [ui_completion.png](../verification/ui_completion.png)
  shows the Worm Awakened summary naming the food and ingot totals, with visible
  Continue in Endless and Return to Menu choices. The completed Objective also
  points toward the Worm Transit unlock. Packaged completion remains an open
  live-session check below.
- Endless route capture — [ui_endless.png](../verification/ui_endless.png) shows
  the completed Objective pointing to a cargo run and an active Worm Outpost
  with cargo mix, capacity, crew, and directional transit controls.
- Empty route capture — [ui_endless_empty.png](../verification/ui_endless_empty.png)
  shows an active awakened outpost with `Status · Awaiting payload`, disabled
  route actions, and the explicit `No cargo or crew ready at the warren`
  recovery state.
- Endless route recovery — automated coverage confirms food-only delivery and
  crew-only return remain valid, so an outpost cannot strand its crew when its
  cargo hold is empty; a full remote crew cannot be duplicated by another load.
- Endless route accounting — fractional local food remains in the warren until
  a whole cargo unit is ready, preventing loss when a route is loaded.
- Remote crew presentation — local Mine and Blacksmith inspection counts, plus
  the map's Overseer aura ring, exclude crew posted to an outpost; focused UI
  coverage keeps these views aligned with the simulation's local-only workforce.
- Factory handoff capture — [ui_factory_complete.png](../verification/ui_factory_complete.png)
  shows the completion overlay with the next Worm Shrine objective visible and
  no setup unlock toast obscuring the handoff.
- Save-state coverage — an in-flight Worm Transit survives a session roundtrip
  with its destination, payload, passengers, and active route state intact.
- Transit launch autosave — successful route departures now persist immediately,
  so a refresh during the worm's journey does not erase the in-flight state;
  departure notices name the destination without assuming the payload.
- Menu-exit autosave — leaving through the visible Menu control preserves a
  viable Warren for Continue, while the field guide still exposes manual Save.
- Session-boundary reset — loading or starting a Warren clears the previous
  run's pause/help/selection state and famine-warning edge before the new HUD
  becomes active.
- Modal reading state — goal reports, non-viable recovery, and the Field Guide
  hold the simulation clock while the player reads them and resume only after
  the visible modal action closes or dismisses the screen. Preview smoke check:
  `00:06` remained unchanged during the open guide and advanced to `00:07`
  after Close.
- Specialist capture — [ui_breeding.png](../verification/ui_breeding.png) shows
  the selected Breeding Pit with readable Hobgoblin, Overseer, and Engineer
  effects and available ingot costs.
- Factory tutorial capture — [ui_tutorial_factory.png](../verification/ui_tutorial_factory.png)
  makes the intended interaction explicit: inspect the existing Mine, place the
  Blacksmith, and when no eligible Idle worker is available, tap `−` by the
  named reassignable role before tapping `+` by Smith.
- Tutorial construction pacing — [ui_tutorial_food.png](../verification/ui_tutorial_food.png)
  names the Farm-then-open-floor placement taps, keeps the lesson active while
  construction is pending, and names the current pressure recovery controls;
  the completed Farm is now the handoff into the Factory lesson. The Food and
  Factory cards no longer promise a fixed Miner or Idle-based reassignment.
- Tutorial tap capture — [ui_tutorial_worm.png](../verification/ui_tutorial_worm.png)
  names the tap on the Worm Shrine before its final-demand inspection.
- One-shot placement coverage — a successful building placement returns the
  pointer to Inspect mode, so the next map tap can select a building; focused
  UI tests cover the mode transition and the live Preview smoke path confirms
  the site count does not increase on that next tap.
- Specialist job coverage — the Overseer capture shows an idle,
  non-reassignable specialist without falsely enabling worker reassignment;
  the pressure hints and visible `+`/`−` controls follow the same rule.
- Blacksmith recovery capture — [ui_blacksmith_queue_full.png](../verification/ui_blacksmith_queue_full.png)
  shows the `Queue 8/8` limit, an explicit `Queue full · finish orders first`
  recovery line, and disabled craft controls; focused UI coverage verifies the
  same capacity boundary.
- Secure-warren raid captures — [ui_raid_warning.png](../verification/ui_raid_warning.png)
  shows the compact top-bar warning naming `− Miner` and `+ Guard` when the
  warren has no idle worker without colliding with the fixed Pause control;
  [ui_raid_food_warning.png](../verification/ui_raid_food_warning.png)
  keeps the food and raid responses visible together; [ui_raid.png](../verification/ui_raid.png)
  shows the active raid after Guards are assigned. The Secure the Warren lesson
  now waits for both the campaign threshold and a Guard assignment, with save
  migration coverage preserving the lesson when a returning save has not yet
  witnessed that defense step.
- Victory handoff capture — [ui_victory.png](../verification/ui_victory.png)
  keeps the threshold report honest when no Guard has been assigned: it says
  onboarding still needs a Guard, names the visible `−` then `+ Guard` sequence,
  and labels the primary choice `Return to Warren`. A guarded warren retains the
  `Continue to Factory` handoff; focused HUD coverage exercises both branches.
- Non-viable security recovery — [ui_security_stuck.png](../verification/ui_security_stuck.png)
  shows the dedicated `Guard Handoff Blocked` recovery state when only
  non-reassignable specialists remain after the reserve threshold, with
  visible Load Last Safe and Return to Menu actions. Awakened specialist
  warrens are explicitly excluded from this failure state. Manual Save,
  milestone autosave, and viable-run Menu exit all refuse to overwrite the
  earlier checkpoint while this recovery state is active.
- Security threshold notice — the live threshold toast now follows the same
  state-aware rule as the tutorial and victory report: it only says onboarding
  is complete after a Guard exists, otherwise it directs the player to assign
  one. Focused game coverage exercises both messages.
- Objective security handoff — the persistent Objective now stays on
  `Finish the security handoff` with `Guard 0/1` after the food/ore threshold,
  and names the same visible `−` then `+ Guard` sequence until the handoff is
  complete. Focused objective coverage verifies the return to the factory goal
  after a Guard is assigned.
- Specialist-safe Objective guidance — the same handoff now checks whether an
  idle creature is actually reassignable before promising `+ Guard`; an idle
  non-reassignable specialist receives the safe fallback instead. Focused
  coverage verifies that disabled job controls are not advertised as available.
- Field Guide recovery capture — [ui_help.png](../verification/ui_help.png)
  keeps the revisitable guide truthful for the current workforce: a fresh
  warren sees the enabled `− Miner, then + Carrier/Guard` recovery paths, and
  specialist-only states receive a `free a worker` fallback instead of a
  disabled assignment.
- Shared security guidance — the victory report, raid banner, and persistent
  Objective now use the same eligibility-aware action hint, including the
  visible `in Jobs` destination and the specialist-only `free a worker`
  recovery. Focused HUD coverage keeps the victory report from promising a
  disabled Guard assignment.
- Tutorial security guidance — the Secure the Warren lesson now uses that same
  action hint instead of a fixed `Idle is 0` instruction, so its visible
  recovery step follows the current workforce and remains aligned with the
  Objective and raid warning. The embedded tutorial source copy carries the
  same state-neutral guidance for returning or tooling contexts.
- Factory Objective guidance — when the Blacksmith exists but no Smith is
  staffed, the Objective now identifies an eligible Idle worker or the exact
  reassignable role to free before adding Smith, instead of advertising a
  disabled `+ Smith` control. Focused objective coverage exercises the no-Idle
  handoff.
- Progressive optional disclosure — advanced buildings and specialist
  controls remain behind the completed `won + Guard` onboarding handoff, while
  awakened warrens retain access for post-campaign play. Refreshed
  [ui_optional.png](../verification/ui_optional.png) and
  [ui_breeding.png](../verification/ui_breeding.png) captures show the
  post-handoff state.
- Final-demand capture alignment — victory and blocked-handoff scenes now show
  advanced controls hidden until onboarding is complete, while factory and
  shrine scenes use a Guard-complete campaign fixture. Refreshed
  [ui_victory.png](../verification/ui_victory.png),
  [ui_security_stuck.png](../verification/ui_security_stuck.png),
  [ui_factory_complete.png](../verification/ui_factory_complete.png), and
  [ui_shrine.png](../verification/ui_shrine.png) captures match that rule.
- Endless route failure capture — [ui_endless_failure.png](../verification/ui_endless_failure.png)
  shows the failed outpost's wrapped recovery message and the Objective's
  complete visible Activate route instruction, alongside the top-bar failure
  signal.
- In-flight route capture — [ui_endless_in_flight.png](../verification/ui_endless_in_flight.png)
  shows the top-bar countdown and the selected outpost's directional transit
  state with a clear wait instruction. The completed-campaign Objective also
  counts successful cargo runs.
- Transit arrival capture — [ui_endless_arrived.png](../verification/ui_endless_arrived.png)
  shows the delivered crew at the active outpost and the completed-campaign
  Objective incremented to `Cargo runs 1`; the arrival is persisted through the
  normal safe-beat autosave path, and the real-loop toast accurately says
  `crew delivered` for this crew-only trip.
- Route guidance captures — [ui_endless.png](../verification/ui_endless.png),
  [ui_endless_in_flight.png](../verification/ui_endless_in_flight.png), and
  [ui_endless_arrived.png](../verification/ui_endless_arrived.png), and
  [ui_endless_empty.png](../verification/ui_endless_empty.png) show the
  Objective and inspection guidance changing from mixed cargo-and-crew return,
  to wait for transit, to return crew, to preparing a new payload.
- Mixed-payload return control — an outpost carrying both goods and passengers
  now labels the visible action with cargo, crew, and the shrine destination
  (for example, `Send 12 cargo + 2 crew to shrine`); focused UI coverage and
  refreshed endless-route captures verify the wording.
- Remote cargo accounting — subsequent outpost loads now append to existing
  ore, ingot, and cooked-food stacks instead of replacing them; focused
  simulation coverage exercises the multi-run case.
- Remote crew boundary — passengers are now marked as remote while in transit
  or stationed at an outpost, so they leave local Jobs, hunger, morale, guard
  combat, and reassignment until the return trip; focused simulation, state,
  and route-readiness coverage exercises arrival, return, and failed departure.

## Still open

- A first-time pointer/touch-only campaign through Worm Awakened has not been
  observed by a qualifying player.
- Windows and WebGL completion, endless continuation, and return-to-menu flows
  still need live packaged-build evidence rather than capture or code evidence.
- Browser storage-quota/blocked-storage behavior has not been forced in a live
  session; the runtime surfaces the shared storage rejection as a save or
  autosave warning when it occurs.
- The five-player comprehension and completion targets remain unmeasured.
