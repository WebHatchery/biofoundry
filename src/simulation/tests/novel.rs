//! Regression coverage for the extended colony, food, and transit loops.

use super::boot;
use crate::simulation::{self, outposts, SIM_DT};
use crate::state::creatures::{Good, Job, Task};
use crate::state::outposts::{CargoPriority, TransitDirection};
use crate::state::structures::Building;
use macroquad_toolkit::grid::TilePos;

fn active_outpost(
    seed: u64,
) -> (
    crate::data::GameData,
    crate::state::GameSession,
    macroquad_toolkit::grid::TilePos,
) {
    let (data, mut session) = boot(seed);
    let shrine = session.spawn_tile();
    session.buildings.push(Building::new("worm_shrine", shrine));
    let outpost_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(p, t)| t.walkable() && session.can_place_building(*p))
        .map(|(p, _)| p)
        .unwrap();
    session
        .buildings
        .push(Building::new("outpost", outpost_pos));
    session.ensure_outpost(outpost_pos);
    session.outposts[0].active = true;
    session.worm_awake = true;
    (data, session, outpost_pos)
}

#[test]
fn new_species_and_unlock_paths_are_data_driven() {
    let (data, mut session) = boot(11);
    for id in ["slime_janitor", "bat_courier", "engineer"] {
        assert!(data.species.get(id).is_some(), "missing species {id}");
    }
    session.unlocked.insert("slime_janitor".to_owned());
    session.unlocked.insert("bat_courier".to_owned());
    session.unlocked.insert("engineer".to_owned());
    assert!(simulation::try_attract_slime_janitor(&mut session, &data));
    assert!(simulation::try_attract_bat_courier(&mut session, &data));

    let pit = session.spawn_tile();
    session.buildings.push(Building::new("breeding_pit", pit));
    session.economy.ingots_stock = data.balance.engineer_ingot_cost;
    assert!(simulation::try_breed_engineer(&mut session, &data));
    assert!(session.creatures.iter().any(|c| c.job == Job::Engineer));
}

#[test]
fn janitor_cleans_waste_and_morale_recovers_from_crowding() {
    let (data, mut session) = boot(12);
    session.unlocked.insert("slime_janitor".to_owned());
    let stockpile = session.stockpile_pos();
    session.building_at_mut(stockpile).unwrap().waste = 8.0;
    session.economy.waste = 8.0;
    session.spawn_creature(&data, "slime_janitor", Job::Janitor);
    for _ in 0..120 {
        simulation::tick(&mut session, &data);
    }
    assert!(session.economy.waste < 8.0);
    assert!(session.economy.waste_processed > 0.0);

    while session.creatures.len() < session.usable_warren_capacity(&data) + 8 {
        session.spawn_creature(&data, "goblin", Job::Idle);
    }
    simulation::tick(&mut session, &data);
    assert!(session.creatures.iter().any(|c| c.morale < 1.0));
}

#[test]
fn raw_food_spoils_into_waste_and_cooked_alias_persists() {
    let (data, mut session) = boot(13);
    let farm = session.buildings_of("farm").next().unwrap().pos;
    session
        .building_at_mut(farm)
        .unwrap()
        .add_stock(Good::Mushroom, 12.0);
    for _ in 0..10 {
        simulation::tick(&mut session, &data);
    }
    assert!(session.economy.raw_food > 0.0);
    assert!(session.economy.cooked_food >= 0.0);
    assert!(session.economy.waste > 0.0);
    assert!(session.progress.waste_generated > 0.0);
    let encoded = serde_json::to_string(&session).unwrap();
    let restored: crate::state::GameSession = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored.economy.cooked_food, session.economy.cooked_food);
    assert_eq!(restored.economy.raw_food, session.economy.raw_food);
}

#[test]
fn raw_food_ledger_matches_stock_after_spoilage() {
    let (data, mut session) = boot(130);
    let farm = session.buildings_of("farm").next().unwrap().pos;
    session
        .building_at_mut(farm)
        .unwrap()
        .add_stock(Good::Mushroom, 20.0);

    crate::simulation::colony::tick_spoilage(&mut session, &data, SIM_DT);

    let stock_total: f32 = session
        .buildings
        .iter()
        .map(|building| building.stock(Good::Mushroom))
        .sum();
    assert!((session.economy.raw_food - stock_total).abs() < 0.001);
}

