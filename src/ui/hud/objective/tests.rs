use super::*;
use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::creatures::Job;
use crate::state::outposts::{Outpost, WormTransit};
use crate::state::structures::{BuildSite, Building};
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
    session.unlocked.insert("worm_shrine".to_owned());
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
fn objective_keeps_the_shrine_lock_consistent_with_build_controls() {
    let (data, mut session) = boot();
    session.creatures[0].job = Job::Guard;
    session.won = true;
    session.factory_complete = true;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(objective.progress, "Worm Shrine · forge 20 ingots (0/20)");
    assert_eq!(
        objective.next,
        "Next: forge 20 ingots to unlock the Worm Shrine."
    );
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
    assert_eq!(objective.progress, "Worm awake · Runs 0 · Hauls 0");
    assert_eq!(
        objective.next,
        "Next: tap Blacksmith in Build & Dig, then place it on open floor."
    );
}

#[test]
fn awakened_objective_recovers_an_exhausted_mine_before_promising_more_ingots() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    let mine = session.buildings_of("mine").next().unwrap().pos;
    session.building_at_mut(mine).unwrap().reserve = 0.0;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap Blacksmith in Build & Dig, then place it on open floor."
    );

    let blacksmith = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session
        .buildings
        .push(Building::new("blacksmith", blacksmith));

    let objective = CampaignObjective::current(&session, &data);
    assert_eq!(
        objective.next,
        "Next: tap Mine in Build & Dig, then place a new Mine on open floor."
    );
}

#[test]
fn awakened_objective_waits_for_a_blacksmith_build_site_already_in_progress() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    let site = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.build_sites.push(BuildSite {
        kind: "blacksmith".to_owned(),
        pos: site,
        ore_needed: 10,
        ore_delivered: 3,
    });

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: keep carriers delivering ore to the Blacksmith site."
    );
}

#[test]
fn awakened_objective_waits_for_a_replacement_mine_build_site() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    let existing_mine = session.buildings_of("mine").next().unwrap().pos;
    session.building_at_mut(existing_mine).unwrap().reserve = 0.0;
    let blacksmith = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session
        .buildings
        .push(Building::new("blacksmith", blacksmith));
    let site = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.build_sites.push(BuildSite {
        kind: "mine".to_owned(),
        pos: site,
        ore_needed: 12,
        ore_delivered: 4,
    });

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: keep carriers delivering ore to the new Mine site."
    );
}

#[test]
fn awakened_objective_names_the_visible_smith_recovery() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    let blacksmith = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session
        .buildings
        .push(Building::new("blacksmith", blacksmith));

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap − beside Miner, then + beside Smith in Jobs."
    );
}

#[test]
fn awakened_objective_names_the_visible_carrier_recovery() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    let blacksmith = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session
        .buildings
        .push(Building::new("blacksmith", blacksmith));
    session.creatures.clear();
    session.spawn_creature(&data, "goblin", Job::Smith);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap − beside Smith, then + beside Carrier in Jobs."
    );
}

#[test]
fn awakened_objective_shows_live_transit_progress_when_the_chain_is_viable() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    let blacksmith = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session
        .buildings
        .push(Building::new("blacksmith", blacksmith));
    session.creatures[0].job = Job::Smith;
    session.economy.ingots_forged = 12;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: forge ingots to unlock Worm Transit (12/60)."
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
fn completed_objective_prioritizes_a_ready_outpost_over_an_empty_active_one() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    let first = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", first));
    session.ensure_outpost(first);
    session.outposts[0].active = true;
    let second = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", second));
    session.ensure_outpost(second);
    session.outposts[1].active = true;
    session.outposts[1].cargo.insert(Good::Ore, 4);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap the active Worm Outpost, then send its cargo to the shrine."
    );
}

#[test]
fn completed_objective_does_not_offer_remote_crew_as_a_new_payload() {
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
    session.spawn_creature(&data, "goblin", Job::Carrier);
    session.creatures[0].remote_outpost = Some(pos);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: keep cargo or crew ready at the warren, then load the active Worm Outpost."
    );
}

#[test]
fn completed_objective_does_not_offer_busy_crew_as_a_new_payload() {
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
    session.spawn_creature(&data, "goblin", Job::Carrier);
    session.creatures[0].carrying = Some((Good::Ore, 1));

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: keep cargo or crew ready at the warren, then load the active Worm Outpost."
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
        "Next: tap the active Worm Outpost, then load food for its expedition."
    );
}

#[test]
fn completed_objective_names_the_wait_step_for_a_scouting_outpost() {
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
    outpost.crew.push(crew_id);
    outpost.cargo.insert(Good::CookedFood, 2);
    outpost.expedition_progress = 10.0;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: let the Outpost expedition finish, then return its ore to the shrine."
    );
}

#[test]
fn completed_objective_names_the_resume_step_for_a_paused_outpost() {
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
    outpost.crew.push(crew_id);
    outpost.cargo.insert(Good::CookedFood, 2);
    outpost.expedition_paused = true;

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap the active Worm Outpost, then Resume scouting."
    );
}

#[test]
fn completed_objective_names_the_food_grid_when_an_expedition_is_unprovisioned() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    session.economy.food = data.balance.worm_feed_reserve;
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
    outpost.crew.push(crew_id);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: keep cooked Food above reserve, then load the Outpost expedition."
    );
}

#[test]
fn completed_objective_recovers_a_loaded_inactive_outpost() {
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
    session.outposts[0].cargo.insert(Good::Ore, 2);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap the loaded Worm Outpost, then tap Activate route to return its payload."
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

    assert_eq!(objective.progress, "Worm awake · Runs 3 · Hauls 0");
}

#[test]
fn completed_objective_sums_scouting_hauls_across_outposts() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.progress.courier_deliveries = 3;
    let mut first = Outpost::new(session.spawn_tile());
    first.expeditions_completed = 2;
    let mut second = Outpost::new(macroquad_toolkit::grid::TilePos::new(8, 8));
    second.expeditions_completed = 1;
    session.outposts = vec![first, second];

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(objective.progress, "Worm awake · Runs 3 · Hauls 3");
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
