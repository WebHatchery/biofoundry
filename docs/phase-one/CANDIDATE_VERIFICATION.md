# Current Candidate Verification

**Date:** 2026-08-30  
**Source revision:** `a3fb885`
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
| New Warren replacement warning | Pass | The visible confirmation names the autosave replacement and both choices: `Start New Warren` or `Keep Save`. |
| Manual Save | Pass | Visible Save produced `Warren saved.` and the run reached `00:36` before the check. |
| Menu exit without manual Save | Pass | A fresh Preview warren left through visible Menu without Save; Continue restored the run at `00:01` with the opening tutorial intact. |
| Refresh and Continue | Pass | Reload returned to the title screen with Continue enabled; Continue restored `00:36` and showed `Warren loaded.`. |
| Fresh-tab relaunch and Continue | Pass | A new Preview tab restored the same `00:36` state and objective with visible controls. |
| Current WebGL refresh recovery | Pass | On the published `1827d8f` Preview, a fresh Warren was saved through the visible controls, returned to the title with Menu, and restored after page refresh through Continue; the run retained tutorial `2/5 — Stabilize the Food Grid` and showed `Warren loaded.`. |
| Fresh WebGL campaign completion and Endless continuation | Pass (developer evidence) | A fresh Preview run used pointer controls through New Warren, zoom, map placement, Jobs reassignment, raid recovery, Blacksmith production, Shrine offerings, and the authored `The Colossal Worm Awakens` completion dialog. `Continue in Endless` opened the awakened-worm loop with a state-aware recovery objective and locked Pit/Outpost gates. This is repeatable developer evidence, not a qualifying first-time-player session. |
| Packaged Windows launch and save recovery | Pass | The optimized `biofoundry.exe` launched from the published Windows package. Visible Continue restored the saved warren, Skip dismissed the tutorial, zoom changed the camera, Save produced `Warren saved.`, and Load restored the run with `Warren loaded.`. This covers the shipped recovery loop; full packaged campaign completion and Endless continuation remain open below. |
| Endless cargo priority control | Pass (capture evidence) | The awakened Outpost inspection card now exposes a visible `Load order · Ore first` control. Cycling it rotates through Ore, Ingots, and Food priority, while the existing default remains ore-first and the compact 800×450 capture keeps the control and route actions readable. Live Preview route interaction remains unclaimed because the resumed developer save has not yet reached the Outpost unlock. |
| State-aware awakened objective recovery | Pass (focused and capture evidence) | After the worm wakes, the Objective now names the visible recovery action when the forge chain is incomplete: place a Blacksmith, replace an exhausted Mine, staff a Smith, or staff a Carrier. The refreshed worm and completion captures show the first of these prompts, and focused coverage keeps the guidance honest for each recovery state. |
| Minimum-layout spot check | Pass | The 1200×675 canvas and required HUD remained visible in the 1280×720 Preview viewport. |
| Public metadata alignment | Pass | The published game page now names the visible touch actions for panning, inspection, tools, Jobs controls, optional specialist recruitment, awakened Outpost cargo runs, and the post-awakening Endless/Menu choices. |
| Compact viewport exploration | Pass with follow-up | The hosted Preview smoke path at 800×450 keeps the title, field guide, and Warren HUD on-canvas; capture probes at 800×450, 1024×576, 1280×720, and 1440×900 keep the Food/Factory/Worm tutorial cards, Blacksmith queue controls, Shrine pause control, completion choices, and Endless outpost actions visible without clipping or overlap. Additional 800×450 probes keep specialist actions, Breeding Pit choices, locked-progress lines, and compact Outpost return/load actions visible. The release capture set verifies that pause, famine, food, raid, transit, and route-failure alerts stay clear of the fixed controls. The 800×450 and 1024×576 layouts remain dense, so first-time-player readability and comprehension still require human validation. |
| Visible-control smoke path | Pass | A fresh full-screen Preview warren advanced through visible New Warren, + zoom, direct map drag, map-tap inspection (`Stockpile`), valid Farm placement, `− Miner`/`+ Carrier` reassignment, Pause/Resume, Help/Close, Save, and Load without keyboard input. Invalid placement also returned the readable `Can't build there.` notice. The Load round-trip restored the saved `04:45` state with the 10-ore construction site, 2 Miner/2 Carrier staffing, and tutorial `2/5 — Stabilize the Food Grid`; successful placement returned to Inspect instead of creating a second site. This is developer smoke evidence, not a qualifying first-time-player session. |
| Packaged HUD recovery controls | Pass | A fresh full-screen Preview run changed Settings volume with visible `−`/`+` controls, opened and closed the Field Guide while paused, and resumed the warren with the same visible HUD and tutorial state. This confirms the modal guide leaves the underlying Warren controls recoverable by pointer; it does not replace the still-open full-campaign evidence. |
| Hosted-page toast safety | Pass | The placement confirmation remained fully readable above and left of the fixed Report a Bug widget in the published Preview. |
| Active tool marker | Pass | The published Preview renders the selected Dig tool as `> Dig`; the active-tool marker is readable instead of the bundled font's missing-glyph square. |
| Locked tool marker | Pass | The published Preview renders the locked Shrine control as `Shrine [L]`; its exact `forge 20 ingots` prerequisite remains visible below the buttons. |

