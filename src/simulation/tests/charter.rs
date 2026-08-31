//! Coverage for the Worm Road Charter milestone.

use crate::simulation::{self, outposts};
use crate::state::creatures::Good;

#[test]
fn charter_awards_ingots_after_the_configured_number_of_hauls() {
    let (data, mut session, _) = super::novel::active_outpost(164);
    let goal = data.balance.outpost_charter_haul_goal;
    let drill = data
        .equipment_def("wormbone_drill")
        .expect("Wormbone Drill data");
    assert!(!session.equipment_unlocked(drill));
    session.economy.ingots_stock = 0;
    for route in &mut session.outposts {
        route.expeditions_completed = goal.saturating_sub(1);
    }

    assert!(!outposts::claim_outpost_charter(&mut session, &data));
    session.outposts[0].expeditions_completed = goal;

    assert!(outposts::claim_outpost_charter(&mut session, &data));
    assert!(session.outpost_charter_claimed);
    assert!(session.equipment_unlocked(drill));
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_charter_reward_ingots
    );
    assert!(!outposts::claim_outpost_charter(&mut session, &data));
}

#[test]
fn charter_claims_from_a_real_expedition_completion() {
    let (data, mut session, _) = super::novel::active_outpost(165);
    let goal = data.balance.outpost_charter_haul_goal;
    let crew: Vec<u32> = session
        .creatures
        .iter()
        .take(2)
        .map(|creature| creature.id)
        .collect();
    session.economy.ingots_stock = 0;
    let route = &mut session.outposts[0];
    route.storage_upgraded = true;
    route.crew_upgraded = true;
    route.survey_upgraded = true;
    route.resonator_upgraded = true;
    route.crew = crew;
    route.cargo.insert(Good::CookedFood, 2);
    route.expeditions_completed = goal.saturating_sub(1);
    route.expedition_progress = outposts::expedition_cycle_sec(route, &data) - simulation::SIM_DT;

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.expedition_completed.len(), 1);
    assert!(report.outpost_charter_awarded);
    assert!(session.outpost_charter_claimed);
    assert_eq!(
        session.economy.ingots_stock,
        data.balance.outpost_charter_reward_ingots
    );
}
