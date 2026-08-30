use super::*;
use crate::data::GameData;
use crate::state::creatures::Task;
use macroquad_toolkit::notifications::{LoggedNotification, NotificationType};

#[test]
fn session_boots_from_config() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, data.config.world_seed);

    assert_eq!(session.tick, 0);
    assert_eq!(session.world.tiles.width, data.config.world_width);
    assert_eq!(
        session.creatures.len() as u32,
        data.balance.start_miners + data.balance.start_carriers + data.balance.start_cooks
    );
    assert!(!session.patch_regrow.is_empty());
    assert!(!session.vein_ore.is_empty());
    assert!(session.economy.food > 0.0);
}

#[test]
fn event_history_survives_session_serialization() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 42);
    session.event_history = vec![LoggedNotification {
        message: "A saved warning remains reviewable.".to_owned(),
        notification_type: NotificationType::Warning,
    }];

    let encoded = serde_json::to_value(&session).unwrap();
    let restored: GameSession = serde_json::from_value(encoded).unwrap();

    assert_eq!(restored.event_history, session.event_history);
}

#[test]
fn starting_buildings_land_on_walkable_floor() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, data.config.world_seed);

    // Stockpile, cook pot, farm, and the prebuilt mine.
    assert_eq!(session.buildings.len(), 4);
    for building in &session.buildings {
        assert!(
            session.world.tiles.get(building.pos).unwrap().walkable(),
            "building {} at {:?} must be walkable",
            building.kind,
            building.pos
        );
        assert!(data.buildings.get(&building.kind).is_some());
    }

    // The prebuilt mine sits beside a vein and carries a full deposit.
    let mine = session.buildings_of("mine").next().expect("prebuilt mine");
    assert!(session.adjacent_ore_vein(mine.pos).is_some());
    assert!((mine.reserve - data.balance.mine_reserve).abs() < 1e-3);
}

#[test]
fn placement_rules_reject_occupied_and_rock_tiles() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);

    let farm_pos = session.buildings_of("farm").next().unwrap().pos;
    assert!(!session.can_place_building(farm_pos), "occupied by farm");

    let rock = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(_, t)| **t == Tile::Rock)
        .map(|(pos, _)| pos)
        .unwrap();
    assert!(!session.can_place_building(rock), "rock is not floor");

    let open = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, t)| **t == Tile::Floor && session.building_at(*pos).is_none())
        .map(|(pos, _)| pos)
        .unwrap();
    assert!(session.can_place_building(open));

    // Dig marks toggle on rock only.
    assert!(session.toggle_dig_mark(rock));
    assert!(session.dig_marks.contains(&rock));
    assert!(session.toggle_dig_mark(rock));
    assert!(!session.dig_marks.contains(&rock));
    assert!(!session.toggle_dig_mark(open));
}

#[test]
fn reassignment_moves_one_goblin_and_resets_its_task() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);
    let miners_before = session.job_count(Job::Miner);
    let carriers_before = session.job_count(Job::Carrier);

    let moved = session.reassign(Job::Miner, Job::Carrier, |s| {
        data.species.get(s).map(|d| d.reassignable).unwrap_or(false)
    });

    assert!(moved);
    assert_eq!(session.job_count(Job::Miner), miners_before - 1);
    assert_eq!(session.job_count(Job::Carrier), carriers_before + 1);
}

#[test]
fn beetles_cannot_be_reassigned() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);
    // Make everyone a beetle-only pool for the source job.
    session.creatures.clear();
    session.spawn_creature(&data, "beetle", Job::Carrier);

    let moved = session.reassign(Job::Carrier, Job::Miner, |s| {
        data.species.get(s).map(|d| d.reassignable).unwrap_or(false)
    });

    assert!(!moved);
}

#[test]
fn specialist_only_security_handoff_is_non_viable() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert!(session.is_non_viable(&data));
}

