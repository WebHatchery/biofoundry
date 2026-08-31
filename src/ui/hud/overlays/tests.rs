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
fn saved_failure_offers_load_or_direct_restart() {
    let ((load_action, load_label), (restart_action, restart_label)) = colony_failure_actions(true);

    assert_eq!(load_action, UiAction::Load);
    assert_eq!(load_label, "Load Last Safe");
    assert_eq!(restart_action, UiAction::StartWarren);
    assert_eq!(restart_label, "Start New Warren");
    assert!(colony_failure_body(ColonyFailure::Silent, true).contains("tap Start New Warren"));
    assert!(
        colony_failure_body(ColonyFailure::GuardHandoff, true).contains("replace this checkpoint")
    );
}

#[test]
fn failure_without_a_save_keeps_new_warren_and_menu_choices() {
    let ((restart_action, restart_label), (menu_action, menu_label)) =
        colony_failure_actions(false);

    assert_eq!(restart_action, UiAction::StartWarren);
    assert_eq!(restart_label, "Start New Warren");
    assert_eq!(menu_action, UiAction::BackToMenu);
    assert_eq!(menu_label, "Return to Menu");
}

#[test]
fn field_guide_explains_breeding_specialists() {
    assert!(INSPECT_HELP_BODY.contains("Breeding Pit"));
    assert!(INSPECT_HELP_BODY.contains("after onboarding"));
    assert!(INSPECT_HELP_BODY.contains("benefits"));
    assert!(INSPECT_HELP_BODY.contains("Study Pen"));
}

#[test]
fn field_guide_switches_to_compact_post_campaign_guidance() {
    let (title, body) = field_guide_inspect_content(true);
    assert_eq!(title, "Endless routes");
    assert!(body.contains("Load order"));
    assert!(body.contains("Pause scouting"));
    assert!(body.contains("remote crew"));

    let objective = field_guide_objective_content(true);
    assert!(objective.contains("Routes"));
    assert!(objective.contains("upgrades"));
    assert!(objective.contains("shared-Worm"));
}

#[test]
fn field_guide_keeps_core_guidance_before_the_worm_wakes() {
    let (title, body) = field_guide_inspect_content(false);
    assert_eq!(title, "Inspect & craft");
    assert!(body.contains("Blacksmith recipe"));
    assert!(!body.contains("awakened Outpost"));

    let objective = field_guide_objective_content(false);
    assert!(objective.contains("Jobs"));
    assert!(objective.contains("Build & Dig"));
    assert!(!objective.contains("shared-Worm"));
}

#[test]
fn field_guide_exposes_the_recent_events_view() {
    assert!(FIELD_GUIDE_INTRO.contains("Recent events"));
}

#[test]
fn active_load_confirmation_names_both_visible_choices_and_the_risk() {
    assert!(LOAD_CONFIRMATION_TEXT.contains("Load Last Save"));
    assert!(LOAD_CONFIRMATION_TEXT.contains("Keep Current"));
    assert!(LOAD_CONFIRMATION_TEXT.contains("discarded"));
}

#[test]
fn recent_event_pages_keep_newest_entries_on_the_first_page() {
    assert_eq!(event_log_page_count(0), 1);
    assert_eq!(event_log_page_count(10), 1);
    assert_eq!(event_log_page_count(11), 2);
    assert_eq!(event_log_page_bounds(25, 0), (15, 25));
    assert_eq!(event_log_page_bounds(25, 1), (5, 15));
    assert_eq!(event_log_page_bounds(25, 2), (0, 5));
    assert_eq!(event_log_page_bounds(5, 9), (0, 0));
}
