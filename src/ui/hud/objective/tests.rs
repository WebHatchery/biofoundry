use super::*;
use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::outposts::WormTransit;
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
fn completed_objective_names_the_load_step_for_an_empty_active_outpost() {
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
        "Next: tap the active Worm Outpost, then load it from the warren."
    );
}

#[test]
fn completed_objective_names_the_return_step_for_a_loaded_outpost() {
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
    outpost.active = true;
    outpost.cargo.insert(Good::Ore, 4);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap the active Worm Outpost, then send its cargo to the shrine."
    );
}

#[test]
fn completed_objective_names_the_wait_step_during_transit() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    session.worm_transit = Some(WormTransit {
        outpost: session.spawn_tile(),
        direction: TransitDirection::ToShrine,
        remaining: 4.0,
        ore: 2,
        ingots: 0,
        food: 0.0,
        passengers: Vec::new(),
    });

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: wait for the worm to reach the shrine, then plan the next run."
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
