# Current Candidate Verification

**Date:** 2026-08-31
**Source revision:** `4549cf8`
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
| Saved event history across refresh | Pass | On the published Preview, a fresh Warren was paused, saved, returned to the title through visible Menu, and restored after a page refresh through Continue; Recent Events retained the earlier `Simulation paused` and `Warren saved.` entries alongside the new `Warren loaded.` entry. |
| Current WebGL refresh recovery | Pass | On the published `1827d8f` Preview, a fresh Warren was saved through the visible controls, returned to the title with Menu, and restored after page refresh through Continue; the run retained tutorial `2/5 — Stabilize the Food Grid` and showed `Warren loaded.`. |
| Fresh WebGL campaign completion and Endless continuation | Pass (developer evidence) | A fresh Preview run used pointer controls through New Warren, zoom, map placement, Jobs reassignment, raid recovery, Blacksmith production, Shrine offerings, and the authored `The Colossal Worm Awakens` completion dialog. `Continue in Endless` opened the awakened-worm loop with a state-aware recovery objective and locked Pit/Outpost gates. This is repeatable developer evidence, not a qualifying first-time-player session. |
| Packaged Windows launch and save recovery | Pass | The optimized `biofoundry.exe` launched from the current published Windows package after the stale prior test instance was isolated. Visible Continue restored the saved warren at `00:01` with the opening tutorial and `Warren loaded.`. Earlier packaged checks also covered Skip, zoom, Save, and Load; full packaged campaign completion and Endless continuation remain open below. |
| Endless cargo priority control | Pass (capture evidence) | The awakened Outpost inspection card now exposes a visible `Load order · Ore first` control. Cycling it rotates through Ore, Ingots, and Food priority, while the shared route forecast shows the exact next Ore/Ingots/Food mix and explains when a full hold must return to the shrine. The compact 800×450 captures keep the control, forecast, and route actions readable. Live Preview route interaction remains unclaimed because the resumed developer save has not yet reached the Outpost unlock. |
| Endless remote scouting loop | Pass (focused and capture evidence) | Once an awakened Outpost has remote crew and cooked food, its expedition advances toward a scouting haul, consumes provisions, and stores ore in the remote hold. The inspection card exposes a touch-first Pause/Resume scouting control that protects remote food without closing the route; the map badge, legend, inspection card, and Objective agree on manual pause, while the card and Objective also agree on progress, food shortfall, or full-hold pause. Completed hauls report their ore and food delta through the visible notification system and autosave as a safe beat. Live Preview route interaction remains unclaimed because the resumed developer save has not yet reached the Outpost unlock. |
| Paused- and blocker-route status legibility | Pass (focused and capture evidence) | A manually paused active Outpost now carries a distinct pause glyph and `Scouting paused` label on the map and status legend, as well as the matching inspection status. Awakened routes without stationed crew, without enough scout food, or with a full remote hold now carry distinct `No scout crew`, `Scout food low`, or `Outpost full` map and inspection-detail states. The compact legend filters to statuses present in the current warren, keeping the badge, recovery control, and Objective visible together. |
| Worm Shrine reserve status legibility | Pass (focused and published capture evidence) | The map and filtered status legend now expose the same final-demand blockers already shown in the Shrine card: `Offerings paused`, `Food reserve low`, and `Ingot reserve low`. An awakened Shrine remains quiet, while the shared blocker helpers keep the map badge and inspection wording aligned. The published [ui_shrine_waiting.png](../verification/ui_shrine_waiting.png) capture shows the food-reserve badge beside the matching inspection state. |
| Endless expedition feedback | Pass (focused and capture evidence) | A completed remote haul now reports its ore gain and food cost through the visible notification system and marks the haul as a safe-beat autosave, so progress is communicated and survives a refresh without keeping the inspection card open. Each Outpost persists its completed-haul count and lifetime ore gathered in the inspection card, while the completed-campaign Objective keeps aggregate `Runs` and `Hauls` visible across the Endless loop. |
| Endless Outpost hold expansion | Pass (focused and capture evidence) | An active awakened route can spend 8 banked ingots once to expand its remote hold from 12 to 20 slots. The inspection card exposes the visible `Expand hold · 8 ingots` control, shows the upgraded `Cargo 6/20` capacity, and keeps transit, scouting, load-preview, full-hold, and Objective room calculations aligned. |
| Endless crew dispatch quota | Pass (focused and capture evidence) | An awakened Outpost exposes the visible `Crew per run · Auto` control, which cycles through cargo-only and bounded scout counts. Legacy saves keep automatic dispatch, while cargo-only loads can deliver provisions without borrowing local workers; the Objective names the control when that setting blocks an otherwise-ready scouting payload. Refreshed [ui_endless_load_preview.png](../verification/ui_endless_load_preview.png), [ui_endless_upgrade.png](../verification/ui_endless_upgrade.png), and [ui_endless_upgraded.png](../verification/ui_endless_upgraded.png) captures keep the route controls readable. |
| Endless cargo-only returns | Pass (focused and capture evidence) | A staffed Outpost exposes `Send N cargo · keep crew` beside the normal full return, so a completed haul can come home without recalling the remote scouts. The route keeps those scouts assigned for another expedition, the departure notice explains the choice, and the Objective/field guide describe the remote-team outcome. Refreshed [ui_endless.png](../verification/ui_endless.png), [ui_endless_expedition_report.png](../verification/ui_endless_expedition_report.png), and [ui_endless_upgraded.png](../verification/ui_endless_upgraded.png) captures keep both return paths readable. |
| Endless automatic cargo returns | Pass (focused and capture evidence) | An awakened Outpost exposes the persisted `Auto-return · Off` / `Auto-return · Cargo only` policy. When an opted-in hold reaches capacity, the fixed-step simulation starts one cargo-only return, preserves the remote scouts and one configured expedition's food when other cargo creates room, and returns provisions too when they alone fill the hold so the route can resupply. It emits a departure notice and autosaves the safe beat; later routes wait for the global worm transit. Refreshed [ui_endless_auto_return.png](../verification/ui_endless_auto_return.png) shows the enabled policy beside the manual return and scouting controls. Live Preview route interaction remains unclaimed because the resumed developer save has not yet reached the Outpost unlock. |
| Endless automatic Outpost resupply | Pass (focused and capture evidence) | An awakened Outpost exposes the persisted `Auto-resupply · Off` / `Auto-resupply · Food only` policy. When staffed remote scouts need provisions, the fixed-step simulation starts a food-only transit from the home reserve without dispatching more crew; a manually paused expedition is left untouched. The departure notice, Objective, and field guide name the automatic behavior. Refreshed [ui_endless_auto_resupply.png](../verification/ui_endless_auto_resupply.png) shows the shortage state, enabled policy, and visible recovery controls. Live Preview route interaction remains unclaimed because the resumed developer save has not yet reached the Outpost unlock. |
| Endless transit failure recovery | Pass (focused and published build evidence) | If an in-flight worm route is deactivated, the simulation restores its payload, records the failed Outpost, and emits a one-shot transit-failure event. The game turns that event into a visible danger notification with the exact recovery action and treats it as a safe-beat autosave, so a refresh cannot erase the route's recovered state. |
| Multi-route failure visibility | Pass (focused evidence) | Reopening one failed Outpost now clears only that route's failure; the global route warning remains visible while any other Outpost still needs recovery, including when a healthy second route starts a transit. This keeps the top-bar warning consistent with the per-route inspection state as remote routes multiply. |
| Loaded route failure visibility | Pass (focused and published build evidence) | Installing a save now reconciles each persisted Outpost failure with the global top-bar warning, repairing a missing banner and clearing a stale one. The route records remain authoritative across refresh and load boundaries. |
| Loaded Outpost record recovery | Pass (focused and published build evidence) | Loading a structurally valid legacy session now recreates a missing route record for every placed Outpost before HUD and remote-crew reconciliation. The repaired route starts inactive at the matching tile, restores Routes ledger availability, and is ready to persist on the next normal Save. |
| Loaded session integrity validation | Pass (focused and published build evidence) | Current-version, migrated primary, and backup saves are checked before installation for valid map storage, non-overlapping world objects, known content IDs, safe actor/task positions, unique roster IDs, bounded remote cargo and crew, valid transit timing, and finite simulation values. To-Outpost transit is checked against both the stored destination hold and its arriving payload, and a transit cannot exist before the worm awakens. The current-version toolkit fast path now receives the same game-owned validation as migrated saves; invalid shapes enter the existing quarantine/backup recovery flow instead of poisoning the live Warren. |
| Backup-only load recovery | Pass (focused and published build evidence) | If a primary slot is missing while its conventional `_backup` survives, `Load` now validates the backup, restores the primary slot, and installs the recovered Warren; if the restore write is rejected, the valid backup remains playable and the player is told to use Save. The focused recovery-path coverage and published candidate include this branch, while ordinary Preview Continue remains verified separately. |
| Backup-only startup recovery | Pass (focused and published build evidence) | Title-screen availability now treats either the primary slot or its conventional `_backup` as a recoverable save. A primary-missing/backup-survives launch can therefore reach `Load` and use the existing backup recovery path instead of incorrectly disabling Continue. |
| Save failure recovery guidance | Pass (focused and published build evidence) | Manual save failures now say whether the previous checkpoint remains available or no new save was written; autosave failures distinguish an existing checkpoint from a first-save attempt and explain the retry path. The four message states are covered by focused tests and the published candidate; forced browser quota/blocked-storage behavior remains open below. |
| Load availability reconciliation | Pass (focused and published build evidence) | If Load finds neither the primary slot nor its backup, the title screen now disables the stale Continue action and reports that no saved warren is available, with a visible New Warren recovery path. A successful Load also re-enables Continue when startup or an earlier recovery had marked the slot unavailable. Startup now recognizes a surviving backup as available, while damaged-save recovery remains unchanged. |
| Outpost route setting persistence | Pass (focused and published build evidence) | Successful route activation, cargo-order changes, crew quotas, scouting pause/resume, and automatic return/resupply policy changes now write an autosave immediately. A refresh after changing a route control therefore preserves the player's recovery or logistics decision instead of waiting for a later transit or expedition beat. |
| Worm Shrine offering policy persistence | Pass (focused and published build evidence) | The visible Pause/Resume offerings control now autosaves immediately after a successful toggle, so protecting the reserve remains in force across refreshes instead of silently restarting the Shrine draw. Invalid targets do not write a checkpoint. |
| Direct player decision persistence | Pass (focused and published build evidence) | Successful job reassignment, specialist recruitment, breeding, Blacksmith queueing, tutorial skip, milestone-report dismissal, building placement, and dig designation now write an immediate autosave. Failed or duplicate actions do not write, and the refreshed save roundtrip covers the persisted job, queue, map, and decision flags. |
| Progression checkpoint persistence | Pass (focused and published build evidence) | Capture unlocks, newly granted systems, breeding-pit hatches, survived raids, and tutorial-step advancement now enter the safe-beat autosave path. The next refresh therefore retains earned progression without turning ordinary simulation ticks into constant storage writes. |
| Food-crisis Farm recovery | Pass (focused and published Preview) | Below the normal Carrier food reserve, a pending Farm now still receives construction ore from the stockpile or a backed-up Mine buffer; unrelated Blacksmith construction remains shed. The Objective names that Farm handoff and the visible Jobs response when Food is falling. After reloading the published Preview save with Food `24` and a Farm site needing `10` ore, the site advanced to `9` ore remaining and the objective reached `2/50` delivered ore during the food crisis; the refreshed [ui_tutorial_food.png](../verification/ui_tutorial_food.png) capture preserves the same handoff. |
| Reachable construction placement | Pass (focused and capture evidence) | Building ghosts now require a walkable path from the stockpile, so a player cannot create a worker-delivered construction site in an isolated floor pocket. The generated spawn chamber also restores its center exits after procedural water placement; focused coverage checks the placement rejection and 128 deterministic world seeds. Legacy worker-serviced buildings already present in older saves now show `No valid route` on the map and filtered legend, with the inspection card directing the player to dig a tunnel; [ui_unreachable_workstation.png](../verification/ui_unreachable_workstation.png) preserves the recovery state. |
| Engineer Mine slot accounting | Pass (focused and published build evidence) | The optional Engineer now uses the same live Mine-claim accounting as ordinary Miners, so it waits when every post is occupied and reserves a free slot before walking toward it. A stale over-capacity arrival is rejected instead of overbooking the Mine; focused simulation coverage verifies both branches. |
| Multi-Outpost automatic scheduling | Pass (focused evidence) | Automatic returns and food-only resupplies share a persisted round-robin cursor, so a continuously needy first route cannot monopolize the single worm transit. The cursor advances only after a successful automatic departure and defaults safely for older saves; focused multi-route simulation and save-roundtrip coverage verifies that later eligible Outposts receive their turn. |
| Multi-route ledger | Pass (published capture evidence) | The awakened HUD now exposes a visible Routes control that opens a modal Worm Route Ledger. Each route shows the same inspection status vocabulary, live `Scouting` state and percentage when active, cargo/hold and crew counts, policy state, and a touch-sized Inspect action into the existing Outpost card; selecting a route also centers the bounds-clamped map camera on it. The release [ui_endless_routes.png](../verification/ui_endless_routes.png) capture shows two routes together; the ledger pauses planning and keeps the Close action visible in the compact probe. |
| Network route summary | Pass (published capture evidence) | The Worm Route Ledger now summarizes the whole network above its cards: total routes, active routes, held cargo, remote crew, scouted ore, and routes needing attention. The summary is derived from the same persisted route state as the cards, so scaling from one route to several does not require opening each card to understand the network. Refreshed [ui_endless_routes.png](../verification/ui_endless_routes.png) and the compact touch audit keep the summary and route controls readable. |
| World-space worm route links | Pass (published capture evidence) | After awakening, the world now draws each shrine-to-Outpost link beneath the buildings, using a bright solid path for active routes and a subdued dashed path for inactive ones. An in-flight return shows a high-contrast pulse moving from the Outpost toward the shrine, while outbound travel reverses that direction. Refreshed [ui_endless.png](../verification/ui_endless.png), [ui_endless_in_flight.png](../verification/ui_endless_in_flight.png), and [ui_endless_arrived.png](../verification/ui_endless_arrived.png) captures show the network feedback without changing route state or save data. |
| Settled touch-target audit | Pass (focused evidence) | Every enabled menu/HUD button now registers with the shared touch audit. The opt-in `scripts/audit_touch_targets.ps1` sweep settles each screen for neighbor-aware hit growth, reports the smallest target and drawn density, and fails on actual grown-target overlap. The 800×450 sweep passed for the title, settings, confirmation, core HUD, crafting, shrine, completion, Recent Events newest and older pages, Outpost, and multi-route ledger screens; modal occlusion keeps covered controls out of the report. |
| State-aware awakened objective recovery | Pass (focused and capture evidence) | After the worm wakes, the Objective now names the visible recovery action when the forge chain is incomplete: place a Blacksmith, replace an exhausted Mine, staff a Smith, or staff a Carrier. The refreshed worm and completion captures show the first of these prompts, and focused coverage keeps the guidance honest for each recovery state. |
| Multi-route objective ordering | Pass (focused evidence) | If several active Worm Outposts are present, the completed-campaign Objective prioritizes a route with cargo or crew ready, then a route that can be loaded, before an empty active route. This keeps the next visible instruction actionable as the existing outpost activity scales. |
| Minimum-layout spot check | Pass | The 1200×675 canvas and required HUD remained visible in the 1280×720 Preview viewport. |
| Public metadata alignment | Pass | The published game page now names the visible touch actions for panning, inspection, tools, Jobs controls, optional specialist recruitment, awakened Outpost cargo runs, optional automatic cargo-only returns and food-only resupply, and the post-awakening Endless/Menu choices. |
| Compact viewport exploration | Pass with follow-up | The hosted Preview smoke path at 800×450 keeps the title, field guide, and Warren HUD on-canvas; capture probes at 800×450, 1024×576, 1280×720, and 1440×900 keep the Food/Factory/Worm tutorial cards, Blacksmith queue controls, Shrine pause control, completion choices, and Endless outpost actions visible without clipping or overlap. The compact branch now gives the high-frequency top-bar, Jobs, Build & Dig, Outpost, Blacksmith, Breeding Pit, and Shrine controls larger visual affordances, and the refreshed warning captures keep their messages clear of that row. Representative [ui_compact_warren.png](../verification/ui_compact_warren.png), [ui_compact_blacksmith.png](../verification/ui_compact_blacksmith.png), [ui_compact_breeding.png](../verification/ui_compact_breeding.png), [ui_compact_shrine.png](../verification/ui_compact_shrine.png), [ui_compact_endless_load_preview.png](../verification/ui_compact_endless_load_preview.png), and [ui_compact_endless_upgraded.png](../verification/ui_compact_endless_upgraded.png) captures preserve the responsive states. The compact Blacksmith and Breeding Pit follow-up now uses 36-pixel action targets with 40-pixel spacing; remaining text density and first-time-player comprehension still require human validation. |
| Visible-control smoke path | Pass | A fresh full-screen Preview warren advanced through visible New Warren, + zoom, direct map drag, map-tap inspection (`Stockpile`), valid Farm placement, `− Miner`/`+ Carrier` reassignment, Pause/Resume, Help/Close, Save, and Load without keyboard input. Invalid placement also returned the readable `Can't build there.` notice. The Load round-trip restored the saved `04:45` state with the 10-ore construction site, 2 Miner/2 Carrier staffing, and tutorial `2/5 — Stabilize the Food Grid`; successful placement returned to Inspect instead of creating a second site. This is developer smoke evidence, not a qualifying first-time-player session. |
| Packaged HUD recovery controls | Pass | A fresh full-screen Preview run changed Settings volume with visible `−`/`+` controls, opened and closed the Field Guide while paused, and resumed the warren with the same visible HUD and tutorial state. This confirms the modal guide leaves the underlying Warren controls recoverable by pointer; it does not replace the still-open full-campaign evidence. |
| Recent event recovery log | Pass (published capture and Preview) | The visible Field Guide now opens a newest-first Recent Events view backed by the bounded notification history. Published Preview verification opened the guide after a refresh and reviewed persisted `Simulation paused`, `Warren saved.`, and `Warren loaded.` messages; the modal kept both `Field Guide` and `Close` actions available. |
| Recent event history paging | Pass (published Preview, capture, and compact touch audit) | The bounded log exposes visible `Older` and `Newer` controls when more than ten notices exist, labels the current page, keeps the newest page as the default, and resets to that page when reopened or crossing a session boundary. Published Preview verification generated twelve visible Pause/Resume notices, tapped `Older` to reveal the earlier saved entries on page `2/2`, then tapped `Newer` to return to page `1/2`. The [ui_event_log_older.png](../verification/ui_event_log_older.png) and compact [ui_compact_event_log_older.png](../verification/ui_compact_event_log_older.png) captures show the older page; the 800×450 audit reports no grown-target overlap. |
| Hosted-page toast safety | Pass | The placement confirmation remained fully readable above and left of the fixed Report a Bug widget in the published Preview. |
| Active tool marker | Pass | The published Preview renders the selected Dig tool as `> Dig`; the active-tool marker is readable instead of the bundled font's missing-glyph square. |
| Locked tool marker | Pass | The published Preview renders the locked Shrine control as `Shrine [L]`; its exact `forge 20 ingots` prerequisite remains visible below the buttons. |

