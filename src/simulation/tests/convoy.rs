//! Coverage for repeatable Worm Road Convoy contracts.

use crate::simulation::outposts;
use crate::simulation::{self, TickReport};

#[test]
fn convoy_needs_relay_three_active_routes_and_post_relay_hauls() {
    let (data, mut session, _) = super::novel::active_outpost(175);
    session.worm_awake = true;
    session.outpost_relay_claimed = true;
    let route_goal = data.balance.outpost_convoy_route_goal;
    let haul_goal = data.balance.outpost_convoy_haul_goal;

    session.outposts[0].expeditions_completed = data.balance.outpost_relay_haul_goal + haul_goal;
    assert_eq!(
        outposts::outpost_convoy_progress(&session, &data),
        (1, haul_goal)
    );
    assert_eq!(outposts::claim_outpost_convoy(&mut session, &data), 0);

    for index in 1..route_goal as usize {
        let pos = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, tile)| {
                tile.walkable()
                    && session.can_place_building(*pos)
                    && !session.outposts.iter().any(|route| route.pos == *pos)
            })
            .map(|(pos, _)| pos)
            .unwrap_or_else(|| panic!("the Convoy test needs route site {index}"));
        session
            .buildings
            .push(crate::state::structures::Building::new("outpost", pos));
        session.ensure_outpost(pos);
        session.outposts[index].active = true;
    }

    assert_eq!(outposts::claim_outpost_convoy(&mut session, &data), 1);
    assert_eq!(session.outpost_convoy_claims, 1);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_convoy_reward_ingots
    );
    assert_eq!(outposts::claim_outpost_convoy(&mut session, &data), 0);
}

#[test]
fn convoy_claims_all_outstanding_cycles_once() {
    let (data, mut session, _) = super::novel::active_outpost(176);
    session.worm_awake = true;
    session.outpost_relay_claimed = true;
    session.outpost_convoy_claims = 1;
    session.economy.ingots_stock = 0;
    session.outposts[0].active = true;
    session.outposts[0].expeditions_completed =
        data.balance.outpost_relay_haul_goal + data.balance.outpost_convoy_haul_goal * 3;
    let route_goal = data.balance.outpost_convoy_route_goal;
    for index in 1..route_goal as usize {
        let pos = session
            .world
            .tiles
            .iter_with_pos()
            .find(|(pos, tile)| {
                tile.walkable()
                    && session.can_place_building(*pos)
                    && !session.outposts.iter().any(|route| route.pos == *pos)
            })
            .map(|(pos, _)| pos)
            .unwrap();
        session
            .buildings
            .push(crate::state::structures::Building::new("outpost", pos));
        session.ensure_outpost(pos);
        session.outposts[index].active = true;
        session.outposts[index].expeditions_completed = 0;
    }

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.outpost_convoy_awarded, 2);
    assert_eq!(session.outpost_convoy_claims, 3);
    assert!(
        session.economy.ingots_stock >= data.balance.outpost_convoy_reward_ingots * 2,
        "the Convoy reward should be present alongside any same-tick production"
    );
}

#[test]
fn old_tick_reports_default_to_no_convoy_award() {
    let report = TickReport::default();

    assert_eq!(report.outpost_convoy_awarded, 0);
}
