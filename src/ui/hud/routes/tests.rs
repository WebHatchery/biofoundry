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
fn route_network_summary_reports_the_whole_warren_network() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 41);
    let first = Outpost::new(TilePos::new(4, 4));
    let mut second = Outpost::new(TilePos::new(8, 8));
    second.active = true;
    second.cargo.insert(Good::Ore, 5);
    second.crew.extend([1, 2]);
    second.ore_scouted = 9;
    session.outposts = vec![first, second];

    assert_eq!(
        route_network_summary(&session, &data),
        "Routes 2 · Active 1 · Held cargo 5 · Remote crew 2 · Ore scouted 9 · Charter 0/3 · +12 ingots · Attention 2"
    );
}

#[test]
fn route_network_summary_confirms_a_claimed_charter() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 42);
    session.outpost_charter_claimed = true;
    session.outpost_archive_claims = 2;
    session.outposts.push(Outpost::new(TilePos::new(4, 4)));

    let summary = route_network_summary(&session, &data);
    assert!(summary.contains("Charter complete"));
    assert!(summary.contains("Pages 2"));
}

#[test]
fn route_network_summary_reports_archive_progress_after_charter() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 44);
    session.outpost_charter_claimed = true;
    session.outposts = vec![Outpost::new(TilePos::new(4, 4))];
    session.outposts[0].expeditions_completed = data.balance.outpost_charter_haul_goal + 2;

    let summary = route_network_summary(&session, &data);

    assert!(summary.contains("Archive 2/5"), "{summary}");
}

#[test]
fn route_network_summary_reports_cached_ingots_across_routes() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 48);
    let mut first = Outpost::new(TilePos::new(4, 4));
    first.signal_cache_upgraded = true;
    first.signal_cache_ingots = 3;
    let mut second = Outpost::new(TilePos::new(8, 8));
    second.signal_cache_upgraded = true;
    second.signal_cache_ingots = 2;
    session.outposts = vec![first, second];

    let summary = route_network_summary(&session, &data);

    assert!(summary.ends_with("Attention 2 · Cache kept 5"), "{summary}");
}

#[test]
fn relay_summary_reports_live_route_and_haul_progress() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 45);
    session.outpost_archive_claims = 1;
    let mut first = Outpost::new(TilePos::new(4, 4));
    first.active = true;
    first.expeditions_completed = 4;
    session.outposts = vec![first];

    let summary = relay_contract_summary(&session, &data).expect("relay should be visible");

    assert!(summary.contains("1/2 routes"), "{summary}");
    assert!(summary.contains("4/6 hauls"), "{summary}");
    assert!(summary.contains("+20 ingots"), "{summary}");
}

#[test]
fn relay_summary_confirms_a_claimed_contract() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 46);
    session.outpost_archive_claims = 1;
    session.outpost_relay_claimed = true;

    assert_eq!(
        relay_contract_summary(&session, &data).as_deref(),
        Some("Worm Road Relay complete · +20 ingots")
    );
}

#[test]
fn convoy_summary_reports_live_progress_after_relay() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 49);
    session.outpost_relay_claimed = true;
    let mut first = Outpost::new(TilePos::new(4, 4));
    first.active = true;
    first.expeditions_completed = data.balance.outpost_relay_haul_goal + 4;
    session.outposts = vec![first];

    let summary = convoy_contract_summary(&session, &data).expect("Convoy should be visible");

    assert!(summary.contains("1/3 routes"), "{summary}");
    assert!(summary.contains("4/12 hauls"), "{summary}");
    assert!(summary.contains("+24 ingots"), "{summary}");
}

#[test]
fn convoy_summary_confirms_a_claimed_contract() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 50);
    session.outpost_relay_claimed = true;
    session.outpost_convoy_claims = 2;

    let summary = convoy_contract_summary(&session, &data).expect("Convoy should be visible");

    assert!(summary.contains("2 cleared"), "{summary}");
    assert!(summary.contains("next 0/12 hauls"), "{summary}");
}

