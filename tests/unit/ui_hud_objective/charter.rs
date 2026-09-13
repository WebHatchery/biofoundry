//! Objective coverage for the Worm Road Charter milestone.

use super::*;
use biofoundry::state::creatures::Good;
use biofoundry::state::outposts::Outpost;
use biofoundry::state::structures::Building;

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

    session.outposts[0].expeditions_completed = data.balance.outpost_charter_haul_goal + 2;
    let objective = CampaignObjective::current(&session, &data);
    assert!(objective.next.contains("Archive 2/5"));
}

#[test]
fn completed_objective_names_the_relay_after_the_first_archive_page() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    session.outpost_charter_claimed = true;
    session.outpost_archive_claims = 1;
    let first_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", first_pos));
    session.ensure_outpost(first_pos);
    let crew_id = session.creatures.first().unwrap().id;
    let first = session.outposts.first_mut().unwrap();
    first.active = true;
    first.crew.push(crew_id);
    first.cargo.insert(Good::CookedFood, 1);
    first.expeditions_completed = data.balance.outpost_relay_haul_goal;

    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| {
            *pos != first_pos && tile.walkable() && session.can_place_building(*pos)
        })
        .map(|(pos, _)| pos)
        .unwrap();
    session.outposts.push(Outpost {
        pos: second_pos,
        active: true,
        ..Outpost::new(second_pos)
    });

    let objective = CampaignObjective::current(&session, &data);

    assert!(objective.next.contains("Worm Road Relay"));
    assert!(objective.next.contains("2/2 active routes"));
    assert!(objective.next.contains("6/6 hauls"));
    assert!(objective.next.contains("+20 ingots"));
}

#[test]
fn completed_objective_names_the_convoy_after_relay() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    session.outpost_charter_claimed = true;
    session.outpost_archive_claims = 1;
    session.outpost_relay_claimed = true;
    let first_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", first_pos));
    session.ensure_outpost(first_pos);
    let crew_id = session.creatures.first().unwrap().id;
    session.outposts[0].active = true;
    session.outposts[0].crew.push(crew_id);
    session.outposts[0].cargo.insert(Good::CookedFood, 1);
    session.outposts[0].expeditions_completed =
        data.balance.outpost_relay_haul_goal + data.balance.outpost_convoy_haul_goal / 2;

    let objective = CampaignObjective::current(&session, &data);

    assert!(objective.next.contains("Worm Road Convoy"));
    assert!(objective.next.contains("1/3 active routes"));
    assert!(objective.next.contains("6/12 hauls"));
    assert!(objective.next.contains("+24 ingots"));
}
