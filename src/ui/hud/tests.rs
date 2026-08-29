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
    assert!(!body.contains("still needs a Guard"));
    assert_eq!(
        warren_victory_continue_label(&session),
        "Continue to Factory"
    );
}
