use super::*;

#[test]
fn full_outpost_inspection_owns_the_right_edge_below_the_top_bar() {
    let layout = InspectionLayout::new("outpost", false, 8, 26.0, 210.0);

    assert_eq!(layout.panel.x, LOGICAL_WIDTH - 262.0);
    assert_eq!(layout.panel.y, 66.0);
    assert_eq!(layout.panel.bottom(), LOGICAL_HEIGHT);
}

#[test]
fn compact_outpost_sheet_preserves_full_touch_targets() {
    let layout = InspectionLayout::new("outpost", true, 8, 34.0, 210.0);

    assert!(layout.compact_outpost);
    assert_eq!(layout.panel.x, 20.0);
    assert_eq!(layout.panel.w, LOGICAL_WIDTH - 40.0);
    assert_eq!(layout.panel.bottom(), LOGICAL_HEIGHT);
    assert_eq!(layout.outpost_button_height, 72.0);
    assert_eq!(layout.outpost_button_step, 72.0);
}
