use super::*;

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
    session.worm_fed = 110.0;
    session.worm_ingots_fed = 10;

    let body = worm_completion_body(&session);

    assert!(body.contains("Fed on 110 food and 10 ingots"));
    assert!(!body.contains("110 offerings"));
}
