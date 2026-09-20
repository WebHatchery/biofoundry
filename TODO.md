# Biofoundry TODO

- [ ] Finish moving player-facing labels, notifications, and dynamic message
  templates into typed JSON (`CODE_STANDARDS.md` §5.3). Start with
  `src/game/messages.rs`, `src/game/persistence.rs`, `src/ui/menu.rs`, and
  `src/ui/hud/`; include the tutorial overrides in `panels/tutorial.rs` and
  derive their numeric goals from balance data. Extend required-ID and
  template validation, and correct `docs/COPY_AUDIT.md` to reflect the scope.
- [ ] Refactor functions exceeding the 100-line maximum into cohesive helpers
  (§4.1). Prioritize `game_actions::apply_action` (614 lines),
  `inspect::draw::draw_inspect_panel` (457),
  `persistence::session_validation::validate_loaded_session` (346), and
  `panels::draw_top_bar` (316); include the large capture-scene dispatchers
  and remaining simulation, update, and rendering functions. Preserve behavior
  and use context structs for long argument lists such as `draw_outpost_details`.
- [ ] Plan cohesive splits before expanding the remaining implementation
  modules above 600 lines (§2.2): `src/simulation/outposts.rs` (743),
  `src/data.rs` (741), `src/game_actions.rs` (698),
  `src/ui/hud/inspect/outpost.rs` (681), and `src/state.rs` (676).
- [ ] Review suites against the strong target of **no more than five cases
  per major feature** (`CODE_STANDARDS.md` §11.3). Consolidate related inputs and
  document concrete reasons for retained exceptions without losing regressions.
  Split near-limit suites by responsibility: `tests/unit/ui_hud_objective.rs`
  and `ui_hud_inspect.rs` (798 lines each), `game.rs` (790), and
  `simulation/novel.rs` (780); splitting files alone does not meet the target.
- [ ] Remove the unused `data` parameter and `let _ = data` suppression from
  `src/simulation/outposts.rs::tick_transit`; update production and test callers
  (§1.4).
- [ ] Handle audio and settings errors in `src/audio.rs` (§6.2): log failed
  sound loads and settings writes, and distinguish absent settings from corrupt
  or unreadable settings while preserving graceful fallback.

## UI_STYLE review — 2026-09-20

Audit baseline: `f702512` on master, with the updated shared documentation
already present in the working tree. This is a planning pass; no runtime UI
changes or new captures were made. The existing maintenance tasks above are
retained; their two touch-polish/acceptance tasks are merged into UI-05,
UI-07, and UI-10 below. Existing completion history remains in
`docs/phase-one/PRODUCTION_PLAN.md` and the candidate verification record;
completed historical checks are not evidence that the current UI passes the
new composition rules.

Read AGENTS.md, UI_STYLE.md, CODE_STANDARDS.md, GAME_DEVELOPMENT_GUIDE.md,
README.md, and the Phase One product brief, playable slice, production plan,
and playtest plan. The README identifies these as the current design authority;
there is no project-local PROJECT_AGENTS.md or separate current GDD.

Evidence reviewed directly: `docs/verification/ui_warren.png`,
`ui_hud_food.png`, `ui_compact_hud_build.png`, `ui_tutorial_food.png`,
`ui_compact_tutorial_factory.png`, `ui_menu.png`, and
`material-storage-compact.png`. These show the drawer-based HUD represented
in current code, although the build capture predates the Stockpile addition.
Also inspected `ui_endless_routes_busy.png`,
`ui_compact_endless_deep_survey.png`, and `ui_touch_audit_endless_routes.png`:
their simultaneous Food/Jobs/Build/Objective dashboard is historical and must
not be reported as the current normal-play layout. Current compact Outpost
code uses a full-height sheet, unlike those older captures.

The normal/minimum review targets below are **1280×720 and 800×450 actual
game-canvas pixels**, based on existing verification practice. Explicitly
declare them in the screen brief rather than treating the 1280×720 virtual
resolution as a support guarantee. Check 1024×576 and 1440×900 for reflow and
letterboxing too. Historical browser evidence includes a 1200×675 canvas in
a 1280×720 browser window; record both dimensions in future verification.

