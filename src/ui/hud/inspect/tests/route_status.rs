use super::*;
use crate::state::outposts::{TransitDirection, WormTransit};

#[test]
fn active_loaded_outpost_reports_payload_ready() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 15);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable outpost location");
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.outposts[0].active = true;
    session.worm_awake = true;
    session.outposts[0].cargo.insert(Good::Ore, 2);

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Payload ready", dark::POSITIVE)
    );
}

#[test]
fn loaded_route_reports_when_the_shared_worm_is_busy_elsewhere() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 53);
    let first_pos = TilePos::new(4, 4);
    let second_pos = TilePos::new(8, 8);
    session.buildings.push(Building::new("outpost", first_pos));
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(first_pos);
    session.ensure_outpost(second_pos);
    session.outposts[0].active = true;
    session.outposts[1].active = true;
    session.outposts[1].cargo.insert(Good::Ore, 2);
    session.worm_awake = true;
    session.worm_transit = Some(WormTransit {
        outpost: first_pos,
        direction: TransitDirection::ToShrine,
        remaining: 5.0,
        ore: 1,
        ingots: 0,
        food: 0.0,
        passengers: Vec::new(),
    });

    let building = session.building_at(second_pos).expect("second outpost");
    assert_eq!(
        inspect_status(&session, &data, building).0,
        "Worm busy · payload ready"
    );
}

#[test]
fn empty_route_reports_when_a_home_load_is_ready_but_the_worm_is_busy() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 54);
    let first_pos = TilePos::new(4, 4);
    let second_pos = TilePos::new(8, 8);
    session.buildings.push(Building::new("outpost", first_pos));
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(first_pos);
    session.ensure_outpost(second_pos);
    session.outposts[0].active = true;
    session.outposts[1].active = true;
    session.economy.ore_stock = 1;
    session.worm_awake = true;
    session.worm_transit = Some(WormTransit {
        outpost: first_pos,
        direction: TransitDirection::ToOutpost,
        remaining: 5.0,
        ore: 1,
        ingots: 0,
        food: 0.0,
        passengers: Vec::new(),
    });

    let building = session.building_at(second_pos).expect("second outpost");
    assert_eq!(
        inspect_status(&session, &data, building).0,
        "Worm busy · load ready"
    );
}
