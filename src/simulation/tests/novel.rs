//! Regression coverage for the extended colony, food, and transit loops.

use super::boot;
use crate::simulation::{self, outposts, SIM_DT};
use crate::state::creatures::{Good, Job, Task};
use crate::state::outposts::TransitDirection;
use crate::state::structures::Building;

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
    let encoded = serde_json::to_string(&session).unwrap();
    let restored: crate::state::GameSession = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored.economy.cooked_food, session.economy.cooked_food);
    assert_eq!(restored.economy.raw_food, session.economy.raw_food);
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
}
