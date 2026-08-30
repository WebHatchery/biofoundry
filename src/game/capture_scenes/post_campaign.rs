//! Capture scenes for the awakened worm and its post-campaign route loop.

use super::super::{format_expedition_completion, Game};
use crate::simulation;
use crate::state::creatures::Good;
use crate::state::outposts::CargoPriority;
use crate::state::structures::Building;
use crate::state::{GameState, StateTransition};
use macroquad_toolkit::grid::TilePos;

/// Seed one of the post-campaign scenes. Returning `true` keeps these route
/// states out of the main capture dispatcher, which also keeps that file
/// below the repository's source-size limit.
pub(super) fn begin(game: &mut Game, scene: &str) -> bool {
    match scene {
        "endless" => {
            game.transition(StateTransition::StartWarren);
            let mut focus_outpost = None;
            if let GameState::Warren(session) = &mut game.state {
                // Stage the first useful post-awakening route: the worm is
                // visible, transit is unlocked, and the selected outpost has
                // cargo and crew ready for a visible return trip.
                session.tutorial_dismissed = true;
                session.economy.food = 240.0;
                session.economy.ore_stock = 12;
                session.economy.ingots_stock = 4;
                session.economy.ore_delivered_total = 100;
                session.economy.ingots_forged = 65;
                session.won = true;
                session.victory_shown = true;
                session.factory_complete = true;
                session.factory_shown = true;
                session.worm_fed = game.data.balance.worm_awaken_at;
                session.worm_ingots_fed = game.data.balance.worm_awaken_ingots;
                session.worm_awake = true;
                session.worm_shown = true;
                session.worm_awakened_at_tick = Some(session.tick.saturating_sub(40));
                for unlock in [
                    "worm_shrine",
                    "hobgoblin",
                    "overseer",
                    "bat_courier",
                    "engineer",
                    "worm_transit",
                ] {
                    session.unlocked.insert(unlock.to_owned());
                }

                let spawn = session.spawn_tile();
                let spots: Vec<TilePos> = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| {
                        session.can_place_building(*pos) && pos.manhattan_distance(&spawn) >= 3
                    })
                    .map(|(pos, _)| pos)
                    .collect();
                let shrine = spots
                    .iter()
                    .copied()
                    .min_by_key(|pos| pos.manhattan_distance(&spawn));
                if let Some(shrine) = shrine {
                    session.buildings.push(Building::new("worm_shrine", shrine));
                }
                // Keep the route endpoints separated in the canonical scene
                // so the world-space network feedback can be inspected rather
                // than hidden beneath the two oversized building sprites.
                let outpost = spots
                    .iter()
                    .copied()
                    .find(|pos| {
                        Some(*pos) != shrine
                            && shrine.is_some_and(|shrine| {
                                let distance = pos.manhattan_distance(&shrine);
                                (6..=10).contains(&distance) && pos.manhattan_distance(&spawn) <= 10
                            })
                    })
                    .or_else(|| spots.iter().copied().find(|pos| Some(*pos) != shrine));
                if let Some(outpost) = outpost {
                    session.buildings.push(Building::new("outpost", outpost));
                    session.ensure_outpost(outpost);
                    let crew: Vec<u32> = session.creatures.iter().take(2).map(|c| c.id).collect();
                    if let Some(route) = session.outposts.last_mut() {
                        route.active = true;
                        route.cargo.insert(Good::Ore, 8);
                        route.cargo.insert(Good::Ingot, 4);
                        route.crew = crew.clone();
                    }
                    for creature in &mut session.creatures {
                        if crew.contains(&creature.id) {
                            creature.remote_outpost = Some(outpost);
                            creature.x = outpost.x as f32 + 0.5;
                            creature.y = outpost.y as f32 + 0.5;
                            creature.clear_task();
                        }
                    }
                    game.selected_building = Some(outpost);
                    focus_outpost = Some(outpost);
                }
            }
            if let Some(outpost) = focus_outpost {
                game.focus_camera_on_tile(outpost);
            }
            true
        }
        "endless_load_preview" => {
            super::begin(game, "endless");
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ore_stock = 6;
                session.economy.ingots_stock = 5;
                session.economy.food = game.data.balance.worm_feed_reserve + 4.0;
                if let Some(route) = session.outposts.last_mut() {
                    route.cargo.clear();
                    route.cargo.insert(Good::Ore, 2);
                    route.cargo.insert(Good::CookedFood, 4);
                    route.cargo_priority = CargoPriority::Ingots;
                    route.expedition_progress = 12.0;
                }
            }
            true
        }
        "endless_routes"
        | "endless_auto_return"
        | "endless_auto_resupply"
        | "endless_upgrade"
        | "endless_upgraded"
        | "endless_crew_upgrade"
        | "endless_crew_upgraded"
        | "endless_survey_upgrade"
        | "endless_survey_upgraded" => {
            super::endless::begin(game, scene);
            true
        }
        "endless_expedition_paused" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.expedition_paused = true;
                }
            }
            true
        }
        "endless_expedition_report" => {
            super::begin(game, "endless_load_preview");
            let report = if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.expedition_progress =
                        game.data.balance.outpost_expedition_cycle_sec - simulation::SIM_DT;
                }
                Some(simulation::tick(session, &game.data))
            } else {
                None
            };
            if let Some(report) = report {
                for completion in report.expedition_completed {
                    game.notifications
                        .info(format_expedition_completion(completion));
                }
            }
            true
        }
        "endless_in_flight" => {
            super::begin(game, "endless");
            if let GameState::Warren(session) = &mut game.state {
                let outpost = session.outposts.first().map(|route| route.pos);
                if let Some(outpost) = outpost {
                    let _ = simulation::outposts::start_to_shrine(session, &game.data, outpost);
                }
            }
            true
        }
        "endless_arrived" => {
            super::begin(game, "endless");
            if let GameState::Warren(session) = &mut game.state {
                let outpost = session.outposts.first().map(|route| route.pos);
                if let Some(outpost) = outpost {
                    if simulation::outposts::start_to_outpost(session, &game.data, outpost) {
                        if let Some(transit) = session.worm_transit.as_mut() {
                            transit.remaining = simulation::SIM_DT;
                        }
                    }
                }
            }
            true
        }
        _ => false,
    }
}