## Automated candidate checks

- `cargo fmt -- --check` — pass.
- `cargo test --all-targets` — 212 unit tests and 2 integration tests pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- `publish.ps1` with no parameters — pass; Windows and WebGL packages
  deployed to Preview.
- Representative layout probes — pass; temporary 1024×576 and 1440×900
  captures kept tutorial, crafting, completion, and Endless controls visible
  without clipping or overlap. The compact 800×450 view remains dense and is
  still covered by the open human playtest gate.
- Optional compact probes — pass; temporary 800×450 captures kept specialist,
  Breeding Pit, locked-progress, and Outpost route controls visible without
  clipping. The small text scale remains a human-readability follow-up.
- Endless cargo priority — pass; the outbound hold obeys Ore, Ingots, or Food
  priority, preserves the local food reserve, and persists the selected order
  through a save roundtrip. `ui_endless.png` shows the visible control in the
  compact Outpost card, and the field guide explains how to use it.
- State-aware awakened objective — pass; focused coverage names the visible
  Build & Dig or Jobs recovery for a missing Blacksmith, exhausted Mine, missing
  Smith, and missing Carrier instead of promising more ingots without a viable
  production chain. Refreshed [ui_worm.png](../verification/ui_worm.png) and
  [ui_completion.png](../verification/ui_completion.png) captures show the
  Blacksmith recovery prompt, while [ui_endless_forge.png](../verification/ui_endless_forge.png)
  shows live `37/60` Worm Transit progress once the chain is viable.
- Project-local capture wrapper — pass; `scripts/capture_ui.ps1` now forwards
  viewport sizing and release/visible capture options to the shared toolkit,
  and its 800×450 completion/Endless path was exercised successfully.
- Native release critical-scene capture — pass; the optimized project binary
  captured completion, Endless outpost, Shrine, and Blacksmith at 800×450.
  Completion choices and outpost actions stayed visible, while the Shrine and
  Blacksmith panels remained readable without clipping or overlap. This is
  packaged-native scene evidence; the live Preview run separately verifies the
  pointer-only completion and Endless continuation branch.
- Native release recovery capture — pass; rebuilt release capture now accepts
  the documented `route_failure` alias and shows the red route alert, inactive
  outpost action, recovery instruction, and returned cargo at 800×450. The
  refreshed [ui_compact_route_failure.png](../verification/ui_compact_route_failure.png)
  is the matching optimized-binary evidence.
- Camera drag release guard — focused input coverage keeps a claimed mouse
  drag from selecting a map tile or activating a HUD control on release.
- Touch map taps — focused input coverage and the explicit gesture path keep a
  short touch on open floor available to world tools and building inspection,
  even when the browser does not synthesize a mouse release; touch releases
  now pass through the live camera pan/zoom before world-tile resolution.
- Touch HUD ownership — focused HUD coverage maps a letterboxed touch release
  into UI coordinates, so a button tap cannot also fall through to the map
  when the mouse cursor is elsewhere.
- Shared toolkit notification-offset tests — 360 tests pass; the hosted-page
  toast offset is opt-in, so existing notification anchors remain unchanged.
