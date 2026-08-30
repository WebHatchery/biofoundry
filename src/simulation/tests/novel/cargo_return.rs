//! Coverage for cargo-only remote returns.

use super::active_outpost;
use crate::simulation;
use crate::simulation::outposts;
use crate::state::creatures::{Good, Job};
use crate::state::outposts::TransitDirection;
use crate::state::structures::Building;

#[test]
fn cargo_only_return_keeps_remote_crew_at_the_outpost() {
    let (data, mut session, outpost_pos) = active_outpost(27);
    session.creatures.clear();
    session.economy.food = 0.0;
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.spawn_creature(&data, "goblin", Job::Carrier);
    let crew_id = session.creatures[0].id;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    session.outposts[0].cargo.insert(Good::Ore, 4);

    assert!(outposts::start_cargo_to_shrine(
        &mut session,
        &data,
        outpost_pos
    ));
    let transit = session
        .worm_transit
        .as_ref()
        .expect("cargo return is in flight");
    assert!(transit.passengers.is_empty());
    assert!(session.outposts[0].crew.contains(&crew_id));
    assert!(session.outposts[0].cargo.is_empty());

    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    assert_eq!(session.economy.ore_stock, 4);
    assert_eq!(session.outposts[0].crew, vec![crew_id]);
    assert_eq!(session.creatures[0].remote_outpost, Some(outpost_pos));
}

#[test]
fn auto_return_starts_when_the_hold_is_full_and_preserves_remote_provisions() {
    let (data, mut session, outpost_pos) = active_outpost(43);
    session.creatures.clear();
    session.economy.food = 0.0;
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.spawn_creature(&data, "goblin", Job::Carrier);
    let crew_id = session.creatures[0].id;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    let capacity = data.balance.outpost_storage_cap;
    session.outposts[0]
        .cargo
        .insert(Good::Ore, capacity.saturating_sub(2));
    session.outposts[0].cargo.insert(Good::CookedFood, 2);
    session.outposts[0].auto_return_cargo = true;

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.auto_return_started, Some(outpost_pos));
    let transit = session
        .worm_transit
        .as_ref()
        .expect("auto return is in flight");
    assert!(transit.passengers.is_empty());
    assert!(session.outposts[0].crew.contains(&crew_id));
    assert_eq!(session.outposts[0].cargo.get(&Good::Ore), None);
    assert_eq!(session.outposts[0].cargo.get(&Good::CookedFood), Some(&1));
}

#[test]
fn auto_resupply_sends_food_without_dispatching_more_crew() {
    let (data, mut session, outpost_pos) = active_outpost(44);
    session.creatures.clear();
    session.economy.food = data.balance.worm_feed_reserve + 4.0;
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.spawn_creature(&data, "goblin", Job::Carrier);
    let crew_id = session.creatures[0].id;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    session.outposts[0].cargo.remove(&Good::CookedFood);
    session.outposts[0].auto_resupply_food = true;
    session.economy.food = data.balance.worm_feed_reserve + 3.0;

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.auto_resupply_started, Some(outpost_pos));
    let transit = session
        .worm_transit
        .as_ref()
        .expect("food resupply is in flight");
    assert_eq!(transit.direction, TransitDirection::ToOutpost);
    assert_eq!(transit.ore, 0);
    assert_eq!(transit.ingots, 0);
    assert_eq!(transit.food, 3.0);
    assert!(transit.passengers.is_empty());
    assert_eq!(session.outposts[0].crew, vec![crew_id]);
    assert_eq!(session.economy.food, data.balance.worm_feed_reserve);
}

#[test]
fn auto_resupply_respects_a_manual_expedition_pause() {
    let (data, mut session, outpost_pos) = active_outpost(45);
    session.creatures.clear();
    session.economy.food = data.balance.worm_feed_reserve + 3.0;
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.spawn_creature(&data, "goblin", Job::Carrier);

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    session.outposts[0].cargo.remove(&Good::CookedFood);
    session.economy.food = data.balance.worm_feed_reserve + 3.0;
    session.outposts[0].auto_resupply_food = true;
    session.outposts[0].expedition_paused = true;

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.auto_resupply_started, None);
    assert!(session.worm_transit.is_none());
    assert_eq!(session.economy.food, data.balance.worm_feed_reserve + 3.0);
}

#[test]
fn automatic_returns_rotate_across_full_outposts() {
    let (data, mut session, first_pos) = active_outpost(46);
    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a second walkable outpost location");
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(second_pos);
    session.outposts[1].active = true;
    session.outposts[0].auto_return_cargo = true;
    session.outposts[1].auto_return_cargo = true;
    let capacity = data.balance.outpost_storage_cap;
    session.outposts[0].cargo.insert(Good::Ore, capacity);
    session.outposts[1].cargo.insert(Good::Ore, capacity);

    assert_eq!(
        outposts::start_auto_return_if_full(&mut session, &data),
        Some(first_pos)
    );
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    session.outposts[0].cargo.insert(Good::Ore, capacity);

    assert_eq!(
        outposts::start_auto_return_if_full(&mut session, &data),
        Some(second_pos)
    );
}