## Automated candidate checks

- `cargo fmt -- --check` — pass.
- `cargo test --all-targets` — 315 unit tests and 2 integration/code-standard
  targets pass,
  including fresh/simulated/remote-transit valid sessions, modal route
  planning, rejected malformed save shapes, and event-history save/load
  compatibility coverage.
- `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- `publish.ps1` with no parameters — pass; Windows and WebGL packages
  deployed to Preview.
- Representative layout probes — pass; temporary 1024×576 and 1440×900
  captures kept tutorial, crafting, completion, and Endless controls visible
  without clipping or overlap. The compact 800×450 view remains dense and is
  still covered by the open human playtest gate.
- Optional compact probes — pass; temporary 800×450 captures kept specialist,
  Breeding Pit, locked-progress, and Outpost route controls visible without
  clipping. The refreshed [ui_compact_blacksmith.png](../verification/ui_compact_blacksmith.png),
  [ui_compact_breeding.png](../verification/ui_compact_breeding.png), and
  [ui_compact_shrine.png](../verification/ui_compact_shrine.png) captures show
  the enlarged craft, specialist, and Shrine actions. Remaining text density
  remains a human-readability follow-up.
- Endless cargo priority — pass; the outbound hold obeys Ore, Ingots, or Food
  priority, preserves the local food reserve, and persists the selected order
  through a save roundtrip. The shared `CargoLoad` forecast now drives both
  route execution and the inspection preview, so the player can see the next
  load before departure and receives a recovery hint when the hold is full.
  `ui_endless.png` shows the full-hold state, while
  [ui_endless_load_preview.png](../verification/ui_endless_load_preview.png)
  shows the selected `Ingots first` mix in the compact Outpost card. The field
  guide still explains how to use the priority control.
- Endless remote scouting — pass; active staffed outposts consume their stored
  cooked-food provision on a data-driven cycle and add the configured ore haul
  to the remote hold. The cycle pauses without enough food or hold capacity,
  and save-compatible defaults preserve older outpost records. A visible
  Pause/Resume scouting control protects the remote provision while keeping
  the route active, and its persisted state is reflected in the inspection
  hint and completed-campaign Objective. Focused simulation/UI coverage and
  the compact route captures verify the progress, blocker, manual pause, output,
  and completion-feedback states. Completed hauls emit a concise resource
  delta and trigger the normal safe-beat autosave path. A distinct map badge
  and filtered status legend keep manually paused, food-starved, and full-hold
  routes legible before inspection; an unstaffed awakened route also exposes
  `No scout crew` and a matching compact detail line. Each route persists its
  completed-haul count and scouted ore total for the next inspection, and the
  completed-campaign Objective summarizes aggregate `Runs` and `Hauls`.
- Endless Outpost hold expansion — pass; an active awakened route can purchase
  its one-time 8-ingot expansion, the save-compatible route flag preserves the
  purchase, and the upgraded 20-slot capacity is shared by transit, expedition
  room, load preview, full-hold status, and completed-campaign guidance. The
  [ui_endless_upgrade.png](../verification/ui_endless_upgrade.png) capture shows
  the enabled purchase control, while
  [ui_endless_upgraded.png](../verification/ui_endless_upgraded.png) shows the
  expanded `Cargo 6/20` hold and success toast.
- Endless crew dispatch quota — pass; a save-compatible optional quota keeps
  older routes on automatic dispatch, cycles the visible control through
  cargo-only and bounded scout counts, and limits new passengers in transit.
  The cargo-only state still permits a provisions or ore load, while the
  inspection readiness and completed-campaign Objective explain the quota
  when it is the reason no new scouts are being offered. Focused simulation,
  save-roundtrip, state, inspection, and Objective coverage passes, and the
  refreshed [ui_endless_load_preview.png](../verification/ui_endless_load_preview.png),
  [ui_endless_upgrade.png](../verification/ui_endless_upgrade.png), and
  [ui_endless_upgraded.png](../verification/ui_endless_upgraded.png) captures
  keep the adjacent route controls readable.
- Endless cargo-only returns — pass; the route can unload ore, ingots, or food
  to the shrine while preserving its stationed crew, so a remote team can
  continue scouting after a haul. Full returns still recall the crew, failed
  cargo-only trips recover the cargo without duplicating or losing remote
  ownership, and the departure notice, Objective, and field guide explain the
  distinction. Focused simulation, game-action, and inspection coverage plus
  refreshed [ui_endless.png](../verification/ui_endless.png),
  [ui_endless_expedition_report.png](../verification/ui_endless_expedition_report.png),
  and [ui_endless_upgraded.png](../verification/ui_endless_upgraded.png) captures
  verify the visible choices.
- Endless automatic cargo returns — pass; an awakened route can persist an
  opt-in `Auto-return · Cargo only` policy. When its remote hold reaches the
  shared capacity, the simulation starts a cargo-only shrine transit, preserves
  the stationed crew and one configured expedition's food when there is other
  cargo to unload, and flushes provisions as well when they alone fill the
  hold, leaving room for a later resupply. It emits a distinct notification
  and marks the departure as a safe-beat autosave. The global worm transit
  remains serialized across multiple routes, and the default-off field keeps
  older saves unchanged.
  Focused state, save-roundtrip, simulation, and game-notice coverage passes,
  with [ui_endless_auto_return.png](../verification/ui_endless_auto_return.png)
  showing the enabled touch control alongside the manual return and scouting
  actions.
- Endless automatic Outpost resupply — pass; an awakened staffed route can
  persist `Auto-resupply · Food only`. When remote provisions fall below one
  expedition cycle and the warren has food above reserve, the simulation sends
  a food-only transit without passengers, reports the departure, and preserves
  the player's manual scouting pause. Save-compatible defaults keep the policy
  off for older routes. Focused state, Objective, overlay, simulation, and
  notice coverage passes, with
  [ui_endless_auto_resupply.png](../verification/ui_endless_auto_resupply.png)
  showing the shortage state and enabled recovery policy.
- Endless transit failure recovery — pass; an in-flight route that loses its
  active Outpost now restores its payload and records the failure in the
  simulation report exactly once. The game announces the touch-first recovery
  action and marks the state change as a safe-beat autosave. Focused simulation
  and game-notice coverage pass, and the published Preview build includes the
  event path.
- Multi-route failure visibility — pass; reopening one failed Outpost no
  longer hides an unresolved failure on another route, and starting a healthy
  route does not clear that remaining warning. Focused coverage keeps the
  global banner and per-route failure records aligned across both recovery
  paths.
- Loaded route failure visibility — pass; loading a session now derives the
  global route warning from persisted per-Outpost failure records, repairing a
  missing banner and clearing a stale one. Focused recovery coverage verifies
  both load-boundary states, and the published Preview includes the update.
- Loaded Outpost record recovery — pass; load reconciliation now creates the
  missing route record for every placed Outpost building before remote-crew
  markers and HUD state are rebuilt. Focused state coverage verifies the
  inactive record is restored at the building tile, and the published Preview
  includes the update.
- Current-version save validation — pass; the game shell validates the session
  returned by the toolkit even when the current-version fast path skips the
  migration callback. Focused coverage rejects a malformed current-version
  payload before it can reach the live Warren. Remote Outpost cargo is also
  checked for arithmetic overflow and hold capacity, including cargo already
  stored at a route and payload currently carried by the worm. The same
  boundary rejects real remote crew over-capacity, including crew already
  stationed and passengers still arriving at a full route, and rejects a
  transit that predates Worm Awakening. The simulation also refunds such an
  impossible transit rather than delivering it if a state bypasses the load
  boundary.
  Local-only goods in a remote hold are rejected as unsupported instead of
  creating a route state that the transport actions cannot unload.
- Outpost route setting persistence — pass; successful route controls now
  trigger immediate autosaves, and the save-roundtrip coverage includes cargo
  priority and scouting pause alongside dispatch and automatic logistics
  policies. The published Preview build includes the updated action path.
- Worm Shrine offering policy persistence — pass; a successful visible
  Pause/Resume offerings action now triggers an immediate autosave, and the
  save-roundtrip coverage preserves the pause flag. The published Preview
  build includes the updated action path.
- Direct player decision persistence — pass; successful job reassignment,
  specialist recruitment, breeding, Blacksmith queueing, tutorial skip,
  milestone-report dismissal, building placement, and dig designation now
  trigger immediate autosaves. Failed or duplicate actions do not write, and
  save-roundtrip coverage preserves the resulting job, queue, map, and
  decision flags. The published Preview build includes the updated action
  path.
- Progression checkpoint persistence — pass; captures, unlock grants,
  breeding-pit hatches, survived raids, and tutorial advancement now mark the
  existing safe-beat autosave boundary. Focused coverage verifies the
  progression report classification, and the published Preview includes the
  updated path.
- Multi-Outpost automatic scheduling — pass; automatic cargo returns and
  food-only resupplies use the same save-compatible round-robin cursor. A
  successful service advances the next starting route, an in-flight worm keeps
  all other routes waiting without changing the cursor, and a route whose
  transit cannot start does not consume its turn. Focused multi-route simulation
  and save-roundtrip coverage passes.
- State-aware awakened objective — pass; focused coverage names the visible
  Build & Dig or Jobs recovery for a missing Blacksmith, exhausted Mine, missing
  Smith, and missing Carrier instead of promising more ingots without a viable
  production chain. Refreshed [ui_worm.png](../verification/ui_worm.png) and
  [ui_completion.png](../verification/ui_completion.png) captures show the
  Blacksmith recovery prompt, while [ui_endless_forge.png](../verification/ui_endless_forge.png)
  shows live `37/60` Worm Transit progress once the chain is viable. Pending
  Blacksmith and replacement Mine build sites also keep their hauling guidance
  instead of asking for duplicate structures.
- Multi-route objective ordering — pass; focused coverage selects a ready or
  loadable active outpost before an empty active route, preserving an actionable
  next step when more than one existing outpost is in play.
- Project-local capture wrapper — pass; `scripts/capture_ui.ps1` now forwards
  viewport sizing and release/visible capture options to the shared toolkit,
  and its 800×450 completion/Endless path was exercised successfully.
- Settled touch-target audit — pass; `scripts/audit_touch_targets.ps1` walks
  the title, settings, confirmation, core HUD, crafting, shrine, completion,
  Outpost, and multi-route ledger scenes at 800×450 after the neighbor map is
  warm. Enabled controls are measured, modal occlusion removes covered HUD
  controls, and the sweep reports no grown-target overlap. The compact branch
  now draws the top-bar controls at 40 logical pixels and the Jobs, Build & Dig,
  Outpost route, Blacksmith, and Breeding Pit controls at 30 logical pixels or
  more; the craft and specialist cards report 36-pixel drawn targets with no
  grown-target overlap. Remaining text density and first-time-player
  comprehension remain honest human follow-ups.
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
- Worm Shrine reserve-stall capture — [ui_shrine_waiting.png](../verification/ui_shrine_waiting.png)
  shows the `Food reserve low` map badge and filtered legend beside the
  inspection card's matching `Waiting for food reserve` state.
- Optional support capture — [ui_optional.png](../verification/ui_optional.png)
  shows active local support as `Beetle x1 haul` and `Salam x1 forge`, while
  the still-available `Slime · clean` and `Bat · 8 cargo` recruit controls name
  their practical actions. The Engineer mine-throughput bonus remains visible
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
  with cargo mix, capacity, crew, directional transit controls, and a clear
  full-hold return hint.
- Load forecast capture —
  [ui_endless_load_preview.png](../verification/ui_endless_load_preview.png)
  shows the selected `Ingots first` order, the exact next Ore/Ingots/Food mix,
  and a staffed expedition progressing toward its next remote ore haul.
- Automatic return policy capture —
  [ui_endless_auto_return.png](../verification/ui_endless_auto_return.png)
  shows `Auto-return · Cargo only` enabled while the manual cargo-only return,
  cargo order, crew quota, hold upgrade, and Pause scouting controls remain
  visible.
- Automatic resupply capture —
  [ui_endless_auto_resupply.png](../verification/ui_endless_auto_resupply.png)
  shows a staffed route short on provisions with `Auto-resupply · Food only`
  enabled, the Objective's wait instruction, and the visible return/load and
  Pause scouting controls.
- Paused expedition capture —
  [ui_endless_expedition_paused.png](../verification/ui_endless_expedition_paused.png)
  shows the touch-first `Resume scouting` control, the persisted player-paused
  hint, the matching `Scouting paused` map/legend status, and the Objective
  instruction while the route remains active.
- Expedition feedback capture —
  [ui_endless_expedition_report.png](../verification/ui_endless_expedition_report.png)
  shows the fully readable `Outpost haul · +6 ore / -2 food.` toast after a
  completed scouting cycle, alongside the persisted `Scouted ore 6 · Hauls 1`
  route record.
- Empty route capture — [ui_endless_empty.png](../verification/ui_endless_empty.png)
  shows an active awakened outpost with `Status · Awaiting payload`, `Need scout
  crew`, disabled route actions, and the explicit `No payload ready at warren`
  recovery state, with a readable gap separating the recovery note from the
  action button. The map badge and compact legend also expose `No scout crew`.
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
- Reachable construction placement — focused state coverage keeps the
  stockpile's worker path as part of the placement predicate, rejects an
  isolated floor pocket, and checks the restored spawn exits across 128
  deterministic seeds. The existing touch build ghost uses the same predicate,
  so unreachable floor reads as invalid before the player taps it.
- Legacy route recovery — older saves with a disconnected worker-serviced
  building remain loadable but now expose the shared `No valid route` map badge,
  filtered legend entry, and inspection instruction to `Dig a tunnel to
  reconnect this node`. The deterministic [ui_unreachable_workstation.png](../verification/ui_unreachable_workstation.png)
  capture shows the complete recovery message without requiring a malformed new
  placement.
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
  for optional cargo runs after the worm wakes. Its visible `Recent events`
  action opens the new notification-history surface shown in
  [ui_event_log.png](../verification/ui_event_log.png); the compact
  [ui_compact_event_log.png](../verification/ui_compact_event_log.png) probe
  keeps the same review and close actions on-canvas.
- Persistent notification history — pass; `GameSession` carries the bounded
  event log with a serde default for older saves, save/autosave checkpoints
  copy the live history, and load rehydrates review history without replaying
  old toasts while normalizing oversized loaded histories back to the toolkit
  limit. A pre-history save with the field absent loads with an empty log, and
  starting a genuinely new Warren clears the previous run's log.
- Reviewable notification history — pass; the Recent Events modal pages the
  full bounded history in ten-entry slices, keeps the first page newest-first,
  and exposes visible Older/Newer controls only when another page exists.
  Refreshed [ui_event_log_older.png](../verification/ui_event_log_older.png) and
  [ui_compact_event_log_older.png](../verification/ui_compact_event_log_older.png)
  captures show the older-page recovery path, and the published Preview
  roundtrip exercised Older then Newer with visible controls.
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
  Objective incremented to `Runs 1 · Hauls 0`; the arrival is persisted through the
  normal safe-beat autosave path, and the real-loop toast accurately says
  `crew delivered` for this crew-only trip.
- Route guidance captures — [ui_endless.png](../verification/ui_endless.png),
  [ui_endless_load_preview.png](../verification/ui_endless_load_preview.png),
  [ui_endless_expedition_paused.png](../verification/ui_endless_expedition_paused.png),
  [ui_endless_expedition_report.png](../verification/ui_endless_expedition_report.png),
  [ui_endless_in_flight.png](../verification/ui_endless_in_flight.png), and
  [ui_endless_arrived.png](../verification/ui_endless_arrived.png), and
  [ui_endless_empty.png](../verification/ui_endless_empty.png) show the
  Objective and inspection guidance changing from mixed cargo-and-crew return,
  to wait for transit, to return crew, to preparing a new payload.
- Multi-route ledger capture — [ui_endless_routes.png](../verification/ui_endless_routes.png)
  shows the post-awakening Routes control opening a paused ledger with two
  route cards, the network summary line, shared status labels, hold/crew counts,
  automatic-policy lines, and visible Inspect/Close targets. The summary gives
  the network totals for active routes, held cargo, remote crew, scouted ore,
  and attention states before a card is opened. A compact 800×450 probe kept
  the same controls on-canvas without overlap.
- World-space route capture — [ui_endless.png](../verification/ui_endless.png)
  shows the active shrine-to-Outpost link behind the awakened landmark, while
  [ui_endless_in_flight.png](../verification/ui_endless_in_flight.png) shows the
  return pulse at the remote endpoint and the existing directional transit
  copy. The deterministic scene keeps the route endpoints separated so this
  feedback remains reviewable in the canonical world view.
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
  autosave warning and now identifies whether the last checkpoint remains
  available when it occurs.
- The five-player comprehension and completion targets remain unmeasured.
