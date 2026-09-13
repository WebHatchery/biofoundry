use biofoundry::data::GameData;

#[test]
fn new_warren_confirmation_names_both_visible_choices() {
    let data = GameData::load().expect("embedded game data");
    let text = data.message("menu.new_warren_confirmation");
    assert!(text.contains("Start New Warren"));
    assert!(text.contains("Keep Save"));
    assert!(!text.contains("Continue"));
}
