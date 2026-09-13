//! Early endless-mode scene staging for route setup and first upgrades.

use super::*;

pub(super) fn handles(scene: &str) -> bool {
    matches!(
        scene,
        "endless_route_build"
            | "endless_wormsong_route"
            | "endless_wormsong_concord"
            | "endless_wormsong_circuit"
            | "endless_wormsong_circuit_awarded"
            | "endless_wormsong_encore"
            | "endless_wormsong_encore_awarded"
            | "endless_wormsong_chorus"
            | "endless_wormsong_chorus_awarded"
            | "endless_wormsong_chorus_haul"
            | "endless_wormsong_encore_inspect"
            | "endless_auto_return"
            | "endless_auto_resupply"
            | "endless_auto_load"
            | "endless_auto_load_started"
            | "endless_upgrade"
            | "endless_upgraded"
            | "endless_rest_hollow"
            | "endless_crew_upgrade"
            | "endless_crew_upgraded"
            | "endless_survey_upgrade"
            | "endless_survey_upgraded"
            | "endless_resonator_upgrade"
            | "endless_resonator_upgraded"
            | "endless_charter"
    )
}

pub(super) fn begin(game: &mut Game, scene: &str) {
    match scene {
        "endless_route_build" => {
            super::super::begin(game, "endless");
            game.notifications.clear();
            game.routes_open = false;
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.outposts.clear();
                session
                    .buildings
                    .retain(|building| building.kind != "outpost");
                for creature in &mut session.creatures {
                    creature.remote_outpost = None;
                }
                session.unlocked.insert("worm_transit".to_owned());
                session.economy.ore_stock = 30;
            }
            game.selected_building = None;
        }
        "endless_wormsong_route" => remote_specialists::begin(game),
        "endless_wormsong_concord" => wormsong_concord::begin(game),
        "endless_wormsong_circuit" => wormsong_circuit::begin(game),
        "endless_wormsong_circuit_awarded" => {
            wormsong_circuit::begin(game);
            if let GameState::Warren(session) = &mut game.state {
                if simulation::outposts::claim_outpost_circuit(session, &game.data) {
                    game.notifications.success(format!(
                        "Wormsong Circuit · +{} ingots · two complete crews linked.",
                        game.data.balance.outpost_circuit_reward_ingots
                    ));
                }
            }
            game.routes_open = true;
            game.paused = true;
        }
        "endless_wormsong_encore" => wormsong_encore::begin(game),
        "endless_wormsong_encore_awarded" => wormsong_encore::award(game),
        "endless_wormsong_chorus" => wormsong_chorus::begin(game),
        "endless_wormsong_chorus_awarded" => wormsong_chorus::award(game),
        "endless_wormsong_chorus_haul" => wormsong_chorus::haul(game),
        "endless_wormsong_encore_inspect" => {
            wormsong_encore::begin(game);
            game.routes_open = false;
        }
        "endless_auto_return" => {
            super::super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.auto_return_cargo = true;
                }
            }
        }
        "endless_auto_resupply" => {
            super::super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.auto_resupply_food = true;
                    route.cargo.remove(&Good::CookedFood);
                }
            }
        }
        "endless_auto_load" => {
            super::super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.auto_load = true;
                    route.cargo.clear();
                }
            }
        }
        "endless_auto_load_started" => {
            super::begin(game, "endless_auto_load");
            if let GameState::Warren(session) = &mut game.state {
                if simulation::outposts::start_auto_load_if_ready(session, &game.data).is_some() {
                    game.notifications.info(
                        "Auto-load departed — cargo and available scouts are on the worm road.",
                    );
                }
            }
        }
        "endless_upgrade" => {
            super::super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_upgrade_ingots;
            }
        }
        "endless_upgraded" => {
            super::begin(game, "endless_upgrade");
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
        "endless_rest_hollow" => {
            super::super::begin(game, "endless");
            game.notifications.clear();
            game.paused = true;
            let mut focus_pos = None;
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ore_stock = game
                    .data
                    .buildings
                    .get("rest_hollow")
                    .expect("Rest Hollow capture data")
                    .cost_ore;
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .max_by_key(|pos| {
                        (pos.manhattan_distance(&session.spawn_tile()), pos.x, pos.y)
                    });
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("rest_hollow", spot));
                    game.selected_building = Some(spot);
                    focus_pos = Some(spot);
                }
            }
            if let Some(spot) = focus_pos {
                game.focus_camera_on_tile(spot);
            }
        }
        "endless_crew_upgrade" => {
            super::super::begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_crew_upgrade_ingots;
            }
        }
        "endless_crew_upgraded" => {
            super::begin(game, "endless_crew_upgrade");
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
            super::super::begin(game, "endless_load_preview");
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
            super::begin(game, "endless_survey_upgrade");
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
            super::super::begin(game, "endless_survey_upgrade");
            if let GameState::Warren(session) = &mut game.state {
                session.economy.ingots_stock = game.data.balance.outpost_survey_upgrade_ingots
                    + game.data.balance.outpost_resonator_upgrade_ingots;
                if let Some(pos) = session.outposts.last().map(|route| route.pos) {
                    let _ = simulation::outposts::upgrade_outpost_survey(session, &game.data, pos);
                }
            }
        }
        "endless_resonator_upgraded" => {
            super::begin(game, "endless_resonator_upgrade");
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
            super::begin(game, "endless_resonator_upgraded");
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
        _ => unreachable!("unhandled early endless scene: {scene}"),
    }
}
