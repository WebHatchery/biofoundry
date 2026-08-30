use super::*;
use crate::data::GameData;
use crate::state::GameSession;

#[test]
fn recovery_guide_names_enabled_controls_for_a_fresh_warren() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    let body = recovery_guide_body(&session, &data);

    assert!(body.contains("tap − Miner, then + Carrier"));
    assert!(body.contains("tap − Miner, then + Guard"));
    assert!(!body.contains("tap + beside Carrier"));
    assert!(!body.contains("tap + beside Guard."));
}

#[test]
fn recovery_guide_keeps_specialists_out_of_disabled_recovery_steps() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", crate::state::creatures::Job::Idle);

    let body = recovery_guide_body(&session, &data);

    assert!(body.contains("free a worker, then + Carrier"));
    assert!(body.contains("free a worker, then + Guard"));
}

#[test]
fn field_guide_explains_breeding_specialists() {
    assert!(INSPECT_HELP_BODY.contains("Breeding Pit"));
    assert!(INSPECT_HELP_BODY.contains("after onboarding"));
    assert!(INSPECT_HELP_BODY.contains("benefits"));
    assert!(INSPECT_HELP_BODY.contains("Load order"));
}

#[test]
fn field_guide_points_to_post_campaign_cargo_runs() {
    assert!(OBJECTIVE_HELP_BODY.contains("worm wakes"));
    assert!(OBJECTIVE_HELP_BODY.contains("tap Routes"));
    assert!(OBJECTIVE_HELP_BODY.contains("Outpost"));
    assert!(INSPECT_HELP_BODY.contains("scout for ore"));
    assert!(INSPECT_HELP_BODY.contains("Auto-resupply · Food only"));
}

#[test]
fn field_guide_exposes_the_recent_events_view() {
    assert!(FIELD_GUIDE_INTRO.contains("Recent events"));
}
