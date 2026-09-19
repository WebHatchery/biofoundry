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
- [ ] Enlarge the small Build & Dig, tutorial, and inspection controls in
  `src/ui/hud/panels.rs`, `panels/tutorial.rs`, and `inspect/layout.rs`.
  Address the 22–40-pixel targets recorded in
  `docs/verification/pointer-touch-acceptance.md`; measure effective touch
  targets after virtual-resolution scaling at desktop and compact sizes (§7.5).
- [ ] Complete a live pointer/touch-only browser acceptance pass at 1280×720
  and 800×450: start, every tutorial step, building, routes, pan/zoom, and
  save/load recovery. Fix any inaccessible controls or unclear instructions,
  update `docs/verification/pointer-touch-acceptance.md` with observed results,
  and replace same-state screenshots directly in `docs/verification/` (§§7.5, 12).
