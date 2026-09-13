use super::{
    blacksmith_equipment_label, blacksmith_recipe_rows, equipment_lock_label,
    inspection_button_metrics,
};
use biofoundry::data::GameData;

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
fn compact_blacksmith_grid_keeps_recipe_buttons_tall_enough_to_read() {
    assert_eq!(
        inspection_button_metrics("blacksmith", true, 8),
        (46.0, 48.0)
    );
    assert_eq!(
        inspection_button_metrics("blacksmith", true, 10),
        (46.0, 48.0)
    );
    assert_eq!(
        inspection_button_metrics("blacksmith", true, 13),
        (46.0, 48.0)
    );
    assert_eq!(
        inspection_button_metrics("blacksmith", false, 8),
        (24.0, 26.0)
    );
}

#[test]
fn compact_shrine_pause_control_keeps_the_critical_handoff_touchable() {
    assert_eq!(
        inspection_button_metrics("worm_shrine", true, 0),
        (72.0, 76.0)
    );
}

#[test]
fn compact_blacksmith_grid_rounds_odd_recipe_catalogues_up_to_a_full_row() {
    assert_eq!(blacksmith_recipe_rows(0), 0);
    assert_eq!(blacksmith_recipe_rows(1), 1);
    assert_eq!(blacksmith_recipe_rows(12), 6);
    assert_eq!(blacksmith_recipe_rows(13), 7);
}
