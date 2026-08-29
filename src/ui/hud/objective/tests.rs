use super::*;
use crate::data::GameData;
use crate::state::structures::Building;
use crate::state::GameSession;

fn boot() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, data.config.world_seed);
    (data, session)
}

#[test]
fn objective_starts_with_both_security_requirements() {
    let (data, session) = boot();
    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(objective.title, "Secure the warren");
    assert!(objective.progress.contains("Food"));
    assert!(objective.progress.contains("Ore"));
    assert!(!objective.complete);
}

#[test]
fn objective_moves_through_factory_and_shrine() {
    let (data, mut session) = boot();
    session.won = true;
    let factory = CampaignObjective::current(&session, &data);
    assert_eq!(factory.title, "Complete the Biofoundry");

    session.factory_complete = true;
    let shrine = CampaignObjective::current(&session, &data);
    assert_eq!(shrine.progress, "Worm Shrine · ready to build");

    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("worm_shrine", pos));
    let offerings = CampaignObjective::current(&session, &data);
    assert_eq!(offerings.title, "Awaken the Worm");
    assert!(offerings.progress.contains("Offerings"));
}

#[test]
fn objective_marks_the_worm_awake_as_complete() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    let objective = CampaignObjective::current(&session, &data);

    assert!(objective.complete);
    assert_eq!(objective.ratio, 1.0);
    assert_eq!(
        objective.progress,
        "The Colossal Worm is awake · Cargo runs 0"
    );
    assert_eq!(
        objective.next,
        "Next: keep forging ingots to unlock Worm Transit."
    );
}

#[test]
fn completed_objective_points_into_an_unlocked_outpost_route() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: build a Worm Outpost and send cargo through the awakened route."
    );
}

#[test]
fn completed_objective_names_the_next_step_for_an_active_outpost() {
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
    session.outposts[0].active = true;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: send a cargo run through the active Worm Outpost."
    );
}

#[test]
fn completed_objective_counts_successful_cargo_runs() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.progress.courier_deliveries = 3;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.progress,
        "The Colossal Worm is awake · Cargo runs 3"
    );
}

#[test]
fn completed_objective_explains_failed_route_recovery() {
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
    let outpost = session.outposts.first_mut().unwrap();
    outpost.last_failure = Some("The worm route collapsed.".to_owned());

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap the failed Worm Outpost, then tap Activate route before sending cargo."
    );
}
