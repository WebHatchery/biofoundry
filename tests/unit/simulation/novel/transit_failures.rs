//! Coverage for failed worm transit reporting and safe route recovery.

use super::active_outpost;
use biofoundry::simulation;
use biofoundry::simulation::outposts;
use biofoundry::state::structures::Building;

#[test]
fn failed_transit_is_reported_once_after_payload_recovery() {
    let (data, mut session, outpost_pos) = active_outpost(47);

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    assert!(outposts::start_to_shrine(&mut session, &data, outpost_pos));

    session.outposts[0].active = false;
    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.transit_failed, Some(outpost_pos));
    assert!(report.transit_completed.is_none());
    assert!(session.worm_transit.is_none());
    assert!(session.last_transit_failure.is_some());
    assert!(session.outposts[0].last_failure.is_some());

    let next_report = simulation::tick(&mut session, &data);
    assert_eq!(next_report.transit_failed, None);
}

#[test]
fn reopening_one_failed_route_keeps_another_route_failure_visible() {
    let (_data, mut session, first_pos) = active_outpost(48);
    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a second walkable outpost location");
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(second_pos);

    session.outposts[0].active = false;
    session.outposts[0].last_failure = Some("First route failed.".to_owned());
    session.outposts[1].active = false;
    session.outposts[1].last_failure = Some("Second route failed.".to_owned());
    session.last_transit_failure =
        Some("Transit failed because an outpost was inactive.".to_owned());

    assert!(outposts::activate_outpost(&mut session, first_pos));
    assert!(session.outposts[0].active);
    assert!(session.outposts[0].last_failure.is_none());
    assert!(session.outposts[1].last_failure.is_some());
    assert!(session.last_transit_failure.is_some());
}

#[test]
fn starting_a_healthy_route_keeps_another_failure_visible() {
    let (data, mut session, first_pos) = active_outpost(50);
    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a second walkable outpost location");
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(second_pos);
    session.outposts[0].active = false;
    session.outposts[0].last_failure = Some("First route failed.".to_owned());
    session.outposts[1].active = true;
    session.last_transit_failure =
        Some("Transit failed because an outpost was inactive.".to_owned());
    session.economy.ore_stock = 1;

    assert!(outposts::start_to_outpost(&mut session, &data, second_pos));
    assert_eq!(session.outposts[0].pos, first_pos);
    assert!(session.last_transit_failure.is_some());
}

#[test]
fn syncing_loaded_routes_repairs_and_clears_the_global_failure_banner() {
    let (_data, mut session, _outpost_pos) = active_outpost(49);
    session.outposts[0].last_failure = Some("The worm route collapsed.".to_owned());
    session.last_transit_failure = None;

    outposts::sync_transit_failure_banner(&mut session);
    assert!(session.last_transit_failure.is_some());

    session.outposts[0].last_failure = None;
    outposts::sync_transit_failure_banner(&mut session);
    assert!(session.last_transit_failure.is_none());
}

#[test]
fn asleep_worm_recovers_an_impossible_transit_without_delivering_it() {
    let (data, mut session, outpost_pos) = active_outpost(51);
    session.worm_awake = false;
    session.economy.ore_stock = 0;
    assert!(!outposts::start_to_outpost(
        &mut session,
        &data,
        outpost_pos
    ));

    session.worm_transit = Some(biofoundry::state::outposts::WormTransit {
        outpost: outpost_pos,
        direction: biofoundry::state::outposts::TransitDirection::ToOutpost,
        remaining: 1.0,
        ore: 2,
        ingots: 0,
        food: 0.0,
        passengers: Vec::new(),
    });

    assert!(
        outposts::tick_transit(&mut session, &data, data.balance.worm_transit_time_sec).is_none()
    );
    assert!(session.worm_transit.is_none());
    assert_eq!(session.economy.ore_stock, 2);
    assert!(session.last_transit_failure.is_some());
}