#[test]
fn spoilage_unlocks_janitor_before_any_janitor_exists() {
    let (data, mut session) = boot(131);
    let farm = session.buildings_of("farm").next().unwrap().pos;
    session
        .building_at_mut(farm)
        .unwrap()
        .add_stock(Good::Mushroom, 24.0);
    session.progress.waste_generated = 9.9995;

    assert!(!session.unlocked.contains("slime_janitor"));
    assert!(!session
        .creatures
        .iter()
        .any(|creature| creature.species == "slime_janitor"));
    simulation::tick(&mut session, &data);

    assert!(session.progress.waste_generated >= 10.0);
    assert!(session.unlocked.contains("slime_janitor"));
}

#[test]
fn bat_courier_can_pick_up_from_a_building_over_terrain() {
    let (data, mut session) = boot(14);
    let farm = session.buildings_of("farm").next().unwrap().pos;
    session
        .building_at_mut(farm)
        .unwrap()
        .add_stock(Good::Mushroom, 3.0);
    session.spawn_creature(&data, "bat_courier", Job::Courier);
    let bat = session.creatures.last_mut().unwrap();
    bat.task = Task::GoFetch(farm);
    bat.path = vec![farm];
    for _ in 0..240 {
        simulation::tick(&mut session, &data);
        if session
            .creatures
            .iter()
            .any(|c| c.species == "bat_courier" && c.carried(Good::Mushroom) > 0)
        {
            return;
        }
    }
    panic!("bat courier did not reach a valid building pickup");
}

#[test]
fn worm_shrine_pause_and_mixed_offerings_require_both_resources() {
    let (data, mut session) = boot(15);
    let shrine = session.spawn_tile();
    session.buildings.push(Building::new("worm_shrine", shrine));
    session.economy.food = data.balance.worm_feed_reserve + 20.0;
    session.economy.ingots_stock =
        data.balance.worm_ingot_reserve + data.balance.worm_awaken_ingots;
    session.worm_feeding_paused = true;
    simulation::tick(&mut session, &data);
    assert_eq!(session.worm_fed, 0.0);
    session.worm_feeding_paused = false;
    for _ in 0..((data.balance.worm_awaken_at / data.balance.worm_food_per_min * 60.0 / SIM_DT)
        as usize
        + 10)
    {
        simulation::tick(&mut session, &data);
        if session.worm_awake {
            break;
        }
    }
    assert!(session.worm_fed > 0.0);
    assert!(session.worm_ingots_fed > 0);
    assert!(!session.worm_awake || session.worm_ingots_fed >= data.balance.worm_awaken_ingots);
}

#[test]
fn worm_transit_moves_cargo_and_recovers_when_route_fails() {
    let (data, mut session) = boot(16);
    let shrine = session.spawn_tile();
    session.buildings.push(Building::new("worm_shrine", shrine));
    let outpost_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(p, t)| t.walkable() && session.can_place_building(*p))
        .map(|(p, _)| p)
        .unwrap();
    session
        .buildings
        .push(Building::new("outpost", outpost_pos));
    session.ensure_outpost(outpost_pos);
    session.outposts[0].active = true;
    session.worm_awake = true;
    session.economy.ore_stock = 5;
    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    for _ in 0..((data.balance.worm_transit_time_sec / SIM_DT) as usize + 2) {
        simulation::tick(&mut session, &data);
    }
    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), Some(&5));
    assert!(outposts::start_to_shrine(&mut session, &data, outpost_pos));
    session.outposts[0].active = false;
    simulation::tick(&mut session, &data);
    assert!(session.last_transit_failure.is_some());
    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), Some(&5));
}

#[test]
fn reactivating_a_failed_route_clears_the_global_failure_banner() {
    let (_data, mut session, outpost_pos) = active_outpost(160);
    session.outposts[0].active = false;
    session.outposts[0].last_failure = Some("The worm route collapsed.".to_owned());
    session.last_transit_failure =
        Some("Transit failed because the outpost was inactive.".to_owned());

    assert!(outposts::activate_outpost(&mut session, outpost_pos));
    assert!(session.outposts[0].active);
    assert!(session.outposts[0].last_failure.is_none());
    assert!(session.last_transit_failure.is_none());
}

