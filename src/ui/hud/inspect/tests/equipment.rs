use super::super::{blacksmith_equipment_label, equipment_lock_label, inspection_button_metrics};
use crate::data::GameData;

#[test]
fn blacksmith_uses_the_exact_archive_gate_for_wayfinder() {
    let data = GameData::load().expect("embedded game data");
    let wayfinder = data
        .equipment_def("archive_wayfinder")
        .expect("Archive Wayfinder data");

    assert_eq!(
        equipment_lock_label(&data, wayfinder),
        "Needs log 1 Archive page"
    );
}

#[test]
fn blacksmith_uses_the_exact_muster_gate_for_wormsong_harness() {
    let data = GameData::load().expect("embedded game data");
    let harness = data
        .equipment_def("wormsong_harness")
        .expect("Wormsong Harness data");

    assert_eq!(
        equipment_lock_label(&data, harness),
        "Needs hold 1 Worm Road Muster"
    );
}

#[test]
fn compact_blacksmith_labels_keep_recipe_and_gate_text_readable() {
    let data = GameData::load().expect("embedded game data");
    let harness = data
        .equipment_def("wormsong_harness")
        .expect("Wormsong Harness data");

    assert_eq!(
        blacksmith_equipment_label(&data, harness, true, false, 0, 0),
        "Wormsong Harness [L] · Muster"
    );
    assert_eq!(
        blacksmith_equipment_label(&data, harness, true, true, 1, 0),
        "Wormsong Harness (22)  ·1 queued"
    );
    assert_eq!(
        blacksmith_equipment_label(&data, harness, false, false, 0, 0),
        "Wormsong Harness [L] · Needs hold 1 Worm Road Muster"
    );
}

#[test]
fn compact_blacksmith_cards_keep_a_touchable_height_for_the_complete_wormsong_tier() {
    assert_eq!(
        inspection_button_metrics("blacksmith", true, 8),
        (34.0, 38.0)
    );
    assert_eq!(
        inspection_button_metrics("blacksmith", true, 10),
        (30.0, 32.0)
    );
    assert_eq!(
        inspection_button_metrics("blacksmith", true, 13),
        (30.0, 32.0)
    );
    assert_eq!(
        inspection_button_metrics("blacksmith", false, 8),
        (24.0, 26.0)
    );
}
