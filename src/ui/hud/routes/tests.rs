use super::*;
use crate::state::creatures::Good;
use crate::state::structures::Building;

#[test]
fn route_name_keeps_the_network_location_visible() {
    assert_eq!(route_name(1, TilePos::new(7, 9)), "Route 2 · (7, 9)");
}

#[test]
fn route_ledger_adds_a_column_before_cards_can_run_into_the_close_button() {
    assert_eq!(route_column_count(2), 3);
    assert_eq!(route_column_count(6), 3);
    assert_eq!(route_column_count(7), 4);
}

#[test]
fn route_policy_label_explains_automatic_choices() {
    let mut route = Outpost::new(TilePos::new(4, 4));
    assert_eq!(route_policy_label(&route), "Manual route");

    route.auto_return_cargo = true;
    route.auto_resupply_food = true;
    assert_eq!(
        route_policy_label(&route),
        "Auto cargo return · Auto food resupply"
    );

    route.expedition_paused = true;
    assert_eq!(route_policy_label(&route), "Scouting paused");
}

#[test]
fn route_metrics_reports_remote_hold_capacity() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.cargo.insert(Good::Ore, 3);
    route.crew.push(1);

    assert_eq!(
        route_metrics(&session, &data, &route),
        format!(
            "Cargo 3/{} · Crew 1/{}",
            data.balance.outpost_storage_cap, data.balance.outpost_capacity
        )
    );
}

#[test]
fn route_metrics_reports_live_scouting_progress() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.active = true;
    route.crew.push(1);
    route.cargo.insert(Good::CookedFood, 1);
    route.expedition_progress = data.balance.outpost_expedition_cycle_sec * 0.4;

    assert_eq!(
        route_metrics(&session, &data, &route),
        format!(
            "Cargo 1/{} · Crew 1/{} · Scout 40%",
            data.balance.outpost_storage_cap, data.balance.outpost_capacity
        )
    );
}

#[test]
fn route_status_reuses_the_inspection_vocabulary() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 43);
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
    let status = inspect_status(&session, &data, session.building_at(pos).unwrap());

    assert_eq!(status.0, "Ready to load");
}
