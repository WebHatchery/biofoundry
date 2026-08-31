//! Coverage for repeatable post-Convoy Worm Road Muster contracts.

use crate::simulation::outposts;
use crate::simulation::{self, TickReport};
use crate::state::structures::Building;

fn add_active_routes(session: &mut crate::state::GameSession, count: usize) {
    while session.outposts.len() < count {
        let existing: Vec<_> = session.outposts.iter().map(|route| route.pos).collect();
        let pos = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, tile)| {
                tile.walkable() && session.can_place_building(*pos) && !existing.contains(pos)
            })
            .map(|(pos, _)| pos)
            .unwrap_or_else(|| panic!("the Muster test needs route {}", existing.len() + 1));
        session.buildings.push(Building::new("outpost", pos));
        session.ensure_outpost(pos);
        session.outposts.last_mut().unwrap().active = true;
    }
}

#[test]
fn muster_needs_a_cleared_convoy_four_routes_and_post_convoy_hauls() {
    let (data, mut session, _) = super::novel::active_outpost(177);
    session.outpost_relay_claimed = true;
    session.outpost_convoy_claims = 1;
    let haul_goal = data.balance.outpost_muster_haul_goal;
    session.outposts[0].expeditions_completed =
        data.balance.outpost_relay_haul_goal + data.balance.outpost_convoy_haul_goal + haul_goal
            - 1;
    add_active_routes(
        &mut session,
        data.balance.outpost_muster_route_goal as usize,
    );

    assert_eq!(
        outposts::outpost_muster_progress(&session, &data),
        (data.balance.outpost_muster_route_goal, haul_goal - 1)
    );
    assert_eq!(outposts::claim_outpost_muster(&mut session, &data), 0);

    session.outposts[0].expeditions_completed += 1;
    assert_eq!(outposts::claim_outpost_muster(&mut session, &data), 1);
    assert_eq!(session.outpost_muster_claims, 1);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_muster_reward_ingots
    );
}

#[test]
fn muster_claims_all_outstanding_cycles_once() {
    let (data, mut session, _) = super::novel::active_outpost(178);
    session.outpost_relay_claimed = true;
    session.outpost_convoy_claims = 1;
    session.economy.ingots_stock = 0;
    add_active_routes(
        &mut session,
        data.balance.outpost_muster_route_goal as usize,
    );
    session.outposts[0].expeditions_completed = data.balance.outpost_relay_haul_goal
        + data.balance.outpost_convoy_haul_goal
        + data.balance.outpost_muster_haul_goal * 3;

    assert_eq!(outposts::claim_outpost_muster(&mut session, &data), 3);
    assert_eq!(session.outpost_muster_claims, 3);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_muster_reward_ingots * 3
    );
    assert_eq!(outposts::claim_outpost_muster(&mut session, &data), 0);
}

#[test]
fn old_tick_reports_default_to_no_muster_award() {
    let report = TickReport::default();

    assert_eq!(report.outpost_muster_awarded, 0);
}

#[test]
fn muster_is_not_available_before_a_convoy_claim() {
    let (data, mut session, _) = super::novel::active_outpost(179);
    session.outpost_relay_claimed = true;
    add_active_routes(
        &mut session,
        data.balance.outpost_muster_route_goal as usize,
    );
    session.outposts[0].expeditions_completed = data.balance.outpost_relay_haul_goal
        + data.balance.outpost_convoy_haul_goal
        + data.balance.outpost_muster_haul_goal;

    assert_eq!(outposts::outpost_muster_progress(&session, &data).1, 0);
    assert_eq!(outposts::claim_outpost_muster(&mut session, &data), 0);
}

#[test]
fn muster_award_is_reported_by_the_simulation_tick() {
    let (data, mut session, _) = super::novel::active_outpost(180);
    session.outpost_relay_claimed = true;
    session.outpost_convoy_claims = 1;
    add_active_routes(
        &mut session,
        data.balance.outpost_muster_route_goal as usize,
    );
    session.outposts[0].expeditions_completed = data.balance.outpost_relay_haul_goal
        + data.balance.outpost_convoy_haul_goal
        + data.balance.outpost_muster_haul_goal;

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.outpost_muster_awarded, 1);
}