#[test]
fn worm_transit_accepts_food_only_and_returns_crew_without_cargo() {
    let (data, mut food_session, outpost_pos) = active_outpost(17);
    food_session.creatures.clear();
    food_session.economy.food = data.balance.worm_feed_reserve + 5.0;
    assert!(outposts::start_to_outpost(
        &mut food_session,
        &data,
        outpost_pos
    ));
    outposts::tick_transit(
        &mut food_session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    assert_eq!(
        food_session.outposts[0].cargo.get(&Good::CookedFood),
        Some(&5)
    );

    let (data, mut crew_session, outpost_pos) = active_outpost(18);
    crew_session.economy.food = 0.0;
    crew_session.economy.ore_stock = 0;
    crew_session.economy.ingots_stock = 0;
    let stockpile = crew_session.stockpile_pos();
    assert!(outposts::start_to_outpost(
        &mut crew_session,
        &data,
        outpost_pos
    ));
    outposts::tick_transit(
        &mut crew_session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    assert_eq!(
        crew_session.outposts[0].crew.len(),
        data.balance.outpost_capacity as usize
    );
    assert!(outposts::start_to_shrine(
        &mut crew_session,
        &data,
        outpost_pos
    ));
    outposts::tick_transit(
        &mut crew_session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    assert!(crew_session.outposts[0].crew.is_empty());
    assert!(crew_session.creatures.iter().all(|c| c.tile() == stockpile));
}

#[test]
fn remote_crew_stays_out_of_local_simulation_until_returned() {
    let (data, mut session, outpost_pos) = active_outpost(24);
    session.creatures.clear();
    session.economy.food = 0.0;
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    let stockpile = session.stockpile_pos();
    session.spawn_creature(&data, "goblin", Job::Carrier);
    let crew_id = session.creatures[0].id;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    assert!(session.creatures[0].is_remote());
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    let crew = session
        .creatures
        .iter()
        .find(|creature| creature.id == crew_id)
        .expect("remote crew remains persisted");
    assert_eq!(crew.remote_outpost, Some(outpost_pos));
    assert_eq!(crew.tile(), outpost_pos);
    assert_eq!(session.job_count(Job::Carrier), 0);

    for _ in 0..100 {
        simulation::tick(&mut session, &data);
    }
    let crew = session
        .creatures
        .iter()
        .find(|creature| creature.id == crew_id)
        .expect("remote crew does not desert locally");
    assert_eq!(crew.tile(), outpost_pos);
    assert_eq!(crew.remote_outpost, Some(outpost_pos));
    assert_eq!(session.economy.food, 0.0);

    assert!(outposts::start_to_shrine(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    let crew = session
        .creatures
        .iter()
        .find(|creature| creature.id == crew_id)
        .expect("returned crew remains persisted");
    assert_eq!(crew.remote_outpost, None);
    assert_eq!(crew.tile(), stockpile);
    assert_eq!(session.job_count(Job::Carrier), 1);

    session.outposts[0].active = true;
    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    session.outposts[0].active = false;
    assert!(outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    )
    .is_none());
    let crew = session
        .creatures
        .iter()
        .find(|creature| creature.id == crew_id)
        .expect("failed transit returns its crew");
    assert_eq!(crew.remote_outpost, None);
    assert_eq!(crew.tile(), stockpile);
}

#[test]
fn remote_mine_tasks_do_not_block_a_local_miner() {
    let (data, mut session) = boot(25);
    let mine = session.buildings_of("mine").next().unwrap().pos;
    session.creatures.clear();

    for _ in 0..3 {
        session.spawn_creature(&data, "goblin", Job::Miner);
        let remote = session.creatures.last_mut().unwrap();
        remote.remote_outpost = Some(TilePos::new(4, 4));
        remote.task = Task::WorkMine(mine);
    }
    session.spawn_creature(&data, "goblin", Job::Miner);
    let local_id = session.creatures.last().unwrap().id;

    simulation::tick(&mut session, &data);

    let local = session
        .creatures
        .iter()
        .find(|creature| creature.id == local_id)
        .expect("local miner remains in the warren");
    assert!(matches!(local.task, Task::GoMine(_) | Task::WorkMine(_)));
}

#[test]
fn worm_transit_does_not_overfill_remote_crew_capacity() {
    let (data, mut session, outpost_pos) = active_outpost(19);
    let capacity = data.balance.outpost_capacity as usize;
    session.outposts[0].crew = session
        .creatures
        .iter()
        .take(capacity)
        .map(|creature| creature.id)
        .collect();
    session.economy.ore_stock = 1;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    let completed = outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    let completion = completed.expect("the worm should reach the outpost");
    assert_eq!(completion.direction, TransitDirection::ToOutpost);
    assert_eq!(completion.cargo_units, 6);
    assert_eq!(completion.passenger_count, 0);

    assert_eq!(session.outposts[0].crew.len(), capacity);
    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), Some(&1));
}

#[test]
fn worm_transit_keeps_fractional_food_at_home_until_a_whole_unit_is_ready() {
    let (data, mut session, outpost_pos) = active_outpost(20);
    session.creatures.clear();
    session.economy.food = data.balance.worm_feed_reserve + 0.5;

    assert!(!outposts::start_to_outpost(
        &mut session,
        &data,
        outpost_pos
    ));
    assert!((session.economy.food - (data.balance.worm_feed_reserve + 0.5)).abs() < 1e-3);

    session.economy.food += 1.0;
    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    assert!((session.economy.food - (data.balance.worm_feed_reserve + 0.5)).abs() < 1e-3);
    let completed = outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    let completion = completed.expect("the worm should reach the outpost");
    assert_eq!(completion.direction, TransitDirection::ToOutpost);
    assert_eq!(completion.cargo_units, 1);
    assert_eq!(completion.passenger_count, 0);
    assert_eq!(session.outposts[0].cargo.get(&Good::CookedFood), Some(&1));
}

#[test]
fn worm_transit_appends_new_cargo_to_existing_remote_stacks() {
    let (data, mut session, outpost_pos) = active_outpost(23);
    session.economy.food = data.balance.worm_feed_reserve + 2.0;
    session.economy.ore_stock = 2;
    session.economy.ingots_stock = 2;
    session.outposts[0].cargo.insert(Good::Ore, 3);
    session.outposts[0].cargo.insert(Good::Ingot, 2);
    session.outposts[0].cargo.insert(Good::CookedFood, 1);

    assert!(simulation::outposts::start_to_outpost(
        &mut session,
        &data,
        outpost_pos
    ));
    simulation::outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );

    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), Some(&5));
    assert_eq!(session.outposts[0].cargo.get(&Good::Ingot), Some(&4));
    assert_eq!(session.outposts[0].cargo.get(&Good::CookedFood), Some(&3));
}

