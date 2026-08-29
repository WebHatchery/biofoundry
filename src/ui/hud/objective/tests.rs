use super::*;
use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::creatures::Job;
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
    session.creatures[0].job = Job::Guard;
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
fn objective_keeps_the_guard_handoff_as_the_next_requirement() {
    let (data, mut session) = boot();
    session.won = true;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(objective.title, "Finish the security handoff");
    assert!(objective.progress.contains("Guard 0/1"));
    assert_eq!(
        objective.next,
        "Next: tap − beside Miner, then + beside Guard in Jobs."
    );
    assert!(!objective.complete);
    assert!((objective.ratio - 2.0 / 3.0).abs() < f32::EPSILON);

    session.creatures[0].job = Job::Guard;
    assert_eq!(
        CampaignObjective::current(&session, &data).title,
        "Complete the Biofoundry"
    );
}

#[test]
fn objective_does_not_promise_a_disabled_guard_assignment() {
    let (data, mut session) = boot();
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: free a worker, then tap + beside Guard in Jobs."
    );
}

#[test]
fn objective_names_the_worker_to_free_before_assigning_a_smith() {
    let (data, mut session) = boot();
    session.won = true;
    session.creatures[0].job = Job::Guard;
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("blacksmith", pos));

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap − beside Miner, then + beside Smith in Jobs."
    );
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
    let crew_id = session.creatures.first().unwrap().id;
    let outpost = session.outposts.first_mut().unwrap();
    outpost.active = true;
    outpost.cargo.insert(Good::Ore, 4);
    outpost.crew.push(crew_id);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap the active Worm Outpost, then send its cargo and crew to the shrine."
    );
}

#[test]
fn completed_objective_explains_when_an_active_outpost_has_nothing_ready() {
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
    session.creatures.clear();
    session.economy.food = data.balance.worm_feed_reserve;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: keep cargo or crew ready at the warren, then load the active Worm Outpost."
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
