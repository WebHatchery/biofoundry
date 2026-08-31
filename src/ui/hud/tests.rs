use super::*;
use crate::state::creatures::Job;

#[test]
fn top_bar_claims_scaled_button_margin_before_world_input() {
    let bar = Rect::new(12.0, 12.0, 1256.0, 48.0);
    let input = top_bar_input_rect(bar, 0.8);

    assert!(input.contains_point(vec2(bar.x + 10.0, bar.y - 10.0)));
    assert!(!input.contains_point(vec2(bar.x, 0.0)));
}

#[test]
fn top_bar_does_not_grow_on_a_full_size_surface() {
    let bar = Rect::new(12.0, 12.0, 1256.0, 48.0);
    let input = top_bar_input_rect(bar, 1.0);

    assert!((input.x - 6.0).abs() < 0.01);
    assert!((input.y - 6.0).abs() < 0.01);
    assert!((input.right() - 1274.0).abs() < 0.01);
}

#[test]
fn panels_claim_scaled_button_margins_before_world_input() {
    let panel = Rect::new(938.0, 72.0, 330.0, 128.0);
    let input = panel_input_rect(panel, 0.8);

    assert!(input.contains_point(vec2(panel.x + 100.0, panel.bottom() + 10.0)));
    assert!(!input.contains_point(vec2(panel.x + 100.0, panel.bottom() + 18.0)));
}

#[test]
fn inspection_card_moves_below_the_tutorial_card() {
    let tutorial = Rect::new(938.0, 72.0, 330.0, 168.0);

    assert_eq!(inspect_panel_top(Some(tutorial), false), 250.0);
    assert_eq!(inspect_panel_top(None, false), 210.0);
}

#[test]
fn compact_inspection_card_uses_the_open_right_side_of_the_hud() {
    assert_eq!(inspect_panel_top(None, true), 60.0);
    assert_eq!(
        inspect_panel_top(Some(Rect::new(938.0, 72.0, 330.0, 168.0)), true),
        250.0
    );
}

#[test]
fn touch_release_position_can_claim_hud_when_mouse_is_elsewhere() {
    let ui = VirtualUi::from_screen_size(1280.0, 720.0, 1440.0, 900.0);
    let touch_screen = ui.ui_to_screen(vec2(50.0, 30.0));
    let touch_ui = interaction_point(&ui, vec2(600.0, 500.0), Some(touch_screen));
    let top_bar = Rect::new(12.0, 12.0, 1256.0, 48.0);

    assert!(top_bar_input_rect(top_bar, ui.scale).contains_point(touch_ui));
    assert!(!top_bar_input_rect(top_bar, ui.scale).contains_point(vec2(600.0, 500.0)));
}

#[test]
fn active_load_confirmation_owns_world_input() {
    assert!(modal_owns_world_input(
        false, false, false, false, false, false, true
    ));
    assert!(!modal_owns_world_input(
        false, false, false, false, false, false, false
    ));
}

#[test]
fn worm_completion_summary_names_the_resources_consumed() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 42);
    session.tick = 1234;
    session.worm_fed = data.balance.worm_awaken_at;
    session.worm_ingots_fed = data.balance.worm_awaken_ingots;

    let body = worm_completion_body(&session);

    assert!(body.contains(&format!(
        "Fed on {:.0} food and {} ingots",
        data.balance.worm_awaken_at, data.balance.worm_awaken_ingots
    )));
    assert!(body.contains("Tap Continue in Endless"));
    assert!(body.contains("tap Return to Menu"));
    assert!(!body.contains("offerings"));
}

#[test]
fn warren_victory_report_keeps_the_guard_handoff_explicit() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 42);
    session.won = true;

    let body = warren_victory_body(&session, &data);
    assert!(body.contains("Onboarding still needs a Guard"));
    assert!(body.contains("tap − beside Miner, then + beside Guard"));
    assert_eq!(warren_victory_continue_label(&session), "Return to Warren");

    session.creatures[0].job = Job::Guard;
    let body = warren_victory_body(&session, &data);
    assert!(body.contains("Onboarding is complete"));
    assert!(body.contains("Tap Continue to Factory"));
    assert!(body.contains("tap Blacksmith in Build & Dig"));
    assert!(body.contains("tap open floor"));
    assert!(!body.contains("still needs a Guard"));
    assert_eq!(
        warren_victory_continue_label(&session),
        "Continue to Factory"
    );
}

#[test]
fn warren_victory_report_does_not_promise_a_specialist_assignment() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 42);
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    let body = warren_victory_body(&session, &data);

    assert!(body.contains("free a worker, then tap + beside Guard in Jobs"));
    assert!(!body.contains("After closing this report, tap + beside Guard in Jobs."));
}

#[test]
fn factory_completion_body_names_the_visible_worm_handoff_controls() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, 42);
    let body = factory_completion_body(&session);

    assert!(body.contains("Tap Continue to Worm"));
    assert!(body.contains("tap Shrine in Build & Dig"));
    assert!(body.contains("tap open floor"));
    assert!(body.contains("tap Return to Menu"));
}

#[test]
fn colony_failure_detects_a_stuck_security_handoff() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 42);
    session.won = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert_eq!(
        colony_failure_reason(&session, &data),
        Some(ColonyFailure::GuardHandoff)
    );
}

#[test]
fn colony_failure_does_not_interrupt_an_awakened_specialist_warren() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, 42);
    session.won = true;
    session.worm_awake = true;
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert_eq!(colony_failure_reason(&session, &data), None);
}
