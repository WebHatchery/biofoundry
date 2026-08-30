//! Deterministic scene setup used by the screenshot verification harness.

use super::{format_expedition_completion, Game};
use crate::simulation;
use crate::state::creatures::{Good, Job};
use crate::state::outposts::CargoPriority;
use crate::state::structures::{BuildSite, Building};
use crate::state::world::Tile;
use crate::state::{GameState, StateTransition};
use macroquad_toolkit::grid::TilePos;

mod endless;
mod optional;
mod overlays;

/// Seed a named scene for the headless screenshot harness.
pub(super) fn begin(game: &mut Game, scene: &str) {
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
                }
            }
        }
        "endless_load_preview" => {
            begin(game, "endless");
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
        }
        "endless_routes" => endless::begin(game, scene),
        "endless_auto_return"
        | "endless_auto_resupply"
        | "endless_upgrade"
        | "endless_upgraded" => endless::begin(game, scene),
        "endless_expedition_paused" => {
            begin(game, "endless_load_preview");
            if let GameState::Warren(session) = &mut game.state {
                if let Some(route) = session.outposts.last_mut() {
                    route.expedition_paused = true;
                }
            }
        }
        "endless_expedition_report" => {
            begin(game, "endless_load_preview");
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
        }
        "endless_forge" => {
            game.transition(StateTransition::StartWarren);
            if let GameState::Warren(session) = &mut game.state {
                // Show the viable forge chain before Worm Transit unlocks, so
                // the completed-campaign objective's live progress can be
                // reviewed without a route modal covering it.
                session.tutorial_dismissed = true;
                session.economy.food = 220.0;
                session.economy.ingots_forged = 37;
                session.economy.ingots_stock = 6;
                session.won = true;
                session.victory_shown = true;
                session.factory_complete = true;
                session.factory_shown = true;
                session.worm_fed = game.data.balance.worm_awaken_at;
                session.worm_ingots_fed = game.data.balance.worm_awaken_ingots;
                session.worm_awake = true;
                session.worm_shown = true;
                session.creatures[0].job = Job::Smith;
                for unlock in ["worm_shrine", "hobgoblin"] {
                    session.unlocked.insert(unlock.to_owned());
                }
                let spawn = session.spawn_tile();
                if let Some(spot) = session
                    .world
                    .tiles
                    .iter_with_pos()
                    .filter(|(pos, _)| session.can_place_building(*pos))
                    .min_by_key(|(pos, _)| (pos.manhattan_distance(&spawn), pos.x, pos.y))
                    .map(|(pos, _)| pos)
                {
                    session.buildings.push(Building::new("blacksmith", spot));
                }
                game.paused = true;
            }
        }
        "endless_empty" => {
            begin(game, "endless");
            game.paused = true;
            if let GameState::Warren(session) = &mut game.state {
                // Show the route's honest empty state: no stored goods, no
                // stockpiled crew, and no payload ready for the next run.
                session.economy.ore_stock = 0;
                session.economy.ingots_stock = 0;
                session.economy.food = game.data.balance.worm_feed_reserve;
                session.creatures.clear();
                if let Some(route) = session.outposts.last_mut() {
                    route.cargo.clear();
                    route.crew.clear();
                }
            }
        }
        "endless_failure" | "route_failure" => {
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
                    if simulation::outposts::start_to_outpost(session, &game.data, outpost) {
                        if let Some(transit) = session.worm_transit.as_mut() {
                            transit.remaining = simulation::SIM_DT;
                        }
                    }
                }
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
        "help" => overlays::help(game),
        "event_log" => overlays::event_log(game),
        "event_log_older" => overlays::event_log_older(game),
        "pause" => overlays::pause(game),
        "collapse" => overlays::collapse(game),
        // "warren" and the harness default "gameplay" boot straight
        // into a fresh session on the config seed.
        _ => game.transition(StateTransition::StartWarren),
    }
}
