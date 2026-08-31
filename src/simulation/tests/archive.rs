//! Coverage for repeatable Worm Road Archive rewards.

use super::novel::active_outpost;
use crate::simulation::outposts;
use crate::simulation::{self, SIM_DT};
use crate::state::creatures::Good;

#[test]
fn archive_waits_for_the_charter_and_pays_each_page_once() {
    let (data, mut session, _) = active_outpost(171);
    let charter_goal = data.balance.outpost_charter_haul_goal;
    let archive_goal = data.balance.outpost_archive_haul_goal;
    session.outposts[0].expeditions_completed = charter_goal + archive_goal;

    assert_eq!(outposts::claim_outpost_archive(&mut session, &data), 0);
    assert_eq!(session.economy.ingots_stock, 0);

    session.outpost_charter_claimed = true;
    assert_eq!(outposts::claim_outpost_archive(&mut session, &data), 1);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_archive_reward_ingots
    );
    assert_eq!(session.outpost_archive_claims, 1);
    assert_eq!(outposts::outpost_archive_progress(&session, &data), 0);
    assert_eq!(outposts::claim_outpost_archive(&mut session, &data), 0);

    session.outposts[0].expeditions_completed = charter_goal + archive_goal * 3;
    assert_eq!(outposts::claim_outpost_archive(&mut session, &data), 2);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_archive_reward_ingots * 3
    );
    assert_eq!(session.outpost_archive_claims, 3);
}

#[test]
fn archive_reward_is_emitted_after_a_real_post_charter_haul() {
    let (data, mut session, _) = active_outpost(172);
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    let route = &mut session.outposts[0];
    route.active = true;
    route.crew = crew;
    route.cargo.insert(Good::CookedFood, 2);
    route.expeditions_completed =
        data.balance.outpost_charter_haul_goal + data.balance.outpost_archive_haul_goal - 1;
    route.expedition_progress = outposts::expedition_cycle_sec(route, &data) - SIM_DT;
    session.outpost_charter_claimed = true;

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.expedition_completed.len(), 1);
    assert_eq!(report.outpost_archive_awarded, 1);
    assert_eq!(session.outpost_archive_claims, 1);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_archive_reward_ingots
    );
}
