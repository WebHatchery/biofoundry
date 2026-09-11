# Biofoundry TODO

This backlog records work identified while reviewing the updated `AGENTS.md`
and `CODE_STANDARDS.md` on 2026-09-12. It focuses on standards gaps found in
the current project rather than repeating items that already pass validation.

## P0 — standards migrations

- [ ] Move the legacy unit and integration tests out of `src/` and into the
  crate's `tests/` directory. The current tree contains 92 source files with
  test declarations and 515 legacy test functions. Preserve useful coverage,
  split suites by responsibility, and remove the corresponding `#[cfg(test)]`,
  `mod tests`, and path-based test wiring from implementation modules.
- [ ] Add `src/lib.rs` as the public test seam for the binary-only crate, then
  make the migrated tests exercise the public game, data, simulation, state,
  and UI contracts. Keep implementation details private unless an intentional
  public seam is required.
- [ ] Reconcile `GAME_DEVELOPMENT_GUIDE.md` with
  `rust_management/docs/GAME_DEVELOPMENT_GUIDE.md`. The project copy currently
  adds the `roost_slug` example field, so either promote that documentation
  change to the canonical source and sync it or remove the local-only drift.
  The shared `check-project-docs.ps1` check must pass afterward.

## P1 — structure and maintainability

- [ ] Split the implementation files already at or above the 600-line
  planning threshold before adding more behavior. Start with
  `src/game.rs` (800 lines), `src/ui/hud/panels.rs` (789),
  `src/game/capture_scenes/endless.rs` (791), `src/game/persistence.rs`
  (757), `src/game/capture_scenes.rs` (754), `src/ui/hud/routes.rs` (763),
  `src/ui/hud/inspect.rs` (761), and `src/ui/hud/objective.rs` (721).
  Keep each extracted module cohesive and preserve the empty exception list in
  the source gate.
- [ ] Add a short `//!` module-purpose comment to the five implementation
  modules that currently lack one: `src/simulation/outposts/milestones.rs`,
  `src/ui/hud/inspect/layout.rs`, `src/ui/hud/inspect/study.rs`,
  `src/ui/hud/panels/top_bar.rs`, and `src/ui/hud/panels/workforce.rs`.
- [ ] Review test suites after migration against the five-case target per
  major feature. Current umbrella suites are well above that target, including
  `src/game/tests.rs` (44), `src/ui/hud/inspect/tests.rs` (40),
  `src/ui/hud/routes/tests.rs` (37), `src/ui/hud/panels/tests.rs` (31),
  `src/state/tests.rs` (31), `src/ui/hud/objective/tests.rs` (32), and
  `src/simulation/tests/novel.rs` (27). Consolidate related inputs with
  table-driven assertions where that improves signal; retain and explain
  distinct regression coverage when five cases are not enough.
- [ ] Correct the description at the top of `tests/code_standards.rs`: it says
  the gate checks “non-test lines”, while the toolkit gate counts every
  physical line, including tests, comments, attributes, and whitespace.

## P1 — data and player-facing copy

- [ ] Audit player-facing strings still embedded in Rust and move them into
  typed JSON content/localization data under `assets/`, loaded through the
  toolkit. The first areas are `src/ui/menu.rs`, `src/ui/hud.rs`,
  `src/ui/legibility.rs`, `src/ui/hud/panels/`,
  `src/ui/hud/inspect/`, and status/notification builders in `src/game.rs`.
  Keep IDs, state keys, and semantic validation in Rust while making the copy
  data-driven.
- [ ] Add schema and semantic validation for the new copy tables: required
  message IDs, references used by UI actions and tutorials, and no missing
  player-facing strings at startup.

## P2 — verification and workflow

- [ ] Run a pointer/touch-only browser acceptance pass at common desktop and
  compact sizes covering start, every tutorial step, core building and route
  interactions, and save/load recovery. Replace duplicate captures in
  `docs/verification/` and record any remaining keyboard-only or unclear
  instruction in the relevant production documentation.
- [ ] Add the project standards drift check to the repository workflow so a
  local copy can no longer diverge silently from the canonical documents.

## Already verified during this review

- The project copies of `AGENTS.md`, `CODE_STANDARDS.md`, and
  `MACROQUAD_TOOLKIT.md` match their canonical documents.
- Game JSON is embedded and parsed through `macroquad_toolkit` with
  project-local semantic validation; no project-local generic JSON loader was
  found.
- `publish.ps1`, `game_page.json`, and the root `catalog_thumbnail.png` are
  present, and no `mod.rs` files or Rust files over the 800-line hard limit
  were found.
