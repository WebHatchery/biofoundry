//! Coverage for Outpost camp capacity expansion.

use super::active_outpost;
use crate::simulation::outposts;
use crate::state::creatures::Job;

#[test]
fn outpost_camp_upgrade_spends_ingots_once_and_expands_crew_capacity() {
    let (data, mut session, outpost_pos) = active_outpost(158);
    session.economy.ingots_stock = data.balance.outpost_crew_upgrade_ingots;

    assert_eq!(
        outposts::crew_capacity(&session.outposts[0], &data),
        data.balance.outpost_capacity
    );
    assert!(outposts::upgrade_outpost_crew(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(session.economy.ingots_stock, 0);
    assert!(session.outposts[0].crew_upgraded);
    assert_eq!(
        outposts::crew_capacity(&session.outposts[0], &data),
        data.balance.outpost_upgraded_capacity
    );
    assert!(!outposts::upgrade_outpost_crew(
        &mut session,
        &data,
        outpost_pos
    ));
}

#[test]
fn upgraded_outpost_transit_can_dispatch_the_extra_scouts() {
    let (data, mut session, outpost_pos) = active_outpost(159);
    session.outposts[0].crew_upgraded = true;
    session.outposts[0].crew_dispatch_limit = Some(data.balance.outpost_upgraded_capacity);
    session.spawn_creature(&data, "goblin", Job::Carrier);
    session.economy.ore_stock = 1;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    let transit = session.worm_transit.as_ref().expect("crew is in transit");
    assert_eq!(
        transit.passengers.len(),
        data.balance.outpost_upgraded_capacity as usize
    );
}
