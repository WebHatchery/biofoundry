# Current Candidate Verification

**Date:** 2026-08-30  
**Source revision:** `60d08a9`
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

## Automated candidate checks

- `cargo fmt -- --check` — pass.
- `cargo test --all-targets` — 107 unit tests and 2 integration tests pass.
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
- Endless route recovery — automated coverage confirms food-only delivery and
  crew-only return remain valid, so an outpost cannot strand its crew when its
  cargo hold is empty; a full remote crew cannot be duplicated by another load.
- Endless route accounting — fractional local food remains in the warren until
  a whole cargo unit is ready, preventing loss when a route is loaded.
- Factory handoff capture — [ui_factory_complete.png](../verification/ui_factory_complete.png)
  shows the completion overlay with the next Worm Shrine objective visible and
  no setup unlock toast obscuring the handoff.
- Save-state coverage — an in-flight Worm Transit survives a session roundtrip
  with its destination, payload, passengers, and active route state intact.
- Transit launch autosave — successful route departures now persist immediately,
  so a refresh during the worm's journey does not erase the in-flight state.
- Menu-exit autosave — leaving through the visible Menu control preserves a
  viable Warren for Continue, while the field guide still exposes manual Save.
- Specialist capture — [ui_breeding.png](../verification/ui_breeding.png) shows
  the selected Breeding Pit with readable Hobgoblin, Overseer, and Engineer
  effects and available ingot costs.
- Factory tutorial capture — [ui_tutorial_factory.png](../verification/ui_tutorial_factory.png)
  makes the intended interaction explicit: inspect the existing Mine in the
  warren before placing the Blacksmith.
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
  normal safe-beat autosave path.
- Route guidance captures — [ui_endless.png](../verification/ui_endless.png),
  [ui_endless_in_flight.png](../verification/ui_endless_in_flight.png), and
  [ui_endless_arrived.png](../verification/ui_endless_arrived.png) show the
  Objective changing from return cargo, to wait for transit, to return crew.

## Still open

- A first-time pointer/touch-only campaign through Worm Awakened has not been
  observed by a qualifying player.
- Windows and WebGL completion, endless continuation, and return-to-menu flows
  still need live packaged-build evidence rather than capture or code evidence.
- Browser storage-quota/blocked-storage behavior has not been forced in a live
  session; the runtime surfaces the shared storage rejection as a save or
  autosave warning when it occurs.
- The five-player comprehension and completion targets remain unmeasured.
