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
