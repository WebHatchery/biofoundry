//! Capture-only scenes for post-campaign Outpost upgrades and route feedback.

use super::super::{format_expedition_completion, Game};
use crate::simulation;
use crate::state::creatures::{Good, Job};
use crate::state::outposts::ExpeditionCompletion;
use crate::state::structures::Building;
use crate::state::GameState;
use macroquad_toolkit::grid::TilePos;

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
        "endless_deep_survey" => {
            begin(game, "endless_charter");
        }
        "endless_deep_survey_upgrade" => {
            begin(game, "endless_charter_awarded");
            game.notifications.clear();
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.outpost_charter_claimed = true;
                session.economy.ingots_stock = game.data.balance.outpost_deep_survey_upgrade_ingots;
                if let Some(route) = session.outposts.last_mut() {
                    route.expedition_paused = true;
                    route.expedition_progress = 0.0;
                }
            }
        }
        "endless_deep_survey_upgraded" => {
            begin(game, "endless_deep_survey_upgrade");
            let upgraded = if let GameState::Warren(session) = &mut game.state {
                session
                    .outposts
                    .last()
                    .map(|route| route.pos)
                    .is_some_and(|pos| {
                        simulation::outposts::upgrade_outpost_deep_survey(session, &game.data, pos)
                    })
            } else {
                false
            };
            if upgraded {
                game.notifications.success(format!(
                    "Deep survey calibrated · {} ore/scout.",
                    game.data.balance.outpost_deep_survey_ore_per_crew
                ));
            }
        }
        "endless_archive" => {
            begin(game, "endless_deep_survey_upgraded");
            game.notifications.clear();
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = 0;
                session.outpost_archive_claims = 0;
                if let Some(route) = session.outposts.last_mut() {
                    let archive_goal = game.data.balance.outpost_archive_haul_goal;
                    route.expeditions_completed = game
                        .data
                        .balance
                        .outpost_charter_haul_goal
                        .saturating_add(archive_goal.saturating_sub(1));
                    route.ore_scouted = route
                        .expeditions_completed
                        .saturating_mul(route.crew.len() as u32)
                        .saturating_mul(simulation::outposts::ore_per_crew(route, &game.data));
                    route.cargo.clear();
                    route.cargo.insert(Good::CookedFood, 2);
                    route.expedition_progress = 0.0;
                    route.expedition_paused = false;
                }
            }
        }
        "endless_archive_awarded" => {
            begin(game, "endless_archive");
            game.paused = false;
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.expedition_progress =
                        simulation::outposts::expedition_cycle_sec(route, &game.data)
                            - simulation::SIM_DT;
                }
            }
        }
        "endless_archive_wayfinder" => {
            begin(game, "endless_wormbone_drill");
            game.notifications.clear();
            game.paused = true;
            let selected = game.selected_building;
            if let GameState::Warren(session) = &mut game.state {
                session.outpost_archive_claims = 1;
                session.unlocked.insert("archive_wayfinder".to_owned());
                session.economy.ingots_stock = game
                    .data
                    .equipment_def("archive_wayfinder")
                    .map(|equipment| equipment.cost_ingots)
                    .unwrap_or(14);
                if let Some(pos) = selected {
                    if let Some(blacksmith) = session.building_at_mut(pos) {
                        blacksmith.orders.clear();
                        blacksmith.stocks.clear();
                    }
                }
            }
        }
        "endless_relay" => {
            begin(game, "endless_routes");
            game.notifications.clear();
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.outpost_archive_claims = 1;
                session.outpost_relay_claimed = false;
                let first_haul_goal = game.data.balance.outpost_relay_haul_goal / 2;
                if let Some(first) = session.outposts.first_mut() {
                    first.active = true;
                    first.expeditions_completed = first_haul_goal;
                }
                if let Some(second) = session.outposts.get_mut(1) {
                    second.active = true;
                    second.expeditions_completed = game
                        .data
                        .balance
                        .outpost_relay_haul_goal
                        .saturating_sub(first_haul_goal);
                }
            }
        }
        "endless_relay_awarded" => {
            begin(game, "endless_relay");
            game.routes_open = false;
            if let GameState::Warren(session) = &mut game.state {
                if simulation::outposts::claim_outpost_relay(session, &game.data) {
                    game.notifications.success(format!(
                        "Worm Road Relay · +{} ingots · twin routes linked.",
                        game.data.balance.outpost_relay_reward_ingots
                    ));
                }
            }
            game.routes_open = true;
            game.paused = true;
        }
        "endless_convoy" => {
            begin(game, "endless_routes");
            game.notifications.clear();
            game.routes_open = true;
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.outpost_charter_claimed = true;
                session.outpost_relay_claimed = true;
                session.outpost_convoy_claims = 0;
                session.economy.ingots_stock = 0;
                let route_goal = game.data.balance.outpost_convoy_route_goal as usize;
                while session.outposts.len() < route_goal {
                    let existing: Vec<TilePos> =
                        session.outposts.iter().map(|route| route.pos).collect();
                    let pos = session
                        .world
                        .tiles
                        .iter_with_pos()
                        .find(|(pos, tile)| {
                            tile.walkable()
                                && session.can_place_building(*pos)
                                && !existing.contains(pos)
                        })
                        .map(|(pos, _)| pos);
                    let Some(pos) = pos else { break };
                    session.buildings.push(Building::new("outpost", pos));
                    session.ensure_outpost(pos);
                }
                let total_hauls = game
                    .data
                    .balance
                    .outpost_relay_haul_goal
                    .saturating_add(game.data.balance.outpost_convoy_haul_goal)
                    .saturating_sub(1);
                for (index, route) in session.outposts.iter_mut().enumerate() {
                    route.active = index < route_goal;
                    route.expedition_paused = true;
                    route.expeditions_completed = if index == 0 {
                        total_hauls / route_goal.max(1) as u32
                    } else {
                        0
                    };
                }
                let assigned = session
                    .outposts
                    .iter()
                    .map(|route| route.expeditions_completed)
                    .sum::<u32>();
                if let Some(route) = session.outposts.first_mut() {
                    route.expeditions_completed = route
                        .expeditions_completed
                        .saturating_add(total_hauls.saturating_sub(assigned));
                }
            }
        }
        "endless_convoy_awarded" => {
            begin(game, "endless_convoy");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.first_mut() {
                    route.expeditions_completed = route.expeditions_completed.saturating_add(1);
                }
                if simulation::outposts::claim_outpost_convoy(session, &game.data) > 0 {
                    game.notifications.success(format!(
                        "Worm Road Convoy · +{} ingots · contract cleared.",
                        game.data.balance.outpost_convoy_reward_ingots
                    ));
                }
            }
            game.routes_open = true;
            game.paused = true;
        }
        "endless_waypoint" => {
            begin(game, "endless_convoy_awarded");
            game.notifications.clear();
            game.routes_open = false;
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_waypoint_upgrade_ingots;
                session.outpost_convoy_claims = session.outpost_convoy_claims.max(1);
                session.outposts.truncate(1);
                let focus = session.outposts.first().map(|route| route.pos);
                for route in &mut session.outposts {
                    route.active = Some(route.pos) == focus;
                    route.expedition_paused = false;
                    route.cargo.clear();
                    route.crew.clear();
                }
                if let Some(route) = session.outposts.first_mut() {
                    route.active = true;
                    route.storage_upgraded = true;
                    route.crew_upgraded = true;
                    route.survey_upgraded = true;
                    route.resonator_upgraded = true;
                    route.deep_survey_upgraded = true;
                    route.signal_cache_upgraded = true;
                    route.waypoint_upgraded = false;
                    route.expedition_paused = false;
                    game.selected_building = Some(route.pos);
                }
            }
        }
        "endless_waypoint_awarded" => {
            begin(game, "endless_waypoint");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(pos) = session.outposts.first().map(|route| route.pos) {
                    if simulation::outposts::upgrade_outpost_waypoint(session, &game.data, pos) {
                        game.notifications.success(format!(
                            "Worm Road Waypoint online · {:.0}s transit.",
                            game.data.balance.outpost_waypoint_transit_time_sec
                        ));
                    }
                }
            }
            game.routes_open = false;
            game.paused = true;
        }
        "endless_waypoint_in_flight" => {
            begin(game, "endless_waypoint_awarded");
            game.notifications.clear();
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.first_mut() {
                    route.cargo.insert(Good::Ore, 2);
                    let pos = route.pos;
                    let _ = simulation::outposts::start_to_shrine(session, &game.data, pos);
                }
            }
        }
        "endless_signal_cache" => {
            begin(game, "endless_relay");
            game.notifications.clear();
            game.routes_open = false;
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.outpost_charter_claimed = true;
                session.outpost_relay_claimed = true;
                session.economy.ingots_stock =
                    game.data.balance.outpost_signal_cache_upgrade_ingots;
                if let Some(route) = session.outposts.first_mut() {
                    route.active = true;
                    route.storage_upgraded = true;
                    route.crew_upgraded = true;
                    route.survey_upgraded = true;
                    route.resonator_upgraded = true;
                    route.deep_survey_upgraded = true;
                    route.signal_cache_upgraded = false;
                }
            }
        }
        "endless_signal_cache_awarded" => {
            begin(game, "endless_signal_cache");
            let selected = game.selected_building;
            if let Some(pos) = selected {
                if let GameState::Warren(session) = &mut game.state {
                    if simulation::outposts::upgrade_outpost_signal_cache(session, &game.data, pos)
                    {
                        game.notifications.success(format!(
                            "Signal cache online · +{} ingot per haul.",
                            game.data.balance.outpost_signal_cache_ingots_per_haul
                        ));
                    }
                }
            }
            game.paused = true;
        }
        "endless_signal_cache_haul" => {
            begin(game, "endless_signal_cache");
            game.notifications.clear();
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.first_mut() {
                    let ore = 2 * game.data.balance.outpost_deep_survey_ore_per_crew;
                    let ingots = game.data.balance.outpost_signal_cache_ingots_per_haul;
                    route.cargo.clear();
                    route.cargo.insert(Good::Ore, ore);
                    route.cargo.insert(Good::Ingot, ingots);
                    route.signal_cache_upgraded = true;
                    route.expeditions_completed = 4;
                    route.ore_scouted = ore * route.expeditions_completed;
                    route.signal_cache_ingots = ingots;
                    game.notifications.success(format_expedition_completion(
                        ExpeditionCompletion {
                            outpost: route.pos,
                            ore,
                            ingots,
                            food_spent: 2,
                        },
                    ));
                }
            }
            game.paused = true;
        }
        "endless_wormbone_drill" => {
            super::post_campaign::begin(game, "endless");
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.outpost_charter_claimed = true;
                session.economy.ingots_stock = 36;
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
                    let _ = session.queue_equipment_order(
                        &game.data,
                        spot,
                        "wormbone_hauling_frame".to_owned(),
                        game.data.balance.order_queue_size,
                    );
                    let _ = session.queue_equipment_order(
                        &game.data,
                        spot,
                        "wormbone_smiths_hammer".to_owned(),
                        game.data.balance.order_queue_size,
                    );
                    let _ = session.queue_equipment_order(
                        &game.data,
                        spot,
                        "wormbone_guard_blade".to_owned(),
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
                        route.signal_cache_upgraded = true;
                        route.signal_cache_ingots = 3;
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