#[test]
fn worm_transit_fills_the_hold_in_the_selected_cargo_order() {
    let (data, mut session, outpost_pos) = active_outpost(26);
    session.creatures.clear();
    session.economy.food = data.balance.worm_feed_reserve + 6.0;
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.outposts[0].cargo_priority = CargoPriority::Food;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    let transit = session.worm_transit.as_ref().expect("cargo is in transit");
    assert_eq!(transit.food, 6.0);
    assert_eq!(transit.ore, 0);
    assert_eq!(transit.ingots, 0);

    session.worm_transit = None;
    session.economy.ore_stock = 20;
    session.economy.ingots_stock = 20;
    session.outposts[0].cargo_priority = CargoPriority::Ingots;
    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    let transit = session.worm_transit.as_ref().expect("cargo is in transit");
    assert_eq!(transit.ingots, data.balance.outpost_storage_cap);
    assert_eq!(transit.ore, 0);
    assert_eq!(transit.food, 0.0);
}

#[test]
fn cargo_preview_matches_the_next_transit_and_respects_existing_cargo() {
    let (data, mut session, outpost_pos) = active_outpost(27);
    session.creatures.clear();
    session.economy.ore_stock = 9;
    session.economy.ingots_stock = 9;
    session.economy.food = data.balance.worm_feed_reserve + 9.0;
    session.outposts[0].cargo.insert(Good::Ore, 5);
    session.outposts[0].cargo_priority = CargoPriority::Food;

    let preview = outposts::preview_outbound_cargo(
        &session.outposts[0],
        data.balance.outpost_storage_cap,
        session.economy.ore_stock,
        session.economy.ingots_stock,
        (session.economy.food - data.balance.worm_feed_reserve).floor() as u32,
    );
    assert_eq!(
        preview,
        outposts::CargoLoad {
            ore: 0,
            ingots: 0,
            food: 7
        }
    );

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    let transit = session.worm_transit.as_ref().expect("cargo is in transit");
    assert_eq!(transit.ore, preview.ore);
    assert_eq!(transit.ingots, preview.ingots);
    assert_eq!(transit.food, preview.food as f32);
}

