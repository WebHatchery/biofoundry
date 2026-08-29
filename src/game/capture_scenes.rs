//! Deterministic scene setup used by the screenshot verification harness.

use super::Game;
use crate::simulation;
use crate::state::creatures::{Good, Job};
use crate::state::structures::Building;
use crate::state::world::Tile;
use crate::state::{GameState, StateTransition};
use macroquad_toolkit::grid::TilePos;

/// Seed a named scene for the headless screenshot harness.
pub(super) fn begin(game: &mut Game, scene: &str) {
    match scene {
        "menu" => game.transition(StateTransition::BackToMenu),
        "new_warren_confirm" => {
            game.transition(StateTransition::BackToMenu);
            game.save_exists = true;
            game.confirm_new_warren = true;
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
        "victory" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = game.data.balance.win_food_surplus;
                session.economy.ore_delivered_total = game.data.balance.win_ore_delivered;
                session.won = true;
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
                // The mine → blacksmith → ingot chain mid-flow: place a
                // blacksmith by the warren, staff a smith, keep everyone
                // fed, and let the ore route light up.
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
                    session.buildings.push(Building::new("blacksmith", spot));
                }
                // Free two miners to haul, and put one on the anvil.
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
        "optional" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Show post-campaign support choices without a modal so their
                // benefits remain reviewable in the canonical HUD capture.
                session.tutorial_dismissed = true;
                session.economy.food = 300.0;
                session.economy.ore_stock = 50;
                session.won = true;
                session.victory_shown = true;
                session.factory_complete = true;
                session.factory_shown = true;
                session.unlocked.insert("slime_janitor".to_owned());
                session.unlocked.insert("bat_courier".to_owned());
            }
        }
        "endless" => {
            game.transition(StateTransition::StartWarren);
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
                    .take(2)
                    .collect();
                if let Some(shrine) = spots.first().copied() {
                    session.buildings.push(Building::new("worm_shrine", shrine));
                }
                if let Some(outpost) = spots.get(1).copied() {
                    session.buildings.push(Building::new("outpost", outpost));
                    session.ensure_outpost(outpost);
                    let crew = session.creatures.iter().take(2).map(|c| c.id).collect();
                    if let Some(route) = session.outposts.last_mut() {
                        route.active = true;
                        route.cargo.insert(Good::Ore, 8);
                        route.cargo.insert(Good::Ingot, 4);
                        route.crew = crew;
                    }
                    game.selected_building = Some(outpost);
                }
            }
        }
        "endless_failure" => {
            begin(game, "endless");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.active = false;
                    route.last_failure =
                        Some("The worm route collapsed; cargo returned to safety.".to_owned());
                }
                session.last_transit_failure =
                    Some("Transit failed because the outpost was inactive.".to_owned());
            }
        }
        "endless_in_flight" => {
            begin(game, "endless");
            if let GameState::Warren(session) = &mut game.state {
                let outpost = session.outposts.first().map(|route| route.pos);
                if let Some(outpost) = outpost {
                    let _ = simulation::outposts::start_to_shrine(session, &game.data, outpost);
                }
            }
        }
        "endless_arrived" => {
            begin(game, "endless");
            if let GameState::Warren(session) = &mut game.state {
                let outpost = session.outposts.first().map(|route| route.pos);
                if let Some(outpost) = outpost {
                    let _ = simulation::outposts::start_to_outpost(session, &game.data, outpost);
                }
                let _ = simulation::outposts::tick_transit(
                    session,
                    &game.data,
                    game.data.balance.worm_transit_time_sec + 0.1,
                );
            }
        }
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
        "help" => {
            game.transition(StateTransition::StartWarren);
            game.help_open = true;
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
            }
        }
        "pause" => {
            game.transition(StateTransition::StartWarren);
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.economy.food = 72.0;
            }
        }
        "collapse" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                session.tutorial_dismissed = true;
                session.creatures.clear();
            }
        }
        // "warren" and the harness default "gameplay" boot straight
        // into a fresh session on the config seed.
        _ => game.transition(StateTransition::StartWarren),
    }
}