- Compact alert captures — [ui_compact_food_warning.png](../verification/ui_compact_food_warning.png),
  [ui_compact_raid_food_warning.png](../verification/ui_compact_raid_food_warning.png),
  [ui_compact_raid_warning.png](../verification/ui_compact_raid_warning.png),
  and [ui_compact_raid.png](../verification/ui_compact_raid.png) verify the
  800×450 warning and active-raid labels stop before Pause; the matching
  [ui_compact_pause.png](../verification/ui_compact_pause.png),
  [ui_compact_transit.png](../verification/ui_compact_transit.png),
  [ui_compact_route_failure.png](../verification/ui_compact_route_failure.png),
  and [ui_compact_famine.png](../verification/ui_compact_famine.png) captures
  cover the remaining top-bar alerts and the fully readable famine toast.
- Fixed-seed campaign beats — secure `19.4m`, factory `24.4m`, shrine
  `29.4m`, worm `39.5m`; handoff gaps `5.0 / 5.0 / 10.1m`.
- Shrine pacing guardrail — the final offering handoff now remains under
  12 minutes on the fixed-seed campaign, while ten food offerings still map
  cleanly to the ten ingots required for awakening.
- Worm Shrine inspection capture — [ui_shrine.png](../verification/ui_shrine.png)
  shows remaining food/ingot offerings, minimum feed time, and the automatic
  offering state in the final-demand card.
- Optional support capture — [ui_optional.png](../verification/ui_optional.png)
  shows recruit controls labeled with their practical actions (`Beetle haul`,
  `Salam. forge`,
  `Slime · clean`, and `Bat · 8 cargo`) plus the Engineer mine-throughput bonus
  at the published HUD scale. This improves discoverability evidence but does
  not replace first-time-player testing of optional-system value.
- Breeding feedback — successful Hobgoblin, Overseer, and Engineer choices now
  name their tuned work, aura, or mine benefit in the recruitment notice;
  focused game-action coverage checks those data-driven percentages alongside
  the existing breeding labels.
- Unlock feedback — newly earned building and creature systems now name their
  exact destination (`Build & Dig` or Jobs/Breeding Pit); when progressive
  disclosure still hides those optional controls, the notice says they become
  available after onboarding instead of sending the player to an invisible
  action. Hardened Guards and Preservation Techniques still announce their
  percentage benefits; focused game coverage checks both visibility states.
- Locked-gate progress capture — the refreshed [ui_optional.png](../verification/ui_optional.png)
  shows current/threshold values for the remaining Build & Dig unlocks, so a
  locked action communicates both its requirement and how close the warren is.
- Objective gate alignment — the optional capture now keeps the Objective's
  Worm Shrine prerequisite aligned with the disabled Shrine button when the
  unlock is missing, including the same forge count and next action.
- Raw ingredient ledger — the Food Grid now shows the raw mushroom total beside
  production and labels the reserve as cooked food; focused simulation coverage
  keeps the displayed raw mirror aligned with post-spoilage stock.
- Spendable ingot ledger — the Food Grid now keeps banked ingots visible beside
  banked ore, so optional breeding and shrine costs have an always-visible
  resource source; [ui_optional.png](../verification/ui_optional.png) confirms
  the line remains readable with advanced controls open.
- Blacksmith input stall — [ui_blacksmith.png](../verification/ui_blacksmith.png)
  shows a staffed smith with an unpaid Iron Pickaxe queued, `Ore 0`, and the
  visible `Status · Starved` plus `Needs 2 ore · next Iron Pickaxe` recovery
  line; focused UI coverage keeps a paid order nominal while exposing the
  missing-ore stall.
- Smelter input stall — [ui_smelter.png](../verification/ui_smelter.png)
  shows a staffed Salamander with both batch inputs withheld, and the
  inspection card names the exact recovery need: `Needs 1 ore + 1 charcoal`.
- Cook Pot input stall — [ui_cook_pot.png](../verification/ui_cook_pot.png)
  shows a staffed cook with no mushrooms, and the critical-path inspection
  card names the next batch need: `Needs 2 mushrooms`.
- Kiln input stall — [ui_kiln.png](../verification/ui_kiln.png) shows the
  autonomous charcoal kiln with an empty wood buffer and the direct recovery
  line `Needs 1 wood`.
- Fractional recipe status — focused legibility coverage keeps a Cook Pot's
  `Starved` status aligned with the simulation's rounded-up batch requirement
  when data-driven recipe multipliers produce a fractional amount.