#[test]
fn remote_crew_is_not_a_local_worker_or_recovery_option() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);
    session.creatures.clear();
    session.spawn_creature(&data, "goblin", Job::Miner);
    session.creatures[0].remote_outpost = Some(TilePos::new(4, 4));
    session.won = true;

    assert_eq!(session.local_creature_count(), 0);
    assert_eq!(session.job_count(Job::Miner), 0);
    assert!(!session.reassign(Job::Miner, Job::Guard, |species| {
        data.species
            .get(species)
            .map(|definition| definition.reassignable)
            .unwrap_or(false)
    }));
    assert!(session.is_non_viable(&data));
}

#[test]
fn remote_crew_does_not_inflate_local_crowding() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);
    let crowded_before = session.overcrowding_ratio(&data);

    session.creatures[0].remote_outpost = Some(TilePos::new(4, 4));

    assert!(session.overcrowding_ratio(&data) < crowded_before);
}

#[test]
fn loading_route_ownership_rebuilds_remote_crew_markers() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 6);
    let outpost_pos = TilePos::new(4, 4);
    let crew_id = session.creatures[0].id;
    session.ensure_outpost(outpost_pos);
    session.outposts[0].crew.push(crew_id);
    let mine_pos = session.buildings_of("mine").next().unwrap().pos;
    session.creatures[0].task = Task::GoMine(mine_pos);

    session.sync_remote_crew_state();

    assert_eq!(session.creatures[0].remote_outpost, Some(outpost_pos));
    assert_eq!(session.creatures[0].tile(), outpost_pos);
    assert_eq!(session.creatures[0].task, Task::Idle);
    assert_eq!(session.job_count(Job::Miner), 2);
}

#[test]
fn loading_route_ownership_discards_unknown_and_duplicate_crew_ids() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 6);
    let first = TilePos::new(4, 4);
    let second = TilePos::new(5, 5);
    let crew_id = session.creatures[0].id;
    session.ensure_outpost(first);
    session.ensure_outpost(second);
    session.outposts[0].crew = vec![crew_id, 9999, crew_id];
    session.outposts[1].crew = vec![crew_id, 8888];

    session.sync_remote_crew_state();

    assert_eq!(session.outposts[0].crew, vec![crew_id]);
    assert!(session.outposts[1].crew.is_empty());
    assert_eq!(session.creatures[0].remote_outpost, Some(first));
}

#[test]
fn loading_route_ownership_discards_duplicate_in_flight_passengers() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 6);
    let outpost_pos = TilePos::new(4, 4);
    let crew_id = session.creatures[0].id;
    session.ensure_outpost(outpost_pos);
    session.outposts[0].crew.push(crew_id);
    session.worm_transit = Some(WormTransit {
        outpost: outpost_pos,
        direction: outposts::TransitDirection::ToShrine,
        remaining: 3.0,
        ore: 1,
        ingots: 0,
        food: 0.0,
        passengers: vec![crew_id, 7777],
    });

    session.sync_remote_crew_state();

    assert_eq!(session.outposts[0].crew, vec![crew_id]);
    assert!(session.worm_transit.as_ref().unwrap().passengers.is_empty());
    assert_eq!(session.creatures[0].remote_outpost, Some(outpost_pos));
}

#[test]
fn viable_workers_keep_the_security_handoff_recoverable() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);
    session.won = true;

    assert!(!session.is_non_viable(&data));
}

#[test]
fn awakened_warren_is_viable_even_without_creatures() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 5);
    session.worm_awake = true;
    session.creatures.clear();

    assert!(!session.is_non_viable(&data));
}

#[test]
fn progression_counter_lookup_includes_study_metrics() {
    let progress = wildlife::Progress {
        specimens: 3,
        knowledge: 4.8,
        ..Default::default()
    };

    assert_eq!(progress.counter("specimens"), 3);
    assert_eq!(progress.counter("knowledge"), 4);
}
