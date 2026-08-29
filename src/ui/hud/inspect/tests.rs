use super::*;
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
