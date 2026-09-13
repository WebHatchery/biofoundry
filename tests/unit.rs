//! Integration coverage grouped by the public domain contract it exercises.
//!
//! The suite keeps the former responsibility-based files together while
//! compiling them against `biofoundry` as an external crate.

#[path = "unit/data.rs"]
mod data;
#[path = "unit/game.rs"]
mod game;
#[path = "unit/game_actions.rs"]
mod game_actions;
#[path = "unit/game_input.rs"]
mod game_input;
#[path = "unit/game_persistence.rs"]
mod game_persistence;
#[path = "unit/simulation.rs"]
mod simulation;
#[path = "unit/simulation_food.rs"]
mod simulation_food;
#[path = "unit/simulation_jobs_equipment.rs"]
mod simulation_jobs_equipment;
#[path = "unit/state.rs"]
mod state;
#[path = "unit/state_creatures.rs"]
mod state_creatures;
#[path = "unit/state_outposts.rs"]
mod state_outposts;
#[path = "unit/state_structures.rs"]
mod state_structures;
#[path = "unit/state_world.rs"]
mod state_world;
#[path = "unit/tutorial.rs"]
mod tutorial;
#[path = "unit/ui.rs"]
mod ui;
#[path = "unit/ui_hud.rs"]
mod ui_hud;
#[path = "unit/ui_hud_inspect.rs"]
mod ui_hud_inspect;
#[path = "unit/ui_hud_inspect_layout.rs"]
mod ui_hud_inspect_layout;
#[path = "unit/ui_hud_inspect_study.rs"]
mod ui_hud_inspect_study;
#[path = "unit/ui_hud_objective.rs"]
mod ui_hud_objective;
#[path = "unit/ui_hud_overlays.rs"]
mod ui_hud_overlays;
#[path = "unit/ui_hud_panels.rs"]
mod ui_hud_panels;
#[path = "unit/ui_hud_panels_food.rs"]
mod ui_hud_panels_food;
#[path = "unit/ui_hud_requirements.rs"]
mod ui_hud_requirements;
#[path = "unit/ui_hud_routes.rs"]
mod ui_hud_routes;
#[path = "unit/ui_legibility.rs"]
mod ui_legibility;
#[path = "unit/ui_menu.rs"]
mod ui_menu;
#[path = "unit/ui_warren_routes.rs"]
mod ui_warren_routes;
#[path = "unit/ui_warren_sprites.rs"]
mod ui_warren_sprites;
