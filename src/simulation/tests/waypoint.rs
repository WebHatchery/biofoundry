//! Coverage for the post-Convoy Worm Road Waypoint.

use super::*;
use crate::simulation::outposts;
use crate::state::structures::Building;
use macroquad_toolkit::grid::TilePos;

fn active_outpost(seed: u64) -> (GameData, GameSession, TilePos) {
    let (data, mut session) = boot(seed);
    let shrine = session.spawn_tile();
    session.buildings.push(Building::new("worm_shrine", shrine));
    let outpost_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("an open Outpost site");
    session
        .buildings
        .push(Building::new("outpost", outpost_pos));
    session.ensure_outpost(outpost_pos);
    session.outposts[0].active = true;
    session.worm_awake = true;
    (data, session, outpost_pos)
}

#[test]
fn waypoint_requires_the_signal_cache_and_a_cleared_convoy() {
    let (data, mut session, outpost_pos) = active_outpost(170);
    let outpost = &mut session.outposts[0];
    outpost.deep_survey_upgraded = true;
    session.economy.ingots_stock = data.balance.outpost_waypoint_upgrade_ingots;

    assert!(!outposts::upgrade_outpost_waypoint(
        &mut session,
        &data,
        outpost_pos
    ));
    session.outposts[0].signal_cache_upgraded = true;
    assert!(!outposts::upgrade_outpost_waypoint(
        &mut session,
        &data,
        outpost_pos
    ));

    session.outpost_convoy_claims = 1;
    assert!(outposts::upgrade_outpost_waypoint(
        &mut session,
        &data,
        outpost_pos
    ));
}

#[test]
fn waypoint_spends_ingots_once_and_shortens_route_transit() {
    let (data, mut session, outpost_pos) = active_outpost(171);
    session.outposts[0].deep_survey_upgraded = true;
    session.outposts[0].signal_cache_upgraded = true;
    session.outpost_convoy_claims = 1;
    session.economy.ingots_stock = data.balance.outpost_waypoint_upgrade_ingots;

    assert!(outposts::upgrade_outpost_waypoint(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(session.economy.ingots_stock, 0);
    assert!(session.outposts[0].waypoint_upgraded);
    assert_eq!(
        outposts::transit_time_sec(&session.outposts[0], &data),
        data.balance.outpost_waypoint_transit_time_sec
    );
    assert!(!outposts::upgrade_outpost_waypoint(
        &mut session,
        &data,
        outpost_pos
    ));
}

#[test]
fn waypoint_route_uses_the_shorter_time_for_an_outbound_transit() {
    let (data, mut session, outpost_pos) = active_outpost(172);
    session.outposts[0].waypoint_upgraded = true;
    session.economy.ore_stock = 1;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    assert_eq!(
        session.worm_transit.expect("outbound transit").remaining,
        data.balance.outpost_waypoint_transit_time_sec
    );
}
