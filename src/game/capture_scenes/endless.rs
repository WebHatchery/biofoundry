//! Capture-only scenes for post-campaign Outpost upgrades and route feedback.

use super::super::Game;
use crate::simulation;
use crate::state::creatures::Good;
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
