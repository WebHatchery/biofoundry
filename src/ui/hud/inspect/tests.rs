use super::*;
use crate::state::creatures::Job;
use crate::state::structures::Building;

fn shrine_session() -> (GameData, GameSession, TilePos) {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable shrine location");
    session.buildings.push(Building::new("worm_shrine", pos));
    (data, session, pos)
}

#[test]
fn shrine_reports_food_reserve_blocker() {
    let (data, mut session, pos) = shrine_session();
    session.economy.food = data.balance.worm_feed_reserve;

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Waiting for food reserve", dark::WARNING)
    );
}

#[test]
fn shrine_reports_ingot_reserve_blocker_after_an_offering() {
    let (data, mut session, pos) = shrine_session();
    session.economy.food = data.balance.worm_feed_reserve + 20.0;
    session.worm_fed = data.balance.worm_food_per_offering;
    session.economy.ingots_stock = data.balance.worm_ingot_reserve;

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Waiting for ingot reserve", dark::WARNING)
    );
}

#[test]
fn shrine_is_working_when_reserves_can_fund_the_next_bite() {
    let (data, session, pos) = shrine_session();

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Working", dark::POSITIVE)
    );
}

#[test]
fn breeding_labels_explain_specialist_roles() {
    let data = GameData::load().expect("embedded game data");

    assert_eq!(
        breed_label("hobgoblin", "Hobgoblin", 4, &data),
        "Hobgoblin · ×2 work (4)"
    );
    assert_eq!(
        breed_label("overseer", "Goblin Overseer", 6, &data),
        "Goblin Overseer · aura +35% (6)"
    );
    assert_eq!(
        breed_label("engineer", "Goblin Engineer", 8, &data),
        "Goblin Engineer · Mine +25% (8)"
    );
}

#[test]
fn shrine_labels_a_manual_pause_without_misattributing_it() {
    let (data, mut session, pos) = shrine_session();
    session.worm_feeding_paused = true;

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Paused — reserve protected", dark::WARNING)
    );
}

#[test]
fn failed_outpost_reports_a_route_failure() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 8);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable outpost location");
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.outposts[0].last_failure = Some("The worm route collapsed.".to_owned());

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Route failed", dark::NEGATIVE)
    );
}

#[test]
fn in_flight_outpost_reports_directional_transit() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 9);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable outpost location");
    session
        .buildings
        .push(Building::new("worm_shrine", session.spawn_tile()));
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.outposts[0].active = true;
    session.worm_awake = true;
    session.economy.ore_stock = 1;
    assert!(crate::simulation::outposts::start_to_outpost(
        &mut session,
        &data,
        pos
    ));

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("In transit", dark::POSITIVE)
    );
    assert_eq!(transit_destination(TransitDirection::ToOutpost), "outpost");
    assert_eq!(transit_destination(TransitDirection::ToShrine), "shrine");
}

#[test]
fn empty_outpost_reports_when_no_payload_is_ready_to_load() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 10);
    session.creatures.clear();
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.economy.food = data.balance.worm_feed_reserve;

    assert!(!outpost_has_loadable_payload(&session, &data, 0, 0));

    session.economy.food += 1.0;
    assert!(outpost_has_loadable_payload(&session, &data, 0, 0));
}

#[test]
fn empty_outpost_does_not_count_remote_crew_as_ready_to_load() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 12);
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.economy.food = data.balance.worm_feed_reserve;
    session.creatures.clear();
    session.spawn_creature(&data, "goblin", crate::state::creatures::Job::Carrier);
    session.creatures[0].remote_outpost = Some(TilePos::new(4, 4));

    assert!(!outpost_has_loadable_payload(&session, &data, 0, 0));
}

#[test]
fn active_empty_outpost_reports_whether_payload_is_ready() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 11);
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
    session.creatures.clear();
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.economy.food = data.balance.worm_feed_reserve;

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Awaiting payload", dark::WARNING)
    );

    session.economy.ore_stock = 1;
    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Ready to load", dark::POSITIVE)
    );
}

#[test]
fn blacksmith_queue_reports_when_another_order_can_be_added() {
    let data = GameData::load().expect("embedded game data");
    let pos = TilePos::new(0, 0);
    let mut shop = Building::new("blacksmith", pos);

    assert!(blacksmith_queue_available(&shop, &data));

    for _ in 0..data.balance.order_queue_size {
        shop.orders.push("iron_pickaxe".to_owned());
    }
    assert!(!blacksmith_queue_available(&shop, &data));
}

#[test]
fn inspection_staffing_ignores_remote_crew_and_recognizes_assigned_crew() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 13);
    let mine = session.buildings_of("mine").next().unwrap().pos;
    let blacksmith = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable blacksmith location");
    session
        .buildings
        .push(Building::new("blacksmith", blacksmith));
    session.creatures.clear();
    session.spawn_creature(&data, "goblin", Job::Miner);
    session.creatures[0].task = Task::WorkMine(mine);
    session.creatures[0].remote_outpost = Some(TilePos::new(4, 4));
    session.spawn_creature(&data, "goblin", Job::Smith);
    session.creatures[1].task = Task::Smithing {
        shop: blacksmith,
        remaining: 1.0,
    };
    session.creatures[1].remote_outpost = Some(TilePos::new(4, 4));

    assert!(!local_mine_worker_at(&session.creatures[0], mine));
    assert!(!local_mine_staffed_at(&session.creatures[0], mine));
    assert!(!local_smith_worker_at(&session.creatures[1], blacksmith));
    assert!(!local_smith_staffed_at(&session.creatures[1], blacksmith));

    session.creatures[0].remote_outpost = None;
    session.creatures[0].task = Task::GoMine(mine);
    session.creatures[1].remote_outpost = None;
    session.creatures[1].task = Task::GoSmith(blacksmith);

    assert!(!local_mine_worker_at(&session.creatures[0], mine));
    assert!(local_mine_staffed_at(&session.creatures[0], mine));
    assert!(local_smith_staffed_at(&session.creatures[1], blacksmith));
}

#[test]
fn outpost_return_label_names_each_payload_kind() {
    assert_eq!(
        outpost_return_label(6, 4),
        "Send 6 cargo + 4 crew to shrine"
    );
    assert_eq!(outpost_return_label(6, 0), "Send 6 cargo to shrine");
    assert_eq!(outpost_return_label(0, 4), "Send 4 crew to shrine");
    assert_eq!(outpost_return_label(0, 0), "No cargo or crew to return");
}

#[test]
fn in_flight_payload_summary_names_cargo_and_crew() {
    let transit = WormTransit {
        outpost: TilePos::new(4, 4),
        direction: TransitDirection::ToShrine,
        remaining: 4.0,
        ore: 2,
        ingots: 3,
        food: 4.0,
        passengers: vec![7, 8],
    };

    assert_eq!(
        transit_payload_line(&transit),
        "In flight · 9 cargo · 2 crew"
    );
}
