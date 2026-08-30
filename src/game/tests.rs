use super::persistence::validate_loaded_session;
use super::*;
use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::outposts::{ExpeditionCompletion, TransitCompletion, TransitDirection};
use crate::state::structures::{BuildSite, Building};
use crate::state::GameSession;
use macroquad_toolkit::notifications::{
    LoggedNotification, NotificationManager, NotificationType, MAX_HISTORY,
};

fn session() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, 42);
    (data, session)
}

#[test]
fn old_tutorial_index_does_not_skip_the_new_factory_lesson() {
    let (data, mut session) = session();
    session.tutorial_step = 4; // old Blacksmith/famine territory
    session.tutorial_built = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, 2);
}

#[test]
fn tutorial_migration_keeps_a_pending_farm_lesson_visible() {
    let (data, mut session) = session();
    session.tutorial_step = 1;
    session.tutorial_built = true;
    session.build_sites.push(BuildSite {
        kind: "farm".to_owned(),
        pos: session.spawn_tile(),
        ore_needed: data.buildings.get("farm").unwrap().cost_ore,
        ore_delivered: 0,
    });

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, 1);
}

#[test]
fn tutorial_migration_uses_completed_campaign_facts() {
    let (data, mut session) = session();
    session.tutorial_step = 6;
    session.tutorial_built = true;
    session.won = true;
    session.worm_awake = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, data.tutorial.len());
}

#[test]
fn tutorial_migration_keeps_unwitnessed_guard_lesson_visible() {
    let (data, mut session) = session();
    session.tutorial_built = true;
    session
        .economy
        .gear_stock
        .insert("iron_pickaxe".to_owned(), 1);
    session.won = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());
    assert_eq!(session.tutorial_step, 3);

    session.creatures[0].job = Job::Guard;
    migrate_tutorial_progress(&mut session, data.tutorial.len());
    assert_eq!(session.tutorial_step, 4);
}

#[test]
fn non_viable_save_notice_protects_the_last_checkpoint() {
    let (data, mut session) = session();
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert_eq!(
        non_viable_save_notice(&session, &data),
        Some("This warren cannot staff the Guard post. Load a safe save or start a new warren instead.")
    );
}

#[test]
fn missing_primary_save_uses_a_surviving_backup() {
    assert!(should_restore_missing_primary(false, true));
    assert!(!should_restore_missing_primary(false, false));
    assert!(!should_restore_missing_primary(true, true));
}

#[test]
fn startup_continue_stays_available_for_a_backup_only_save() {
    assert!(save_slot_available(true, false));
    assert!(save_slot_available(false, true));
    assert!(!save_slot_available(false, false));
}

#[test]
fn missing_save_clears_a_stale_continue_indicator() {
    assert!(no_saved_slot_available(false, false));
    assert!(!no_saved_slot_available(false, true));
    assert!(!no_saved_slot_available(true, false));
    assert_eq!(
        missing_save_notice("slot not found"),
        "Load failed — no saved warren is available: slot not found. Use New Warren to begin again."
    );
}

#[test]
fn save_failure_notice_keeps_recovery_state_explicit() {
    assert_eq!(
        save_failure_notice(false, true, "storage full"),
        "Save failed — previous save remains available: storage full"
    );
    assert_eq!(
        save_failure_notice(false, false, "storage full"),
        "Save failed — no new save was written: storage full"
    );
    assert_eq!(
        save_failure_notice(true, true, "storage full"),
        "Autosave failed — previous save remains available; use Save to retry: storage full"
    );
    assert_eq!(
        save_failure_notice(true, false, "storage full"),
        "Autosave failed — use Save to create a checkpoint: storage full"
    );
}

#[test]
fn viable_save_notice_stays_empty_for_a_recoverable_warren() {
    let (data, mut session) = session();
    session.won = true;

    assert_eq!(non_viable_save_notice(&session, &data), None);
}

#[test]
fn loaded_session_validation_accepts_a_fresh_warren() {
    let (data, session) = session();

    validate_loaded_session(&session, &data).expect("fresh session should be loadable");
}

#[test]
fn current_version_save_payloads_still_receive_integrity_validation() {
    let (data, mut session) = session();
    session.buildings[0].kind = "unknown_building".to_owned();

    // The toolkit's current-version fast path deserializes the wrapper
    // directly, so this mirrors the payload that the game shell receives
    // before its post-load validation boundary.
    let encoded = serde_json::to_value(&session).expect("session serializes");
    let restored: GameSession = serde_json::from_value(encoded).expect("payload deserializes");
    let error = super::persistence::validate_loaded_session_boundary(restored, &data)
        .expect_err("current-version payload must not bypass validation");

    assert!(error.contains("unknown building id"));
}

