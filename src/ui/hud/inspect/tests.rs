use super::*;
use crate::state::creatures::Job;
use crate::state::outposts::CargoPriority;
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
fn locked_specialist_marker_uses_font_safe_ascii() {
    assert_eq!(LOCKED_SPECIALIST_MARKER, "[L]");
}

#[test]
fn locked_specialists_show_their_live_unlock_progress() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 13);
    session.economy.ingots_forged = 7;

    assert_eq!(
        breeding_unlock_hint(&session, &data, "hobgoblin").as_deref(),
        Some("Next: forge 30 ingots (7/30)")
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

    assert!(!outpost_has_loadable_payload(
        &session,
        &data,
        0,
        0,
        data.balance.outpost_storage_cap,
        None,
    ));

    session.economy.food += 1.0;
    assert!(outpost_has_loadable_payload(
        &session,
        &data,
        0,
        0,
        data.balance.outpost_storage_cap,
        None,
    ));
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

    assert!(!outpost_has_loadable_payload(
        &session,
        &data,
        0,
        0,
        data.balance.outpost_storage_cap,
        None,
    ));
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
fn empty_awakened_outpost_explains_the_missing_scout_crew() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 13);
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

    assert_eq!(
        outpost_expedition_hint(&data, &session.outposts[0]).as_deref(),
        Some("Need scout crew")
    );
}

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
fn inactive_loaded_outpost_reports_payload_recovery() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 17);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable outpost location");
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.outposts[0].cargo.insert(Good::Ore, 1);

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Payload held · route inactive", dark::WARNING)
    );
}

#[test]
fn active_outpost_before_awakening_reports_route_state() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 16);
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

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Route active", dark::POSITIVE)
    );
}

#[test]
fn paused_outpost_reports_its_manual_scouting_state() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 17);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable outpost location");
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.worm_awake = true;
    session.outposts[0].active = true;
    session.outposts[0].crew.push(session.creatures[0].id);
    session.outposts[0].expedition_paused = true;

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Scouting paused", dark::WARNING)
    );
}

#[test]
fn staffed_outpost_reports_food_and_hold_blockers() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 18);
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable outpost location");
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);
    session.worm_awake = true;
    session.outposts[0].active = true;
    session.outposts[0].crew.push(session.creatures[0].id);

    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Needs scout food", dark::WARNING)
    );

    session.outposts[0].cargo.insert(Good::CookedFood, 1);
    session.outposts[0].cargo.insert(
        Good::Ore,
        data.balance.outpost_storage_cap.saturating_sub(1),
    );
    assert_eq!(
        inspect_status(&session, &data, session.building_at(pos).unwrap()),
        ("Outpost hold full", dark::NEGATIVE)
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
fn blacksmith_input_hint_names_the_queued_order_and_shortfall() {
    let data = GameData::load().expect("embedded game data");
    let mut shop = Building::new("blacksmith", TilePos::new(0, 0));
    shop.orders.push("iron_pickaxe".to_owned());
    shop.add_stock(Good::Ore, 1.0);

    assert_eq!(
        blacksmith_input_hint(&shop, &data),
        "Needs 1 ore · next Iron Pickaxe"
    );
}

#[test]
fn smelter_input_hint_names_each_missing_material() {
    let data = GameData::load().expect("embedded game data");
    let mut smelter = Building::new("smelter", TilePos::new(0, 0));

    assert_eq!(
        smelter_input_hint(&smelter, &data),
        "Needs 1 ore + 1 charcoal"
    );

    smelter.add_stock(Good::Ore, data.balance.smelt_batch_ore as f32);
    assert_eq!(smelter_input_hint(&smelter, &data), "Needs 1 charcoal");
}

#[test]
fn kitchen_input_hints_name_the_missing_material() {
    let data = GameData::load().expect("embedded game data");
    let pot = Building::new("cook_pot", TilePos::new(0, 0));

    assert_eq!(
        cook_pot_input_hint(&pot, &data),
        format!(
            "Needs {} mushrooms",
            (data.balance.cook_batch_mushrooms as f32 * data.balance.raw_recipe_multiplier).ceil()
                as u32
        )
    );
    assert_eq!(kiln_input_hint(), "Needs 1 wood");
}

#[test]
fn waste_hint_explains_the_available_recovery_path() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 33);
    let farm = session.buildings_of("farm").next().unwrap().pos;
    session.building_at_mut(farm).unwrap().waste = 2.5;

    assert_eq!(
        waste_inspection_hint(&session, &data, session.building_at(farm).unwrap()),
        "Waste 2.5 · Secure warren first"
    );

    session.won = true;
    session.spawn_creature(&data, "goblin", Job::Guard);
    session.unlocked.insert("slime_janitor".to_owned());
    assert_eq!(
        waste_inspection_hint(&session, &data, session.building_at(farm).unwrap()),
        "Waste 2.5 · Attract Slime"
    );

    session.spawn_creature(&data, "slime_janitor", Job::Janitor);
    assert_eq!(
        waste_inspection_hint(&session, &data, session.building_at(farm).unwrap()),
        "Waste 2.5 · Slime Janitor cleans it"
    );
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
fn inspection_staffing_recognizes_an_engineer_at_a_mine() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 31);
    let mine = session.buildings_of("mine").next().unwrap().pos;
    session.creatures.clear();
    session.spawn_creature(&data, "engineer", Job::Engineer);
    session.creatures[0].task = Task::WorkMine(mine);

    assert!(local_mine_worker_at(&session.creatures[0], mine));
    assert!(local_mine_staffed_at(&session.creatures[0], mine));
}

