//! Deterministic scene setup used by the screenshot verification harness.

use super::Game;
use crate::simulation;
use crate::state::creatures::{Good, Job};
use crate::state::structures::{BuildSite, Building};
use crate::state::world::Tile;
use crate::state::{GameState, StateTransition};
use macroquad_toolkit::grid::TilePos;

mod endless;
mod optional;
mod overlays;
mod post_campaign;
mod recovery;

/// Seed a named scene for the headless screenshot harness.
pub(super) fn begin(game: &mut Game, scene: &str) {
    if post_campaign::begin(game, scene) {
        return;
    }
    match scene {
        "touch_audit" => {
            begin(game, "warren");
            game.arm_touch_target_audit();
        }
        scene if scene.starts_with("touch_audit_") => {
            let base_scene = &scene["touch_audit_".len()..];
            begin(game, base_scene);
            game.arm_touch_target_audit();
        }
        "menu" => game.transition(StateTransition::BackToMenu),
        "new_warren_confirm" => {
            game.transition(StateTransition::BackToMenu);
            game.save_exists = true;
            game.confirm_new_warren = true;
        }
        "load_confirm" => {
            game.transition(StateTransition::StartWarren);
            game.paused = true;
            game.confirm_load = true;
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = 80.0;
            }
        }
        "load_confirm_resolved" => {
            game.transition(StateTransition::StartWarren);
            game.confirm_load = true;
            let loaded = match &game.state {
                GameState::Warren(session) => session.as_ref().clone(),
                GameState::Menu => return,
            };
            game.install_loaded_session(loaded);
            game.notifications.success("Warren loaded.");
        }
        "settings" => {
            game.transition(StateTransition::BackToMenu);
            game.settings_open = true;
        }
        "factory" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Stage a mid-build factory: banked ore, ghosts, digs.
                session.tutorial_dismissed = true;
                session.economy.ore_stock = 24;
                session.economy.food = 60.0;
                let spawn = session.spawn_tile();
                let mut spots: Vec<TilePos> = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .collect();
                spots.sort_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                for (kind, spot) in ["farm", "cook_pot"].iter().zip(spots.iter().skip(2)) {
                    simulation::try_place_build_site(session, &game.data, kind, *spot);
                }
                for mark in session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(_, t)| **t == Tile::Rock)
                    .map(|(pos, _)| pos)
                    .filter(|p| p.manhattan_distance(&spawn) <= 6)
                    .take(4)
                    .collect::<Vec<_>>()
                {
                    session.toggle_dig_mark(mark);
                }
                for _ in 0..900 {
                    simulation::tick(session, &game.data);
                }
            }
        }
        "tutorial_food" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_step = 1;
                session.economy.food = 36.0;
                // Show the lesson at the exact recovery handoff: a player-
                // placed Farm is waiting on ore while Food Grid is under
                // pressure, so the Objective can name both visible responses.
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .find(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos);
                if let Some(spot) = spot {
                    session.build_sites.push(BuildSite {
                        kind: "farm".to_owned(),
                        pos: spot,
                        ore_needed: 10,
                        ore_delivered: 0,
                    });
                }
            }
        }
        "tutorial_factory" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Hold on the factory lesson so its distinction between the
                // prebuilt Mine and the Mine build button is reviewable.
                session.tutorial_step = 2;
                session.tutorial_built = true;
                session.economy.food = 80.0;
                session.economy.ore_stock = 24;
            }
        }
        "tutorial_worm" => {
            begin(game, "shrine");
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = false;
                session.tutorial_step = 4;
            }
        }
        "victory" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = game.data.balance.win_food_surplus;
                session.economy.ore_delivered_total = game.data.balance.win_ore_delivered;
                session.won = true;
            }
        }
        "security_stuck" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.won = true;
                session.victory_shown = true;
                session.creatures.clear();
                session.spawn_creature(&game.data, "overseer", Job::Idle);
            }
        }
        "factory_complete" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = 160.0;
                session.economy.ore_delivered_total = game.data.balance.win_ore_delivered;
                session.economy.ingots_forged = game.data.balance.win2_ingots;
                session.won = true;
                session.victory_shown = true;
                session.factory_complete = true;
                session.creatures[0].job = Job::Guard;
                session.unlocked.insert("worm_shrine".to_owned());
            }
        }
        "mine" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // The prebuilt mine mid-extraction, inspection open.
                session.tutorial_dismissed = true;
                session.economy.food = 80.0;
                for _ in 0..400 {
                    simulation::tick(session, &game.data);
                }
                game.selected_building = session.buildings_of("mine").next().map(|b| b.pos);
            }
        }
        "blacksmith" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // A smith with a queued order but no ore: the inspection
                // card should expose the missing input instead of calling
                // an unpaid order nominally "working".
                session.tutorial_dismissed = true;
                session.economy.food = 200.0;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    let mut shop = Building::new("blacksmith", spot);
                    shop.orders.push("iron_pickaxe".to_owned());
                    session.buildings.push(shop);
                }
                // Free two miners to haul, and put one on the anvil so the
                // blocked order is visibly staffed rather than abandoned.
                let species = &game.data.species;
                session.reassign(Job::Miner, Job::Carrier, |s| {
                    species.get(s).map(|d| d.reassignable).unwrap_or(false)
                });
                session.reassign(Job::Miner, Job::Smith, |s| {
                    species.get(s).map(|d| d.reassignable).unwrap_or(false)
                });
                for _ in 0..500 {
                    simulation::tick(session, &game.data);
                }
                game.selected_building = session.buildings_of("blacksmith").next().map(|b| b.pos);
            }
        }
        "smelter" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Keep the living furnace staffed while withholding both
                // batch inputs, so the inspection card demonstrates the
                // actionable starvation wording at the published scale.
                session.tutorial_dismissed = true;
                session.economy.food = 80.0;
                session.economy.ore_stock = 0;
                session.creatures.clear();
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("smelter", spot));
                    session.spawn_creature(&game.data, "salamander", Job::Smelter);
                    game.selected_building = Some(spot);
                }
            }
        }
        "cook_pot" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Keep a cook stationed at an empty pot so the critical-path
                // recovery line names the mushrooms needed for one batch.
                session.tutorial_dismissed = true;
                session.economy.food = 80.0;
                session.creatures.clear();
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("cook_pot", spot));
                    session.spawn_creature(&game.data, "goblin", Job::Cook);
                    if let Some(cook) = session.creatures.last_mut() {
                        cook.x = spot.x as f32 + 0.5;
                        cook.y = spot.y as f32 + 0.5;
                    }
                    game.selected_building = Some(spot);
                }
            }
        }
        "kiln" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // The kiln is autonomous, so an empty wood buffer is enough
                // to expose its next required input in the inspection card.
                session.tutorial_dismissed = true;
                session.economy.food = 80.0;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("kiln", spot));
                    game.selected_building = Some(spot);
                }
            }
        }
        "waste" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Keep the early waste state visible before specialist
                // controls unlock, so the inspection card explains the next
                // campaign gate instead of offering an unavailable action.
                session.tutorial_dismissed = true;
                session.economy.food = 80.0;
                let farm = session.buildings_of("farm").next().map(|b| b.pos);
                if let Some(farm) = farm {
                    if let Some(building) = session.building_at_mut(farm) {
                        building.waste = 2.5;
                    }
                    game.selected_building = Some(farm);
                }
            }
        }
        "equipment" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // The feedback loop mid-flow: an equipped miner at the
                // prebuilt mine, a blacksmith crafting a queued pickaxe,
                // inspection open on the mine to show the boosted rate.
                session.tutorial_dismissed = true;
                session.economy.food = 300.0;
                session.economy.ingots_stock = 4;
                // Equip the working miner with an Iron Pickaxe outright.
                if let Some(m) = session.creatures.iter_mut().find(|c| c.job == Job::Miner) {
                    m.equipment = Some("iron_pickaxe".to_owned());
                }
                // A blacksmith with ingots and a queued craft.
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    let mut shop = Building::new("blacksmith", spot);
                    shop.add_stock(Good::Ingot, 3.0);
                    shop.orders.push("hauling_frame".to_owned());
                    session.buildings.push(shop);
                }
                let species = &game.data.species;
                session.reassign(Job::Miner, Job::Smith, |s| {
                    species.get(s).map(|d| d.reassignable).unwrap_or(false)
                });
                for _ in 0..200 {
                    simulation::tick(session, &game.data);
                }
                game.selected_building = session.buildings_of("mine").next().map(|b| b.pos);
            }
        }
        "blacksmith_queue_full" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = 300.0;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    let mut shop = Building::new("blacksmith", spot);
                    for index in 0..game.data.balance.order_queue_size {
                        let item = &game.data.equipment[index % game.data.equipment.len()];
                        shop.orders.push(item.id.clone());
                    }
                    session.buildings.push(shop);
                    game.selected_building = Some(spot);
                }
            }
        }
        "overseer" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // The evolution line: a lean elite crew — one Hobgoblin
                // miner in an Overseer's aura out-produces a mid-game
                // crowd. Count the legs on screen.
                session.tutorial_dismissed = true;
                session.economy.food = 400.0;
                session.unlocked.insert("hobgoblin".to_owned());
                session.unlocked.insert("overseer".to_owned());
                session.creatures.clear();
                session.spawn_creature(&game.data, "goblin", Job::Carrier);
                session.spawn_creature(&game.data, "hobgoblin", Job::Miner);
                session.spawn_creature(&game.data, "overseer", Job::Idle);
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("breeding_pit", spot));
                }
                for _ in 0..400 {
                    simulation::tick(session, &game.data);
                }
                game.selected_building = session.buildings_of("mine").next().map(|b| b.pos);
            }
        }
        "famine" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                for _ in 0..600 {
                    simulation::tick(session, &game.data);
                }
                session.economy.food = 0.0;
                for creature in &mut session.creatures {
                    creature.satiation = 0.3;
                }
                for _ in 0..100 {
                    simulation::tick(session, &game.data);
                }
            }
        }
        "food_warning" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_step = 1;
                session.economy.food = 10.0;
                session.economy.production_ema_per_min = 0.0;
                session.raid_in = game.data.balance.raid_first_sec;
            }
        }
        "raid_food_warning" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Keep both warnings visible so the shared top-bar response
                // remains a canonical, deterministic regression scene.
                session.tutorial_step = 3;
                session.economy.food = 10.0;
                session.economy.production_ema_per_min = 0.0;
                session.raid_in = 150.0;
            }
        }
        "raid" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Stage an active raid with guards responding.
                session.tutorial_dismissed = true;
                session.economy.food = 60.0;
                let species = &game.data.species;
                for _ in 0..2 {
                    session.reassign(Job::Miner, Job::Guard, |s| {
                        species.get(s).map(|d| d.reassignable).unwrap_or(false)
                    });
                }
                for _ in 0..300 {
                    simulation::tick(session, &game.data);
                }
                session.raid_in = 0.0;
                for _ in 0..80 {
                    simulation::tick(session, &game.data);
                }
            }
        }
        "raid_warning" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_step = 3;
                session.economy.food = 80.0;
                session.economy.ore_delivered_total = 35;
                session.raid_in = 150.0;
            }
        }
        "breeding" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Stage the capture → study → adapt chain mid-flow.
                session.tutorial_dismissed = true;
                session.economy.food = 260.0;
                session.economy.ingots_stock = 20;
                session.won = true;
                session.victory_shown = true;
                session.creatures[0].job = Job::Guard;
                for unlock in ["breeding_pit", "hobgoblin", "overseer", "engineer"] {
                    session.unlocked.insert(unlock.to_owned());
                }
                let spawn = session.spawn_tile();
                let mut spots: Vec<TilePos> = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .collect();
                spots.sort_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                for (kind, spot) in ["trap", "study_pen", "breeding_pit"]
                    .iter()
                    .zip(spots.iter().skip(1))
                {
                    session.buildings.push(Building::new(kind, *spot));
                }
                session.progress.beetles_captured = 2;
                session.progress.specimens = 2;
                session.wild_spawn_in = 0.0;
                for _ in 0..200 {
                    simulation::tick(session, &game.data);
                }
                game.selected_building = session.buildings_of("breeding_pit").next().map(|b| b.pos);
            }
        }
        "breeding_locked" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Keep specialist choices locked but close enough to show the
                // live ingot prerequisite on every breeding button.
                session.tutorial_dismissed = true;
                session.economy.food = 260.0;
                session.economy.ingots_forged = 7;
                session.won = true;
                session.victory_shown = true;
                session.creatures[0].job = Job::Guard;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("breeding_pit", spot));
                    game.selected_building = Some(spot);
                }
            }
        }
        "optional" => optional::begin(game),
        "shrine" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Stage the final production demand before awakening, with
                // the inspection card open so its estimate is reviewable.
                session.tutorial_dismissed = true;
                session.economy.food = 72.0;
                session.economy.ingots_stock = 6;
                session.won = true;
                session.victory_shown = true;
                session.factory_complete = true;
                session.factory_shown = true;
                session.creatures[0].job = Job::Guard;
                session.worm_fed = 44.0;
                session.worm_ingots_fed = 4;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| {
                        session.can_place_building(*pos) && pos.manhattan_distance(&spawn) >= 3
                    })
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("worm_shrine", spot));
                    game.selected_building = Some(spot);
                }
            }
        }
        "shrine_waiting" => {
            begin(game, "shrine");
            if let GameState::Warren(session) = &mut game.state {
                // Keep the final-demand Shrine below its protected food
                // reserve so the map badge and inspection wording can be
                // reviewed together in the published capture set.
                session.economy.food = game.data.balance.worm_feed_reserve;
            }
        }
        "unreachable_workstation" => recovery::unreachable_workstation(game),
        "worm" | "completion" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Stage the awakened monument.
                session.tutorial_dismissed = true;
                let spawn = session.spawn_tile();
                let spot = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| {
                        session.can_place_building(*pos) && pos.manhattan_distance(&spawn) >= 3
                    })
                    .map(|(pos, _)| pos)
                    .min_by_key(|p| (p.manhattan_distance(&spawn), p.x, p.y));
                if let Some(spot) = spot {
                    session.buildings.push(Building::new("worm_shrine", spot));
                }
                session.economy.ingots_forged = game.data.balance.win2_ingots;
                session.won = true;
                session.victory_shown = true;
                session.factory_complete = true;
                session.factory_shown = true;
                session.worm_fed = game.data.balance.worm_awaken_at;
                session.worm_ingots_fed = game.data.balance.worm_awaken_ingots;
                session.worm_awake = true;
                session.worm_shown = scene != "completion";
                for _ in 0..300 {
                    simulation::tick(session, &game.data);
                }
                session.worm_awakened_at_tick = Some(session.tick.saturating_sub(12));
            }
        }
        "save_failure" => {
            begin(game, "warren");
            game.checkpoint_warning = Some(super::persistence::save_failure_banner(true));
        }
        "save_failure_first_save" => {
            begin(game, "warren");
            game.save_exists = false;
            game.checkpoint_warning = Some(super::persistence::save_failure_banner(false));
        }
        "menu_save_guard" => {
            begin(game, "warren");
            game.save_exists = false;
            game.checkpoint_warning = Some(super::persistence::save_failure_banner(false));
            game.notifications
                .danger("Menu held — tap Save before leaving.");
        }
        "save_recovery_failure" => {
            begin(game, "warren");
            game.checkpoint_warning = Some(super::persistence::save_recovery_failure_banner());
        }
        "help" => overlays::help(game),
        "event_log" => overlays::event_log(game),
        "event_log_older" => overlays::event_log_older(game),
        "pause" => overlays::pause(game),
        "collapse" => overlays::collapse(game),
        "crowding" => {
            game.transition(StateTransition::StartWarren);
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = 240.0;
                session.creatures.clear();
                let capacity = session.local_warren_capacity(&game.data);
                for _ in 0..capacity + 8 {
                    session.spawn_creature(&game.data, "goblin", Job::Idle);
                }
            }
        }
        // "warren" and the harness default "gameplay" boot straight
        // into a fresh session on the config seed.
        _ => game.transition(StateTransition::StartWarren),
    }
}
