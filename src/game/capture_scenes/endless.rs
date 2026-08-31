//! Capture-only scenes for post-campaign Outpost upgrades and route feedback.

use super::super::Game;
use crate::simulation;
use crate::state::creatures::{Good, Job};
use crate::state::structures::Building;
use crate::state::GameState;

pub(super) fn begin(game: &mut Game, scene: &str) {
    match scene {
        "endless_auto_return" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.auto_return_cargo = true;
                }
            }
        }
        "endless_auto_resupply" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.auto_resupply_food = true;
                    route.cargo.remove(&Good::CookedFood);
                }
            }
        }
        "endless_upgrade" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_upgrade_ingots;
            }
        }
        "endless_upgraded" => {
            begin(game, "endless_upgrade");
            let upgraded = if let GameState::Warren(session) = &mut game.state {
                session
                    .outposts
                    .last()
                    .map(|route| route.pos)
                    .is_some_and(|pos| {
                        simulation::outposts::upgrade_outpost(session, &game.data, pos)
                    })
            } else {
                false
            };
            if upgraded {
                game.notifications.success(format!(
                    "Outpost hold expanded to {} slots.",
                    game.data.balance.outpost_upgraded_storage_cap
                ));
            }
        }
        "endless_crew_upgrade" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_crew_upgrade_ingots;
            }
        }
        "endless_crew_upgraded" => {
            begin(game, "endless_crew_upgrade");
            let upgraded = if let GameState::Warren(session) = &mut game.state {
                session
                    .outposts
                    .last()
                    .map(|route| route.pos)
                    .is_some_and(|pos| {
                        simulation::outposts::upgrade_outpost_crew(session, &game.data, pos)
                    })
            } else {
                false
            };
            if upgraded {
                game.notifications.success(format!(
                    "Outpost camp expanded to {} crew.",
                    game.data.balance.outpost_upgraded_capacity
                ));
            }
        }
        "endless_survey_upgrade" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_upgrade_ingots
                    + game.data.balance.outpost_crew_upgrade_ingots
                    + game.data.balance.outpost_survey_upgrade_ingots;
                if let Some(pos) = session.outposts.last().map(|route| route.pos) {
                    let _ = simulation::outposts::upgrade_outpost(session, &game.data, pos);
                    let _ = simulation::outposts::upgrade_outpost_crew(session, &game.data, pos);
                    session.economy.ingots_stock = game.data.balance.outpost_survey_upgrade_ingots;
                }
            }
        }
        "endless_survey_upgraded" => {
            begin(game, "endless_survey_upgrade");
            let upgraded = if let GameState::Warren(session) = &mut game.state {
                session
                    .outposts
                    .last()
                    .map(|route| route.pos)
                    .is_some_and(|pos| {
                        simulation::outposts::upgrade_outpost_survey(session, &game.data, pos)
                    })
            } else {
                false
            };
            if upgraded {
                game.notifications.success(format!(
                    "Survey rig online · {}/scout.",
                    game.data.balance.outpost_upgraded_ore_per_crew
                ));
            }
        }
        "endless_resonator_upgrade" => {
            super::begin(game, "endless_survey_upgrade");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_survey_upgrade_ingots
                    + game.data.balance.outpost_resonator_upgrade_ingots;
                if let Some(pos) = session.outposts.last().map(|route| route.pos) {
                    let _ = simulation::outposts::upgrade_outpost_survey(session, &game.data, pos);
                }
            }
        }
        "endless_resonator_upgraded" => {
            begin(game, "endless_resonator_upgrade");
            let upgraded = if let GameState::Warren(session) = &mut game.state {
                session
                    .outposts
                    .last()
                    .map(|route| route.pos)
                    .is_some_and(|pos| {
                        simulation::outposts::upgrade_outpost_resonator(session, &game.data, pos)
                    })
            } else {
                false
            };
            if upgraded {
                game.notifications.success(format!(
                    "Resonance beacon tuned · {:.0}s surveys.",
                    game.data.balance.outpost_resonator_cycle_sec
                ));
            }
        }
        "endless_charter" => {
            begin(game, "endless_resonator_upgraded");
            game.notifications.clear();
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = 0;
                if let Some(route) = session.outposts.last_mut() {
                    let goal = game.data.balance.outpost_charter_haul_goal;
                    route.expeditions_completed = goal.saturating_sub(1);
                    route.ore_scouted = route
                        .expeditions_completed
                        .saturating_mul(route.crew.len() as u32)
                        .saturating_mul(game.data.balance.outpost_upgraded_ore_per_crew);
                    route.cargo.clear();
                    route.cargo.insert(Good::CookedFood, 2);
                    route.expedition_progress = 0.0;
                    route.expedition_paused = false;
                }
                session.outpost_charter_claimed = false;
            }
        }
        "endless_charter_awarded" => {
            begin(game, "endless_charter");
            game.paused = false;
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.expedition_progress =
                        simulation::outposts::expedition_cycle_sec(route, &game.data)
                            - simulation::SIM_DT;
                }
            }
        }
        "endless_wormbone_drill" => {
            super::post_campaign::begin(game, "endless");
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.outpost_charter_claimed = true;
                session.economy.ingots_stock = 8;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|pos| (pos.manhattan_distance(&spawn), pos.x, pos.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("blacksmith", spot));
                    let _ = session.queue_equipment_order(
                        &game.data,
                        spot,
                        "wormbone_drill".to_owned(),
                        game.data.balance.order_queue_size,
                    );
                    if let Some(worker) = session
                        .creatures
                        .iter_mut()
                        .find(|creature| !creature.is_remote())
                    {
                        worker.job = Job::Smith;
                        worker.x = spot.x as f32 + 0.5;
                        worker.y = spot.y as f32 + 0.5;
                        worker.clear_task();
                    }
                    game.selected_building = Some(spot);
                    game.focus_camera_on_tile(spot);
                }
            }
        }
        "endless_routes" => {
            super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                let spawn = session.spawn_tile();
                let second_outpost = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| {
                        session.can_place_building(*pos) && pos.manhattan_distance(&spawn) >= 4
                    })
                    .map(|(pos, _)| pos)
                    .find(|pos| !session.outposts.iter().any(|route| route.pos == *pos));
                if let Some(pos) = second_outpost {
                    session.buildings.push(Building::new("outpost", pos));
                    session.ensure_outpost(pos);
                    if let Some(route) = session.outposts.iter_mut().find(|route| route.pos == pos)
                    {
                        route.active = true;
                        route.cargo.clear();
                        route.cargo.insert(Good::Ingot, 5);
                        route.expedition_paused = true;
                        route.auto_return_cargo = true;
                    }
                }
                game.routes_open = true;
            }
        }
        _ => {}
    }
}
