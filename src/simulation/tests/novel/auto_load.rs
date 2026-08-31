//! Coverage for opt-in standard outbound route loading.

use super::active_outpost;
use crate::simulation;
use crate::simulation::outposts;
use crate::state::creatures::Good;
use crate::state::structures::Building;

#[test]
fn auto_load_starts_a_standard_run_and_reports_the_route() {
    let (data, mut session, outpost_pos) = active_outpost(171);
    session.creatures.clear();
    session.economy.ore_stock = 1;
    session.economy.ingots_stock = 0;
    session.economy.food = data.balance.worm_feed_reserve;
    session.outposts[0].auto_load = true;
    session.outposts[0].cargo.clear();

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.auto_load_started, Some(outpost_pos));
    let transit = session
        .worm_transit
        .as_ref()
        .expect("auto-load is in flight");
    assert_eq!(
        transit.direction,
        crate::state::outposts::TransitDirection::ToOutpost
    );
    assert_eq!(transit.ore, 1);
    assert_eq!(transit.ingots, 0);
    assert_eq!(transit.food, 0.0);
    assert!(transit.passengers.is_empty());
    assert_eq!(session.economy.ore_stock, 0);
}

#[test]
fn auto_load_respects_pause_full_holds_and_cargo_only_dispatch() {
    let (data, mut session, outpost_pos) = active_outpost(172);
    let capacity = data.balance.outpost_storage_cap;
    session.economy.ore_stock = 2;
    session.outposts[0].auto_load = true;
    session.outposts[0].expedition_paused = true;

    assert_eq!(
        outposts::start_auto_load_if_ready(&mut session, &data),
        None
    );
    session.outposts[0].expedition_paused = false;
    session.outposts[0].cargo.insert(Good::Ore, capacity);

    assert_eq!(
        outposts::start_auto_load_if_ready(&mut session, &data),
        None
    );
    session.outposts[0].cargo.clear();
    session.outposts[0].crew_dispatch_limit = Some(0);
    session.economy.ore_stock = 0;
    session.economy.food = data.balance.worm_feed_reserve;

    assert_eq!(
        outposts::start_auto_load_if_ready(&mut session, &data),
        None
    );
    assert!(
        session.worm_transit.is_none(),
        "route {outpost_pos:?} stayed idle"
    );
}

#[test]
fn automatic_loads_rotate_when_multiple_routes_can_receive_cargo() {
    let (data, mut session, first_pos) = active_outpost(173);
    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a second walkable outpost location");
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(second_pos);
    session.outposts[0].auto_load = true;
    session.outposts[1].active = true;
    session.outposts[1].auto_load = true;
    session.economy.ore_stock = 1;

    assert_eq!(
        outposts::start_auto_load_if_ready(&mut session, &data),
        Some(first_pos)
    );
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    session.economy.ore_stock = 1;

    assert_eq!(
        outposts::start_auto_load_if_ready(&mut session, &data),
        Some(second_pos)
    );
}