### Verified findings and implementation tasks

Tasks are ordered by player impact and layout dependencies. UI-01 sets the
composition contract; UI-02–04 establish its shell and interaction model;
UI-05–09 refine the affected views. Keep rendering as UI intents, reuse
toolkit layout/camera/pointer helpers, and coordinate copy changes with the
typed-JSON task above. Do not recreate the old permanent dashboard.

- [ ] **UI-01 — Keep the next decision and food health visible without reopening ledgers.**
  **Screen/files:** fresh and resumed Warren, all campaign beats;
  `src/ui/hud.rs::draw`, `hud/dock.rs::draw_command_strip`,
  `hud/panels.rs::draw_objective_panel`, `hud/panels/food.rs`,
  `src/game/persistence.rs::reset_session_view_state`, README.md.
  **Observed:** `hud_panel` starts at None and Goal/Food only render when
  explicitly opened. `ui_warren.png` shows no next objective or food trend;
  its small FOOD/ORE/INGOTS box gives all three equal status. The Food drawer
  repeats bank totals and adds ore/ingot throughput even in the opening.
  This conceals the campaign direction and central food constraint described
  in PLAYABLE_SLICE.md; the large map alone does not explain what to do next.
  **Change:** first record the UI_STYLE §1 screen brief for onboarding,
  normal campaign, crisis, inspection, and route planning. Compose a single
  compact support area with current objective/next requirement and cooked
  reserve plus trend (time-to-empty when relevant). Let its visible actions
  open Goal details, Food, Jobs, or Build as appropriate. Keep raw ingredients
  and throughput in contextual detail; move industrial rates out of Food and
  give banked resources one clear home beside spending decisions. Remove
  redundant nested resource-box borders and duplicate headings as part of
  this composition, not as a separate cosmetic pass.
  **Acceptance/verify:** at both target canvas sizes, fresh, resumed,
  tutorial-skipped, secured, and pre-Shrine states communicate one next goal
  and food health without opening a drawer. World, decision support, and
  current contextual view consume at most 2–3 strong attention regions;
  utilities remain quiet. Tap from the next requirement to its actual action
  and back; costs, shortages, and current reserves remain available.

- [ ] **UI-02 — Separate gameplay controls and warnings from save/menu utilities.**
  **Screen/files:** normal/paused Warren and Endless;
  `hud/panels.rs::draw_top_bar`, `hud/dock.rs`, `hud/widgets.rs`,
  `src/game_actions.rs`, `hud/overlays.rs`.
  **Observed:** the full-width top surface groups population, warnings,
  Pause, Help, zoom, Save, Load, and Menu with equal button treatment.
  Endless also places Routes here even though the dock already has Routes.
  In current code, the status text starts at x=392 without a measured right
  boundary; Routes begins at x=662, so additional controls reduce warning
  space. Actual dense-state overflow needs a fresh capture (see UI-10).
  **Change:** retain one Routes entry in the gameplay area; place pan/zoom
  and simulation controls with the world. Move save/load/settings/menu into
  a visibly separate, quiet utility entry with a clear return path. Keep
  checkpoint failures and recovery access visible when unresolved. Allocate
  measured warning space independently of utility buttons, reflowing long
  text rather than painting it underneath controls. Remove the permanent
  “Drag map · +/− zoom · tap Help” banner after first-use teaching.
  **Acceptance/verify:** at both sizes, gameplay actions never share a visual
  group with save/exit/settings. Exercise Pause/Resume, zoom, Routes, Save,
  Load confirmation/cancel, failed-save recovery, and Menu/Continue by taps.
  Check simultaneous raid/food pressure and checkpoint failure; no essential
  warning or recovery action is hidden by the utility layout.