#[test]
fn route_network_summary_counts_active_blockers_as_attention() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 42);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.active = true;
    route.expedition_paused = true;
    route.crew.push(1);
    session.outposts.push(route);

    assert!(route_needs_attention(&session.outposts[0], &data));
    assert!(route_network_summary(&session, &data).ends_with("Attention 1"));
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
fn route_metrics_reports_an_expanded_remote_camp() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.crew_upgraded = true;
    route.crew.extend([1, 2, 3, 4, 5]);

    assert_eq!(
        route_metrics(&session, &data, &route),
        format!(
            "Cargo 0/{} · Crew 5/{}",
            data.balance.outpost_storage_cap, data.balance.outpost_upgraded_capacity
        )
    );
}

#[test]
fn route_metrics_reports_survey_rig_yield() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.survey_upgraded = true;

    assert_eq!(
        route_metrics(&session, &data, &route),
        format!(
            "Cargo 0/{} · Crew 0/{} · Yield {}/scout",
            data.balance.outpost_storage_cap,
            data.balance.outpost_capacity,
            data.balance.outpost_upgraded_ore_per_crew
        )
    );
}

#[test]
fn route_metrics_reports_resonance_beacon_cycle() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.survey_upgraded = true;
    route.resonator_upgraded = true;

    assert_eq!(
        route_metrics(&session, &data, &route),
        format!(
            "Cargo 0/{} · Crew 0/{} · Yield {}/scout · Cycle {:.0}s",
            data.balance.outpost_storage_cap,
            data.balance.outpost_capacity,
            data.balance.outpost_upgraded_ore_per_crew,
            data.balance.outpost_resonator_cycle_sec
        )
    );
}

#[test]
fn route_metrics_reports_deep_survey_yield() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.survey_upgraded = true;
    route.resonator_upgraded = true;
    route.deep_survey_upgraded = true;

    assert_eq!(
        route_metrics(&session, &data, &route),
        format!(
            "Cargo 0/{} · Crew 0/{} · Deep yield {}/scout · Cycle {:.0}s",
            data.balance.outpost_storage_cap,
            data.balance.outpost_capacity,
            data.balance.outpost_deep_survey_ore_per_crew,
            data.balance.outpost_resonator_cycle_sec
        )
    );
}

#[test]
fn route_metrics_reports_signal_cache_payload() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 47);
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.signal_cache_upgraded = true;
    route.signal_cache_ingots = 3;

    assert_eq!(
        route_metrics(&session, &data, &route),
        format!(
            "Cargo 0/{} · Crew 0/{} · Cache +{}/haul · Kept 3",
            data.balance.outpost_storage_cap,
            data.balance.outpost_capacity,
            data.balance.outpost_signal_cache_ingots_per_haul
        )
    );
}

#[test]
fn compact_route_cards_keep_signal_cache_history_in_the_policy_line() {
    let data = GameData::load().expect("embedded game data");
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.signal_cache_upgraded = true;
    route.signal_cache_ingots = 3;
    route.expedition_paused = true;

    assert_eq!(
        compact_route_metrics(&data, &route),
        format!(
            "0/{} cargo · 0/{} crew",
            data.balance.outpost_storage_cap, data.balance.outpost_capacity
        )
    );
    assert_eq!(
        route_policy_summary(&route, &data, true),
        "Paused · Cache +1 · Kept 3"
    );
}

#[test]
fn route_policy_summary_reports_the_worm_road_waypoint() {
    let data = GameData::load().expect("embedded game data");
    let mut route = Outpost::new(TilePos::new(4, 4));
    route.waypoint_upgraded = true;

    assert_eq!(
        route_policy_summary(&route, &data, false),
        "Manual route · Waypoint 12s transit"
    );
    assert_eq!(
        route_policy_summary(&route, &data, true),
        "Manual route · Waypoint 12s"
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