#[test]
fn notification_history_rehydrates_without_replaying_old_toasts() {
    let mut manager = NotificationManager::new();
    let history = vec![
        LoggedNotification {
            message: "The warren is secure.".to_owned(),
            notification_type: NotificationType::Success,
        },
        LoggedNotification {
            message: "Food is low.".to_owned(),
            notification_type: NotificationType::Warning,
        },
    ];

    super::persistence::restore_notification_history(&mut manager, &history);

    assert_eq!(manager.history(), history.as_slice());
    assert!(manager.is_empty());
}

#[test]
fn notification_history_rehydration_keeps_only_the_bounded_tail() {
    let history = (0..MAX_HISTORY + 2)
        .map(|index| LoggedNotification {
            message: format!("Event {index}"),
            notification_type: NotificationType::Info,
        })
        .collect::<Vec<_>>();
    let mut manager = NotificationManager::new();

    super::persistence::restore_notification_history(&mut manager, &history);

    assert_eq!(manager.history().len(), MAX_HISTORY);
    assert_eq!(manager.history().first().unwrap().message, "Event 2");
    assert_eq!(manager.history().last().unwrap().message, "Event 201");
    assert!(manager.is_empty());
}

#[test]
fn loaded_session_validation_accepts_a_simulated_warren() {
    let (data, mut session) = session();
    for _ in 0..600 {
        crate::simulation::tick(&mut session, &data);
    }

    validate_loaded_session(&session, &data).expect("simulated session should remain loadable");
}

