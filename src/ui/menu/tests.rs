use super::*;

#[test]
fn new_warren_confirmation_names_both_visible_choices() {
    assert!(NEW_WARREN_CONFIRMATION_TEXT.contains("Start New Warren"));
    assert!(NEW_WARREN_CONFIRMATION_TEXT.contains("Keep Save"));
    assert!(!NEW_WARREN_CONFIRMATION_TEXT.contains("Continue"));
}
