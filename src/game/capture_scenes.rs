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
        "breeding" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Stage the capture → study → adapt chain mid-flow.
                session.tutorial_dismissed = true;
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
            }
        }
        "worm" => {
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
                session.worm_shown = true;
                for _ in 0..300 {
                    simulation::tick(session, &game.data);
                }
            }
        }
        // "warren" and the harness default "gameplay" boot straight
        // into a fresh session on the config seed.
        _ => game.transition(StateTransition::StartWarren),
    }
}