- Smelter staffing — the refreshed [ui_smelter.png](../verification/ui_smelter.png)
  identifies the local Salamander as `Salamander stationed` while the den is
  starved; focused inspection coverage excludes remote Salamanders from that
  local staffing read.
- Waste recovery capture — [ui_waste.png](../verification/ui_waste.png) shows
  an early food node with `Waste accumulating`, the current `Waste 2.5`, and
  the truthful `Secure warren first` recovery hint before specialist controls
  are available; focused inspection coverage also exercises the unlocked and
  staffed Slime paths.
- Completion capture — [ui_completion.png](../verification/ui_completion.png)
  shows the Worm Awakened summary naming the food and ingot totals, with visible
  Continue in Endless and Return to Menu choices. The completed Objective also
  points toward the first visible Endless forge recovery step. Packaged completion remains an open
  live-session check below.
- Endless route capture — [ui_endless.png](../verification/ui_endless.png) shows
  the completed Objective pointing to a cargo run and an active Worm Outpost
  with cargo mix, capacity, crew, and directional transit controls.
- Empty route capture — [ui_endless_empty.png](../verification/ui_endless_empty.png)
  shows an active awakened outpost with `Status · Awaiting payload`, disabled
  route actions, and the explicit `No cargo or crew ready at the warren`
  recovery state, with a readable gap separating the recovery note from the
  action button.
- Endless route recovery — automated coverage confirms food-only delivery and
  crew-only return remain valid, so an outpost cannot strand its crew when its
  cargo hold is empty; a full remote crew cannot be duplicated by another load.
- Endless route accounting — fractional local food remains in the warren until
  a whole cargo unit is ready, preventing loss when a route is loaded.
- Remote crew presentation — local Mine and Blacksmith inspection counts, plus
  the map's Overseer aura ring, exclude crew posted to an outpost; focused UI
  coverage keeps these views aligned with the simulation's local-only workforce.
- Route toggle feedback — the activation toast now names the resulting active or
  inactive state, matching the selected outpost's button and persisted route.
  Focused game-action coverage exercises both toggle outcomes.
- Route failure recovery — reopening a failed inactive route now clears the
  global top-bar failure banner as well as the selected outpost's local failure,
  so recovery feedback does not remain stale before the next transit.
- Station inspection feedback — Mine and Blacksmith detail cards now distinguish
  assigned or en-route local workers from active production, while excluding
  remote outpost crew; focused UI coverage exercises both boundaries.
- Engineer mine staffing — Mine status icons and inspection cards recognize the
  dedicated Engineer as active mine staff, while retaining the remote-crew
  exclusion; focused UI coverage exercises both presentation paths.
- Unique optional posts — the Jobs panel now labels an already recruited Slime
  Janitor or Bat Courier as posted instead of presenting a disabled recruit
  control without a reason; focused UI coverage exercises both label states.
- Slime unlock reachability — the locked Jobs-panel prerequisite now tracks
  lifetime spoiled food, so the Slime Janitor no longer depends on a janitor
  having already processed waste; focused simulation coverage confirms the
  unlock crosses naturally before any janitor exists.
- Spoiled-store visibility — the map badge and inspection status now flag
  waste on Farms and Cook Pots as well as Feeding Troughs, so the cleanup
  target remains visible before the Slime Janitor is recruited; focused UI
  coverage exercises a spoiled Farm.
- Local Engineer summary — the Jobs panel now distinguishes Engineers working
  in the warren from Engineers posted remotely, so the Mine bonus is not shown
  as locally active when the specialist is away; focused UI coverage exercises
  the summary states.
- Local workforce capacity — the Jobs panel now shows local workers against
  local floor capacity beside Idle, using the same remote-outpost exclusion as
  the crowding simulation; the refreshed optional capture keeps the readout
  legible with advanced controls visible.
- In-flight payload feedback — the selected outpost now names the cargo and crew
  currently carried by the worm during transit, so emptied storage counters are
  not mistaken for lost payload; focused UI coverage exercises the summary.
- Outpost route status — the selected route now distinguishes active, ready to
  load, payload ready, and awaiting payload states instead of calling a loaded
  route generically “Working”; focused UI coverage exercises the route states.
