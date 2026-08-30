use super::*;

#[test]
fn compact_raid_hint_names_the_controls_for_a_fresh_warren() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(session.job_count(Job::Idle), 0);
    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "tap − Miner, then + Guard"
    );
}

#[test]
fn compact_raid_hint_shortens_when_a_worker_is_idle_or_guarded() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);

    session.creatures[0].job = Job::Idle;
    assert_eq!(compact_raid_defense_hint(&session, &data), "tap + Guard");

    session.creatures[1].job = Job::Guard;
    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "guards on watch"
    );
}

#[test]
fn compact_raid_hint_uses_an_available_specialist_job() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);
    session.spawn_creature(&data, "goblin", Job::Smith);

    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "tap − Smith, then + Guard"
    );
}

#[test]
fn secure_tutorial_names_the_available_guard_action() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.tutorial_step = 3;
    let step = crate::tutorial::current_step(&session, &data).expect("secure tutorial step");

    let body = tutorial_body(step, &session, &data);

    assert!(body.contains("tap − beside Miner, then + beside Guard in Jobs"));
    assert!(!body.contains("if Idle is 0"));
}

#[test]
fn food_tutorial_names_the_available_carrier_action() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.tutorial_step = 1;
    let step = crate::tutorial::current_step(&session, &data).expect("food tutorial step");

    let body = tutorial_body(step, &session, &data);

    assert!(body.contains("tap − beside Miner, then + beside Carrier in Jobs"));
    assert!(!body.contains("If Idle is 0"));
}

#[test]
fn factory_tutorial_names_the_available_smith_action() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.tutorial_step = 2;
    let step = crate::tutorial::current_step(&session, &data).expect("factory tutorial step");

    let body = tutorial_body(step, &session, &data);

    assert!(body.contains("tap − beside Miner, then + beside Smith in Jobs"));
    assert!(!body.contains("If Idle is 0"));
}

#[test]
fn secure_tutorial_keeps_specialist_recovery_honest() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.tutorial_step = 3;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);
    let step = crate::tutorial::current_step(&session, &data).expect("secure tutorial step");

    let body = tutorial_body(step, &session, &data);

    assert!(body.contains("free a worker, then tap + beside Guard in Jobs"));
}

#[test]
fn compact_food_hint_fits_the_top_bar() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(
        compact_food_recovery_hint(&session, &data),
        "tap − Miner, then + Carrier"
    );
}

#[test]
fn compact_raid_hint_stays_short_when_a_worker_is_free() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures[0].job = Job::Idle;

    assert_eq!(compact_raid_defense_hint(&session, &data), "tap + Guard");
}

#[test]
fn condensed_alert_hints_fit_a_narrow_top_bar() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(
        condensed_raid_defense_hint(&session, &data),
        "tap −Miner→+Guard"
    );
    assert_eq!(
        condensed_food_recovery_hint(&session, &data),
        "tap −Miner→+Carrier"
    );
}

#[test]
fn compact_top_bar_only_applies_below_the_design_scale() {
    assert!(compact_top_bar(0.625));
    assert!(!compact_top_bar(0.9));
    assert!(!compact_top_bar(1.0));
    assert!(!compact_top_bar(f32::NAN));
}

#[test]
fn hints_do_not_promise_reassignment_of_a_specialist() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert_eq!(reassignable_job_count(&session, &data, Job::Idle), 0);
    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "free a worker, then + Guard"
    );
    assert_eq!(
        compact_food_recovery_hint(&session, &data),
        "free a worker, then + Carrier"
    );
}

#[test]
fn advanced_systems_wait_for_the_security_handoff() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.won = true;

    assert!(!advanced_systems_unlocked(&session));

    session.creatures[0].job = Job::Guard;
    assert!(advanced_systems_unlocked(&session));
}

#[test]
fn awakened_warrens_keep_advanced_systems_visible() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.worm_awake = true;

    assert!(advanced_systems_unlocked(&session));
}

#[test]
fn optional_specialist_buttons_explain_when_the_unique_post_is_filled() {
    assert_eq!(
        optional_specialist_label("slime_janitor", false),
        "Slime · clean"
    );
    assert_eq!(
        optional_specialist_label("slime_janitor", true),
        "Slime · posted"
    );
    assert_eq!(
        optional_specialist_label("bat_courier", false),
        "Bat · 8 cargo"
    );
    assert_eq!(
        optional_specialist_label("bat_courier", true),
        "Bat · posted"
    );
}

#[test]
fn optional_support_buttons_name_their_practical_roles() {
    assert_eq!(optional_support_label("beetle", 25), "Beetle haul (25)");
    assert_eq!(
        optional_support_label("salamander", 20),
        "Salam. forge (20)"
    );
}

#[test]
fn jobs_panel_capacity_uses_local_workers_and_floor_space() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(
        workforce_capacity_label(&session, &data),
        format!(
            "Idle 0 · Local {}/{}",
            session.local_creature_count(),
            session.local_warren_capacity(&data)
        )
    );
}

#[test]
fn engineer_summary_distinguishes_local_and_posted_specialists() {
    assert_eq!(engineer_status_label(0, 0), "Engineer 0 · no local bonus");
    assert_eq!(engineer_status_label(0, 1), "Engineer 0 local · 1 posted");
    assert_eq!(engineer_status_label(1, 1), "Engineer 1 local · Mine +25%");
}

#[test]
fn active_tool_marker_uses_a_font_safe_glyph() {
    assert_eq!(active_tool_marker(false), "");
    assert_eq!(active_tool_marker(true), "> ");
    assert!(!active_tool_marker(true).contains('▶'));
}

#[test]
fn locked_tool_marker_uses_font_safe_ascii() {
    assert_eq!(LOCKED_TOOL_MARKER, "[L]");
    assert!(!LOCKED_TOOL_MARKER.contains('🔒'));
}

#[test]
fn construction_progress_label_explains_remaining_ore() {
    assert_eq!(
        construction_progress_label(1, 10),
        "Build 1 site · 10 ore left"
    );
    assert_eq!(
        construction_progress_label(2, 18),
        "Build 2 sites · 18 ore left"
    );
}