- [ ] **UI-03 — Frame the working colony and visibly retain building selection.**
  **Screen/files:** initial Warren, selected workstations, route-to-map handoff;
  `src/game.rs::{reset_camera_for,focus_camera_on_tile}`,
  `src/game/input.rs`, `src/game/render.rs`, `src/ui/warren.rs::draw_world`,
  `ui/warren/sprites.rs`, `ui/warren/buildings.rs`.
  **Observed:** the opening capture gives the world most of the screen, but
  most of that is rock around a small central workplace. Reset always centers
  the spawn at zoom 1.0; HUD route selection centers on the full screen rather
  than the unobscured play area. `draw_world` receives mode and hover, but no
  selected tile, so inspection lacks a persistent world-selection indicator.
  **Change:** derive opening framing from the starter production cluster and
  usable world rectangle; tune default zoom for recognizable workers, cargo,
  and buildings without losing nearby expansion sites. Frame selected nodes
  clear of the open inspector and add a persistent outline/marker matching
  the inspected object. Provide a visible return-to-colony view control;
  preserve manual panning and zoom instead of continuously recentering.
  **Acceptance/verify:** compare normal/minimum opening captures, Mine,
  Blacksmith, Stockpile, Shrine, and distant Outpost selection. A player can
  identify the selected node and follow a carrier visually. Tap targets map
  correctly after resize, zoom extremes, and display scaling; pan away and
  return without a keyboard. Tune the final zoom in play, not from code alone.

- [ ] **UI-04 — Keep active placement/dig mode explicit and cancellation visible.**
  **Screen/files:** Build & Dig, switching drawers while a tool is armed;
  `src/game_actions.rs::{SetMode,ToggleHudPanel}` action arms,
  `src/game.rs::world_click`, `hud/panels.rs::draw_tools_panel`,
  `hud/dock.rs`, `src/ui/warren.rs::draw_tool_ghost`.
  **Observed (code):** toggling or switching the drawer leaves Build/Dig mode
  armed; its active marker lives inside the now-hidden palette. Help explains
  tapping the active tool again, but no always-visible Cancel/Inspect control
  exists. A map tap can therefore build or mark rock when the player expects
  to inspect. Successful placement already returns to Inspect; retain that.
  **Change:** show a concise active-tool strip with selected building cost,
  placement instruction, and Cancel/Done while a tool is armed. Define drawer
  changes to cancel the tool or explicitly preserve that visible strip.
  Distinguish invalid placement reasons such as unaffordable cost, occupied
  ground, and invalid terrain instead of only “Can't build there.”
  **Acceptance/verify:** at both sizes, tap Build → Farm, close/switch drawer,
  inspect a Mine, cancel an invalid placement, and enter/leave Dig. No hidden
  mode causes an unintended action; dragging and pinch releases never place
  a building or activate a control. Verify this path live; no interaction
  failure was reproduced during this audit.

- [ ] **UI-05 — Reflow Build and Jobs around the current choice and touch targets.**
  **Screen/files:** early Build, post-security Jobs/Build;
  `hud.rs` drawer rectangles, `hud/panels.rs::draw_tools_panel`,
  `hud/panels/jobs.rs`, `hud/panels/specialists.rs`, `hud/widgets.rs`.
  **Observed:** the compact Build capture puts Locked gates text over the Dig
  row: the row grows to 72 logical pixels while its notes remain at y+36/y+52.
  Current code retains those offsets. Jobs uses a fixed compact height even
  when optional specialist rows and gate explanations are appended. Desktop
  Build/Jobs controls remain 22/26 logical pixels tall. All advanced building
  and recruitment choices appear together after the single security gate,
  competing with the next campaign step.
  **Change:** calculate content height from actual rows; use deliberate
  scrolling/paging within a bounded drawer before shrinking controls. Group
  core construction by player purpose rather than alphabetical ID; expose
  optional buildings/recruitment through visible categories or disclosure.
  Put each prerequisite next to its selected/locked action, removing the
  detached list of every locked gate. Keep cost units explicit and keep
  worker counts/idle availability beside assignment controls.
  **Acceptance/verify:** both sizes show readable labels, cost/reason, and
  Dig without collision. Measure drawn and effective touch targets, aiming
  for at least 44 actual pixels with unambiguous neighboring hit areas.
  Exercise Farm/Stockpile placement, Miner→Carrier and Smith→Guard changes,
  optional recruit unlocks, and dense Jobs with crowding. All content stays
  inside the drawer and off the command dock; no optional prerequisite is lost.

