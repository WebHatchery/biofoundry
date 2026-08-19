use super::*;
use crate::state::structures::Building;

fn boot() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, data.config.world_seed);
    (data, session)
}

#[test]
fn unstaffed_mine_reads_no_worker_then_exhausted() {
    let (data, mut session) = boot();
    session.creatures.clear();
    let mine = session.buildings_of("mine").next().unwrap().pos;

    // Nobody on it → no worker.
    let b = session.building_at(mine).unwrap();
    assert_eq!(
        building_status(&session, &data, b),
        Some(BuildingStatus::NoWorker)
    );

    // Dry deposit dominates.
    session.building_at_mut(mine).unwrap().reserve = 0.0;
    let b = session.building_at(mine).unwrap();
    assert_eq!(
        building_status(&session, &data, b),
        Some(BuildingStatus::Exhausted)
    );
}

#[test]
fn full_mine_buffer_reads_backed_up() {
    let (data, mut session) = boot();
    let mine = session.buildings_of("mine").next().unwrap().pos;
    // Staff it and cap the buffer.
    session.spawn_creature(&data, "goblin", Job::Miner);
    {
        let m = session.building_at_mut(mine).unwrap();
        m.add_stock(Good::Ore, data.balance.mine_buffer_cap);
    }
    session.creatures.last_mut().unwrap().task = Task::WorkMine(mine);
    let b = session.building_at(mine).unwrap();
    assert_eq!(
        building_status(&session, &data, b),
        Some(BuildingStatus::OutputFull)
    );
}

#[test]
fn starved_blacksmith_and_kiln_read_starved() {
    let (data, mut session) = boot();
    let spot = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("blacksmith", spot));
    session.spawn_creature(&data, "goblin", Job::Smith);
    session.creatures.last_mut().unwrap().task = Task::GoSmith(spot);
    let b = session.building_at(spot).unwrap();
    // No ore, no orders → starved.
    assert_eq!(
        building_status(&session, &data, b),
        Some(BuildingStatus::InputStarved)
    );
}

#[test]
fn trough_waste_and_inactive_outpost_are_visible_states() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 44);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(p, t)| t.walkable() && session.can_place_building(*p))
        .map(|(p, _)| p)
        .unwrap();
    let mut trough = Building::new("feeding_trough", pos);
    trough.waste = 1.0;
    session.buildings.push(trough);
    assert_eq!(
        building_status(&session, &data, session.building_at(pos).unwrap()),
        Some(BuildingStatus::WasteOverflow)
    );

    let outpost_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(p, t)| t.walkable() && session.can_place_building(*p))
        .map(|(p, _)| p)
        .unwrap();
    session
        .buildings
        .push(Building::new("outpost", outpost_pos));
    session.ensure_outpost(outpost_pos);
    assert_eq!(
        building_status(&session, &data, session.building_at(outpost_pos).unwrap()),
        Some(BuildingStatus::InputStarved)
    );
}

#[test]
fn pending_hauls_counts_waiting_goods() {
    let (_data, mut session) = boot();
    let mine = session.buildings_of("mine").next().unwrap().pos;
    let before = pending_hauls(&session);
    session
        .building_at_mut(mine)
        .unwrap()
        .add_stock(Good::Ore, 5.0);
    assert!(pending_hauls(&session) > before);
}
