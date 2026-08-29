use super::UiMode;

#[test]
fn successful_build_placement_returns_to_inspect() {
    assert_eq!(
        UiMode::Build("farm".to_owned()).after_successful_placement(),
        UiMode::Inspect
    );
}

#[test]
fn placement_completion_does_not_change_dig_mode() {
    assert_eq!(UiMode::Dig.after_successful_placement(), UiMode::Dig);
}