- [ ] **UI-06 — Teach one immediate action and route players to the visible control.**
  **Screen/files:** all five onboarding beats;
  `hud/panels/tutorial.rs::{draw_tutorial_panel,tutorial_body}`,
  `assets/data/tutorial.json`, `hud/objective.rs`, `hud/dock.rs`.
  **Observed:** food teaching says “Read Food Grid” and “Tap Farm” while only
  the Tutorial drawer is open; opening Food/Build replaces the tutorial.
  Secure copy calls the destination Objective while the dock says Goal.
  The compact Factory capture cuts the long body after “tap Iron”; its body
  still has a fixed 94 logical pixels even when the outer card grows.
  **Change:** replace multi-action paragraphs with the current substep and a
  visible action that opens the required drawer or focuses the workstation.
  Use exact paths such as “Build & Dig → Farm” and “Jobs → + Carrier.” Keep
  the short current prompt associated with the action from UI-01; put longer
  explanations in reopenable Guide. Preserve Close versus Skip semantics,
  completed-step dismissal, and prerequisite/shortage instructions.
  **Acceptance/verify:** run each beat by taps at both sizes, including a
  player with no idle workers. Prompt, relevant cost, and target can be read
  together without truncation; completing the step removes its prose. Close,
  reopen, Skip, and resumed-save states remain understandable.

- [ ] **UI-07 — Make inspectors readable, dismissible, and progressively detailed.**
  **Screen/files:** Blacksmith, breeding, Shrine, Stockpile, Outpost;
  `hud/inspect/{draw,layout,blacksmith,draw_outpost}.rs`,
  `hud/inspect/outpost.rs`, `hud/inspect/outpost/compact_controls.rs`.
  **Observed (code; old late-game captures are not current proof):** compact
  Blacksmith puts the entire recipe catalog into two columns inside a
  250-logical-pixel card, with 46-pixel-high recipe buttons (about 29 actual
  pixels at 800×450). Every recipe, including late locked gear, is enumerated.
  Desktop Outpost stacks status/history/policies/upgrades into a fixed right
  column. Compact Outpost already uses a full-height sheet, but retains long
  joined summary lines. Only that compact sheet has an explicit Close button.
  **Change:** make the chosen building's current blocker, inventory/queue,
  and available action the primary inspection content. Give recipes a wider,
  scrollable or paged chooser; reveal later gear by milestone/category with
  discoverable unlock details. Split Outpost current operations from automation,
  upgrades, and historical totals using visible sections. Keep exact load
  forecast, food/crew costs, reserve policy, and return/reactivation recovery
  beside the affected action. Provide Close on every inspector and size the
  content to the available rectangle rather than fixed kind-only heights.
  **Acceptance/verify:** at both sizes, queue Iron Pickaxe with locked later
  gear and with a full queue; change empty Stockpile material; pause/resume
  Shrine; recruit a specialist; return cargo/crew and recover an inactive
  Outpost. Labels and prerequisites are readable and targets meet UI-05's
  measurements. A deliberate compact management sheet may dominate its phase,
  but closing it restores the world and preserves a clear selection.

