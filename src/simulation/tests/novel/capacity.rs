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

#[test]
fn survey_rig_requires_logistics_expansions_and_spends_ingots_once() {
    let (data, mut session, outpost_pos) = active_outpost(160);
    session.economy.ingots_stock = data.balance.outpost_survey_upgrade_ingots;

    assert!(!outposts::upgrade_outpost_survey(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_survey_upgrade_ingots
    );

    session.outposts[0].storage_upgraded = true;
    session.outposts[0].crew_upgraded = true;
    assert!(outposts::upgrade_outpost_survey(
        &mut session,
        &data,
        outpost_pos
    ));
    assert_eq!(session.economy.ingots_stock, 0);
    assert!(session.outposts[0].survey_upgraded);
    assert_eq!(
        outposts::ore_per_crew(&session.outposts[0], &data),
        data.balance.outpost_upgraded_ore_per_crew
    );
    assert!(!outposts::upgrade_outpost_survey(
        &mut session,
        &data,
        outpost_pos
    ));
}

#[test]
fn survey_rig_increases_the_next_expedition_yield() {
    let (data, mut session, _outpost_pos) = active_outpost(161);
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    let route = &mut session.outposts[0];
    route.storage_upgraded = true;
    route.crew_upgraded = true;
    route.survey_upgraded = true;
    route.crew = crew;
    route
        .cargo
        .insert(crate::state::creatures::Good::CookedFood, 2);
    route.expedition_progress = data.balance.outpost_expedition_cycle_sec;

    let completed = outposts::tick_expeditions(&mut session, &data, 0.0);

    assert_eq!(
        completed[0].ore,
        2 * data.balance.outpost_upgraded_ore_per_crew
    );
    assert_eq!(session.outposts[0].ore_scouted, completed[0].ore);
}
