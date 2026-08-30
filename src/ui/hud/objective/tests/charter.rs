//! Objective coverage for the Worm Road Charter milestone.

use super::*;
use crate::state::creatures::Good;
use crate::state::structures::Building;

#[test]
fn completed_objective_tracks_the_charter_across_remote_routes() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    let crew_id = session.creatures.first().unwrap().id;
    let route = session.outposts.first_mut().unwrap();
    route.active = true;
    route.crew.push(crew_id);
    route.cargo.insert(Good::CookedFood, 1);
    route.expeditions_completed = data.balance.outpost_charter_haul_goal.saturating_sub(1);

    let objective = CampaignObjective::current(&session, &data);

    assert!(objective.next.contains("Charter 2/3"));
    assert!(objective.next.contains("+12 ingots"));
}

#[test]
fn completed_objective_returns_to_route_guidance_after_charter_claim() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    let crew_id = session.creatures.first().unwrap().id;
    let route = session.outposts.first_mut().unwrap();
    route.active = true;
    route.crew.push(crew_id);
    route.cargo.insert(Good::CookedFood, 1);
    session.outpost_charter_claimed = true;

    let objective = CampaignObjective::current(&session, &data);

    assert!(objective.next.contains("let the Outpost expedition finish"));
    assert!(!objective.next.contains("Charter"));
}