#[test]
fn loaded_session_validation_accepts_remote_transit_states() {
    let (data, mut session) = session();
    let positions: Vec<_> = session
        .world
        .tiles
        .iter_with_pos()
        .filter(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .take(2)
        .collect();
    assert_eq!(positions.len(), 2, "route test needs two free floor tiles");
    session
        .buildings
        .push(Building::new("worm_shrine", positions[0]));
    session
        .buildings
        .push(Building::new("outpost", positions[1]));
    session.ensure_outpost(positions[1]);
    session.worm_awake = true;
    session.outposts[0].active = true;
    session.economy.ore_stock = 2;

    assert!(crate::simulation::outposts::start_to_outpost(
        &mut session,
        &data,
        positions[1]
    ));
    validate_loaded_session(&session, &data).expect("in-flight route should be loadable");

    crate::simulation::outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    validate_loaded_session(&session, &data).expect("arrived remote crew should be loadable");
}

#[test]
fn loaded_session_validation_rejects_broken_grid_storage() {
    let (data, mut session) = session();
    session.world.tiles.width -= 1;

    let error = validate_loaded_session(&session, &data).expect_err("broken grid must be rejected");

    assert!(error.contains("world grid dimensions"));
}

#[test]
fn loaded_session_validation_rejects_overlapping_known_buildings() {
    let (data, mut session) = session();
    let existing = session.buildings[0].clone();
    session
        .buildings
        .push(Building::new(&existing.kind, existing.pos));

    let error = validate_loaded_session(&session, &data)
        .expect_err("overlapping buildings must be rejected");

    assert!(error.contains("multiple buildings"));
}

#[test]
fn loaded_session_validation_rejects_unknown_content_ids() {
    let (data, mut session) = session();
    session.buildings[0].kind = "unknown_building".to_owned();

    let error =
        validate_loaded_session(&session, &data).expect_err("unknown content must be rejected");

    assert!(error.contains("unknown building id"));
}

#[test]
fn goal_modal_holds_the_simulation_until_the_report_is_dismissed() {
    let (data, mut session) = session();
    session.won = true;

    assert!(simulation_blocked_by_modal(&session, &data, false, false));

    session.victory_shown = true;
    session.factory_complete = true;
    assert!(simulation_blocked_by_modal(&session, &data, false, false));

    session.factory_shown = true;
    session.worm_awake = true;
    assert!(simulation_blocked_by_modal(&session, &data, false, false));

    session.worm_shown = true;
    assert!(!simulation_blocked_by_modal(&session, &data, false, false));
}

#[test]
fn field_guide_pauses_a_viable_warren_while_open() {
    let (data, session) = session();

    assert!(simulation_blocked_by_modal(&session, &data, true, false));
    assert!(!simulation_blocked_by_modal(&session, &data, false, false));
}

#[test]
fn route_ledger_pauses_an_awakened_warren_while_open() {
    let (data, mut session) = session();
    session.worm_awake = true;
    session.worm_shown = true;

    assert!(simulation_blocked_by_modal(&session, &data, false, true));
    assert!(!simulation_blocked_by_modal(&session, &data, false, false));
}

#[test]
fn route_ledger_focus_uses_the_center_of_the_selected_tile() {
    assert_eq!(
        tile_world_center(TilePos::new(4, 7), 32.0),
        Some(vec2(144.0, 240.0))
    );
    assert_eq!(tile_world_center(TilePos::new(4, 7), 0.0), None);
    assert_eq!(tile_world_center(TilePos::new(4, 7), f32::NAN), None);
}

#[test]
fn progression_events_are_safe_autosave_beats() {
    let mut report = simulation::TickReport::default();
    assert!(!progression_reaches_safe_beat(&report));

    report.wild.captured = 1;
    assert!(progression_reaches_safe_beat(&report));

    report.wild.captured = 0;
    report.wild.unlocked.push("new_route".to_owned());
    assert!(progression_reaches_safe_beat(&report));

    report.wild.unlocked.clear();
    report.wild.raid_survived = true;
    assert!(progression_reaches_safe_beat(&report));

    report.wild.raid_survived = false;
    report.wild.bred_beetle = true;
    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn non_viable_recovery_stops_the_remaining_specialists() {
    let (data, mut session) = session();
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert!(simulation_blocked_by_modal(&session, &data, false, false));
}

#[test]
fn secure_threshold_notice_waits_for_the_guard_handoff() {
    let (_data, mut session) = session();

    assert_eq!(
        warren_secured_notice(&session),
        "The reserve gate is secure — assign a Guard to finish onboarding."
    );

    session.creatures[0].job = Job::Guard;
    assert_eq!(
        warren_secured_notice(&session),
        "The warren is secure — onboarding complete."
    );
}

#[test]
fn transit_completion_notice_names_a_crew_only_arrival() {
    let completion = TransitCompletion {
        direction: TransitDirection::ToOutpost,
        cargo_units: 0,
        passenger_count: 2,
    };

    assert_eq!(
        transit_completion_notice(completion),
        "The worm reaches the outpost — crew delivered."
    );
}

#[test]
fn expedition_completion_notice_names_the_remote_yield_and_upkeep() {
    assert_eq!(
        format_expedition_completion(ExpeditionCompletion {
            outpost: macroquad_toolkit::grid::TilePos::new(4, 4),
            ore: 6,
            food_spent: 2,
        }),
        "Outpost haul · +6 ore / -2 food."
    );
}

#[test]
fn auto_return_notice_names_the_remote_team_outcome() {
    assert_eq!(
        auto_return_notice(),
        "Outpost hold full — cargo returning while scouts remain remote."
    );
}

#[test]
fn auto_resupply_notice_names_the_food_only_transit() {
    assert_eq!(
        auto_resupply_notice(),
        "Outpost scouts need food — a food-only resupply is on its way."
    );
}

#[test]
fn transit_failure_notice_points_to_route_recovery() {
    assert_eq!(
        transit_failure_notice(),
        "The worm route failed — tap the outpost, then reactivate the route before trying again."
    );
}

#[test]
fn transit_completion_notice_names_mixed_payloads() {
    let completion = TransitCompletion {
        direction: TransitDirection::ToShrine,
        cargo_units: 3,
        passenger_count: 1,
    };

    assert_eq!(
        transit_completion_notice(completion),
        "The worm returns to the shrine — cargo and crew delivered."
    );
}

#[test]
fn unlock_notice_teaches_how_to_use_new_content() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Beetle Breeding Pit"),
        "Unlocked: Beetle Breeding Pit — available in Build & Dig after onboarding."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Hobgoblin Brood"),
        "Unlocked: Hobgoblin Brood — breed at the Breeding Pit after onboarding."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Slime Janitor"),
        "Unlocked: Slime Janitor — recruit from Jobs after onboarding."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Bat Courier"),
        "Unlocked: Bat Courier — recruit from Jobs after onboarding."
    );
}

#[test]
fn unlock_notice_switches_to_visible_optional_controls_after_onboarding() {
    let (data, mut session) = session();
    session.won = true;
    session.creatures[0].job = Job::Guard;

    assert_eq!(
        unlock_notice(&data, &session, "Beetle Breeding Pit"),
        "Unlocked: Beetle Breeding Pit — build Breeding Pit from Build & Dig."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Slime Janitor"),
        "Unlocked: Slime Janitor — recruit from Jobs."
    );
}

#[test]
fn unlock_notice_teaches_passive_benefits() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Hardened Guards"),
        "Unlocked: Hardened Guards — Guards deal +50% damage."
    );
    assert_eq!(
        unlock_notice(&data, &session, "Preservation Techniques"),
        "Unlocked: Preservation Techniques — Farms hold +50% food."
    );
}

#[test]
fn unlock_notice_keeps_unknown_names_safe() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Future discovery"),
        "Unlocked: Future discovery"
    );
}
