use super::super::{equipment_lock_label, inspection_button_metrics};
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
fn compact_blacksmith_cards_shrink_for_the_complete_wormsong_tier() {
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
        (26.0, 28.0)
    );
    assert_eq!(
        inspection_button_metrics("blacksmith", false, 8),
        (24.0, 26.0)
    );
}