#[test]
fn inspection_smelter_staffing_ignores_remote_salamanders() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 32);
    let spot = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable smelter location");
    session.buildings.push(Building::new("smelter", spot));
    session.creatures.clear();
    session.spawn_creature(&data, "salamander", Job::Smelter);
    session.creatures[0].task = Task::GoSmelt(spot);

    assert!(!local_smelter_worker_at(&session.creatures[0], spot));
    assert!(local_smelter_staffed_at(&session.creatures[0], spot));

    session.creatures[0].remote_outpost = Some(TilePos::new(4, 4));
    assert!(!local_smelter_staffed_at(&session.creatures[0], spot));
}

#[test]
fn mine_staffing_label_names_specialist_neutral_states() {
    assert_eq!(mine_staffing_label(0, 2, false), "No mine worker — stopped");
    assert_eq!(mine_staffing_label(1, 2, false), "Mine staff 1/2");
    assert_eq!(mine_staffing_label(0, 2, true), "Deposit exhausted");
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
fn outpost_cargo_only_return_label_keeps_the_remote_team_explicit() {
    assert_eq!(
        outpost_cargo_only_return_label(6),
        "Send 6 cargo · keep crew"
    );
}

#[test]
fn outpost_cargo_priority_labels_are_player_readable() {
    assert_eq!(CargoPriority::Ore.label(), "Ore first");
    assert_eq!(CargoPriority::Ingots.label(), "Ingots first");
    assert_eq!(CargoPriority::Food.label(), "Food first");
}

#[test]
fn outpost_load_hint_previews_the_selected_priority_and_reserve() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 34);
    let pos = TilePos::new(4, 4);
    let mut outpost = crate::state::outposts::Outpost::new(pos);
    outpost.cargo_priority = CargoPriority::Ingots;
    session.economy.ore_stock = 10;
    session.economy.ingots_stock = 10;
    session.economy.food = data.balance.worm_feed_reserve + 10.0;

    assert_eq!(
        outpost_load_hint(&session, &data, &outpost).as_deref(),
        Some("Load · Ore 2 · Ingots 10 · Food 0")
    );

    outpost.cargo_priority = CargoPriority::Food;
    assert_eq!(
        outpost_load_hint(&session, &data, &outpost).as_deref(),
        Some("Load · Ore 2 · Ingots 0 · Food 10")
    );
}

#[test]
fn full_outpost_load_hint_explains_the_disabled_load_action() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 35);
    let pos = TilePos::new(4, 4);
    let mut outpost = crate::state::outposts::Outpost::new(pos);
    outpost
        .cargo
        .insert(Good::Ore, data.balance.outpost_storage_cap);
    session.economy.ore_stock = 4;

    assert_eq!(
        outpost_load_hint(&session, &data, &outpost).as_deref(),
        Some("Hold full · return to shrine")
    );
}

#[test]
fn upgraded_outpost_load_hint_uses_the_expanded_hold() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 36);
    let mut outpost = crate::state::outposts::Outpost::new(TilePos::new(4, 4));
    outpost.storage_upgraded = true;
    outpost.cargo_priority = CargoPriority::Ore;
    session.economy.ore_stock = 10;
    session.economy.ingots_stock = 10;

    assert_eq!(
        outpost_load_hint(&session, &data, &outpost).as_deref(),
        Some("Load · Ore 10 · Ingots 10 · Food 0")
    );
}

#[test]
fn cargo_only_outpost_does_not_count_local_crew_as_loadable() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 41);
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.economy.food = data.balance.worm_feed_reserve;

    assert!(!outpost_has_loadable_payload(
        &session,
        &data,
        0,
        0,
        data.balance.outpost_storage_cap,
        Some(0),
    ));
}

#[test]
fn outpost_expedition_hint_names_remote_progress_and_blockers() {
    let data = GameData::load().expect("embedded game data");
    let mut outpost = crate::state::outposts::Outpost::new(TilePos::new(4, 4));
    outpost.active = true;
    outpost.crew = vec![1, 2];
    outpost.cargo.insert(Good::CookedFood, 4);
    outpost.expedition_progress = 15.0;

    assert_eq!(
        outpost_expedition_hint(&data, &outpost).as_deref(),
        Some("Expedition 50% · +6 ore / -2 food")
    );

    outpost.cargo.insert(Good::CookedFood, 1);
    assert_eq!(
        outpost_expedition_hint(&data, &outpost).as_deref(),
        Some("Expedition paused · need 1 food")
    );

    outpost.expedition_paused = true;
    assert_eq!(
        outpost_expedition_hint(&data, &outpost).as_deref(),
        Some("Expedition paused · player paused")
    );
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