- Inactive route map status — a closed Worm Outpost now uses a distinct
  crossed-ring `Route inactive` badge and legend entry instead of the generic
  `Starved` marker, including when its cargo or crew is held for recovery.
- Inactive loaded-route recovery — a deactivated outpost holding cargo or crew
  now reports that its payload is held and directs the player to reactivate the
  route before returning it; focused UI coverage exercises the inspection and
  objective recovery states.
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
- Tutorial save migration — a save made while the tutorial Farm is still under
  construction keeps the Food lesson visible after reload; migration only
  advances that beat after a finished second Farm is present.
- Modal reading state — goal reports, non-viable recovery, and the Field Guide
  hold the simulation clock while the player reads them and resume only after
  the visible modal action closes or dismisses the screen. Preview smoke check:
  `00:06` remained unchanged during the open guide and advanced to `00:07`
  after Close.
- Specialist capture — [ui_breeding.png](../verification/ui_breeding.png) shows
  the selected Breeding Pit with readable Hobgoblin, Overseer, and Engineer
  effects and available ingot costs.
- Locked specialist labels — focused inspection coverage keeps unavailable
  breeding choices readable as `[L]` instead of the bundled font's missing
  glyph.
- Locked specialist progress — [ui_breeding_locked.png](../verification/ui_breeding_locked.png)
  shows each unavailable breeding choice with its exact forge requirement and
  live progress, while the refreshed [ui_breeding.png](../verification/ui_breeding.png)
  keeps the unlocked specialist benefits and costs readable. Both cards now
  state that specialist choices spend banked ingots.
- Factory tutorial capture — [ui_tutorial_factory.png](../verification/ui_tutorial_factory.png)
  makes the intended interaction explicit: inspect the existing Mine, place the
  Blacksmith, and when no eligible Idle worker is available, tap `−` by the
  named reassignable role before tapping `+` by Smith.
- Construction remainder capture — the refreshed [ui_factory.png](../verification/ui_factory.png)
  labels pending work as `Build 1 site · 4 ore left`, making the amount still
  owed to the construction site explicit instead of presenting an unlabeled
  ore counter.
- Tutorial construction pacing — [ui_tutorial_food.png](../verification/ui_tutorial_food.png)
  names the Farm-then-open-floor placement taps, keeps the lesson active while
  construction is pending, and names the current pressure recovery controls;
  the completed Farm is now the handoff into the Factory lesson. The Food and
  Factory cards no longer promise a fixed Miner or Idle-based reassignment.
  The expanded tutorial card keeps the final recovery instruction above Skip;
  the Food, Factory, and Worm beats remain readable in an 800×450 probe.
- Tutorial completion guard — focused coverage confirms that completing an
  unrelated building cannot satisfy the Farm-construction lesson; the lesson
  now waits for the player-built second Farm.
- Tutorial tap capture — [ui_tutorial_worm.png](../verification/ui_tutorial_worm.png)
  names the tap on the Worm Shrine before its final-demand inspection; the
  inspection card now starts below the expanded tutorial card, leaving the
  visible `Skip` action reachable in the canonical and compact captures.
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
  disabled assignment. The same guide now points to the Breeding Pit for
  specialist unlocks, costs, and benefits after onboarding, and to the Outpost
  for optional cargo runs after the worm wakes.
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
  signal; the inactive-route recovery note is spaced below its action button.
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
- Route load recovery — loading a save now restores stationed crew to the
  outpost tile and clears stale local tasks, while preserving in-flight
  passenger coordinates; focused state coverage exercises the repair.
- Route ownership recovery — loading a save drops unknown or duplicate crew IDs
  from outpost and in-flight ownership records, keeping route capacity and
  return payloads tied to the actual creature roster.

## Still open

- A first-time pointer/touch-only campaign through Worm Awakened has not been
  observed by a qualifying player.
- Windows packaged campaign completion/Endless continuation still needs live
  packaged-build evidence rather than capture or code evidence. Packaged launch,
  Continue, Save, and Load now pass; the current WebGL Preview has live developer
  evidence for both authored completion and `Continue in Endless`; neither result
  counts toward the first-time-player gate.
- Browser storage-quota/blocked-storage behavior has not been forced in a live
  session; the runtime surfaces the shared storage rejection as a save or
  autosave warning when it occurs.
- The five-player comprehension and completion targets remain unmeasured.
