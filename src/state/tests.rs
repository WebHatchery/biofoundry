use super::*;
use crate::data::GameData;

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