#[test]
fn staffed_outpost_scouting_consumes_food_and_stores_remote_ore() {
    let (data, mut session, _outpost_pos) = active_outpost(28);
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    session.outposts[0].crew = crew;
    session.outposts[0].cargo.insert(Good::CookedFood, 4);

    assert_eq!(
        outposts::expedition_state(&session.outposts[0], &data),
        outposts::ExpeditionState::Scouting {
            progress_percent: 0,
            ore_yield: 6,
            food_cost: 2,
        }
    );

    outposts::tick_expeditions(
        &mut session,
        &data,
        data.balance.outpost_expedition_cycle_sec,
    );

    assert_eq!(session.outposts[0].cargo.get(&Good::CookedFood), Some(&2));
    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), Some(&6));
    assert_eq!(session.outposts[0].expedition_progress, 0.0);
}

#[test]
fn outpost_scouting_pauses_without_provisions_or_hold_room() {
    let (data, mut session, _outpost_pos) = active_outpost(29);
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    session.outposts[0].crew = crew;
    session.outposts[0].cargo.insert(Good::CookedFood, 1);

    assert_eq!(
        outposts::expedition_state(&session.outposts[0], &data),
        outposts::ExpeditionState::NeedsFood {
            required: 2,
            available: 1,
        }
    );
    outposts::tick_expeditions(
        &mut session,
        &data,
        data.balance.outpost_expedition_cycle_sec,
    );
    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), None);

    session.outposts[0].cargo.insert(Good::CookedFood, 2);
    session.outposts[0].cargo.insert(
        Good::Ore,
        data.balance.outpost_storage_cap.saturating_sub(2),
    );
    assert_eq!(
        outposts::expedition_state(&session.outposts[0], &data),
        outposts::ExpeditionState::HoldFull
    );
}

#[test]
fn player_paused_outpost_preserves_remote_food_and_progress() {
    let (data, mut session, _outpost_pos) = active_outpost(30);
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    session.outposts[0].crew = crew;
    session.outposts[0].cargo.insert(Good::CookedFood, 4);
    session.outposts[0].expedition_progress = 8.0;
    session.outposts[0].expedition_paused = true;

    assert_eq!(
        outposts::expedition_state(&session.outposts[0], &data),
        outposts::ExpeditionState::Paused
    );

    outposts::tick_expeditions(
        &mut session,
        &data,
        data.balance.outpost_expedition_cycle_sec,
    );

    assert_eq!(session.outposts[0].cargo.get(&Good::CookedFood), Some(&4));
    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), None);
    assert_eq!(session.outposts[0].expedition_progress, 8.0);
}

#[test]
fn simulation_reports_arrival_after_a_cargo_run_completes() {
    let (data, mut session, outpost_pos) = active_outpost(22);
    session.economy.ore_stock = 1;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    session.worm_transit.as_mut().unwrap().remaining = SIM_DT;

    let report = simulation::tick(&mut session, &data);

    let completion = report
        .transit_completed
        .expect("the tick should report the worm's arrival");
    assert_eq!(completion.direction, TransitDirection::ToOutpost);
    assert_eq!(completion.cargo_units, 6);
    assert_eq!(completion.passenger_count, 4);
    assert_eq!(session.progress.courier_deliveries, 1);
    assert!(session.worm_transit.is_none());
}

#[test]
fn in_flight_worm_transit_survives_a_save_roundtrip() {
    let (data, mut session, outpost_pos) = active_outpost(21);
    session.outposts[0].cargo_priority = CargoPriority::Food;
    session.economy.ore_stock = 3;
    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));

    let json = serde_json::to_string(&session).unwrap();
    let restored: crate::state::GameSession = serde_json::from_str(&json).unwrap();
    let transit = restored.worm_transit.as_ref().expect("transit is saved");

    assert_eq!(transit.direction, TransitDirection::ToOutpost);
    assert_eq!(transit.outpost, outpost_pos);
    assert_eq!(transit.ore, 3);
    assert_eq!(
        transit.passengers.len(),
        data.balance.outpost_capacity as usize
    );
    assert!(restored.outposts[0].active);
    assert_eq!(restored.outposts[0].cargo_priority, CargoPriority::Food);
}