- [ ] **UI-08 — Keep urgent state separate from event feedback and optional history.**
  **Screen/files:** calm, food/raid crisis, route failure, save failure;
  `hud/panels.rs::draw_top_bar`, `hud/overlays.rs::draw_status_legend`,
  `src/game/render.rs`, `src/game/notifications.rs`, `hud/inspect/draw.rs`.
  **Observed:** the banner's exclusive priority chain places checkpoint
  warning, pause, transit, and unresolved route failure before raid/famine.
  This can suppress the visible current food/raid warning (code finding).
  The compact tutorial/build captures also show a large truncated autosave
  toast in the world plus the persistent save-failure banner. Existing event
  history and unresolved route-failure reconciliation are useful and should
  remain; do not misclassify an unresolved fault as a stale event.
  **Change:** compose current safety state with a concise actionable food/raid
  warning even when a save/route issue exists. Place route recovery beside
  Routes/its inspector and checkpoint recovery with utilities. Keep success
  deltas temporary and retrievable in Recent Events. Derive toast placement
  from the active layout instead of the fixed screen-space (-250,-82) offset;
  deduplicate repeated failure notices without hiding an unresolved fault.
  Collapse the status legend into contextual explanations when long; retain
  shaped world badges and a tap-accessible way to learn their meaning.
  **Acceptance/verify:** at both sizes exercise raid+low food, save failure+
  famine, route failure+food warning, and several simultaneous haul/unlock
  events. Current danger and recovery remain visible; expired feedback leaves
  understandable current values/history. Inspect every badge without hover,
  and ensure toasts do not cover the selected node, costs, or primary action.

- [ ] **UI-09 — Give starting/continuing the warren priority on the title screen.**
  **Screen/files:** title with/without save, settings and replacement warning;
  `src/ui/menu.rs::{draw,menu_button,draw_start_new_warren_confirmation}`.
  **Observed:** `ui_menu.png` and the entries loop give New Warren, Continue,
  Settings, and Exit Game identical green bordered boxes and spacing. The
  tailored title art already communicates the game; utility emphasis dilutes
  its otherwise clear start action. No unadapted template copy was found.
  **Change:** emphasize New Warren for a fresh player and Continue for a
  returning player; keep the other start option clearly available. Relocate
  Settings/Exit to a quiet, separated utility group. Preserve confirmation
  before replacing a save and keep the existing themed tableau.
  **Acceptance/verify:** normal/minimum title captures with and without a
  save have an obvious next action. Tap through Settings/Done, New Warren/
  Keep Save, Continue, and platform-appropriate Exit; no utility visually
  outranks entering play and confirmation text remains readable.

### Further inspection and verification gate — not verified defects

- [ ] **UI-10 — Refresh dense-state evidence and complete live touch acceptance.**
  **Screen/files:** current capture harness `src/game/capture_scenes/`,
  `src/ui/hud/routes.rs::draw_route_overview`, game input/render/layout files
  above, `docs/verification/pointer-touch-acceptance.md` and Phase One evidence.
  **Evidence gap:** no game launch, fresh scene capture, browser interaction,
  or physical touch test was performed in this audit. Existing reports mix
  historical live checks with capture/target audits; the latter do not prove
  readability or successful touch paths. In particular, the route ledger
  clamps panel height but continues drawing every route and accumulated
  milestone summary without paging. Determine the reachable worst case before
  claiming a current overlap. Also inspect expanded compact Jobs/Outpost,
  many simultaneous legend statuses, and maximum supported label/value lengths.
  **Action:** capture those actual supported states at 1280×720 and 800×450;
  if content exceeds available space, add scrolling/paging and contextual
  summaries with fixed reachable Close/recovery controls as part of UI-05/07.
  Exercise 1024×576, 1440×900, browser embedding and display scaling. Perform
  New Warren → every tutorial beat → build/dig/craft → Shrine completion →
  Endless/Routes → Menu/Continue entirely through visible controls. Include
  pan/pinch, drawer switching with armed tools, modal dismissal, Save/Load
  cancellation, incompatible saves, and failed writes with recovery.
  **Acceptance:** record candidate revision, actual canvas and browser sizes,
  input method, scenes, observed outcomes, and remaining limitations. Replace
  equivalent captures directly in `docs/verification/`; preserve historical
  completion records and identify superseded images. Apply every UI_STYLE §9
  check, including attention budget and post-toast comprehension, then run
  `publish.ps1` without parameters after meaningful UI implementation and
  report its result. Automated target checks supplement live touch evidence;
  first-time-player comprehension remains the separate PLAYTEST_PLAN gate.
