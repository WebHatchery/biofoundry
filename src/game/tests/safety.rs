use super::super::{
    auto_load_notice, auto_resupply_notice, auto_return_notice, format_expedition_completion,
    progression_reaches_safe_beat,
};
use crate::simulation::TickReport;
use crate::state::outposts::ExpeditionCompletion;
use macroquad_toolkit::grid::TilePos;

#[test]
fn relay_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_relay_awarded: true,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn convoy_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_convoy_awarded: 1,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn muster_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_muster_awarded: 1,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn concord_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_concord_awarded: true,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn circuit_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_circuit_awarded: true,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn encore_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_encore_awarded: 1,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn chorus_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_chorus_awarded: true,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn auto_load_departure_is_a_safe_autosave_beat() {
    let report = TickReport {
        auto_load_started: Some(TilePos::new(4, 4)),
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}

#[test]
fn expedition_notice_names_signal_cache_ingots() {
    assert_eq!(
        format_expedition_completion(ExpeditionCompletion {
            outpost: TilePos::new(4, 4),
            ore: 12,
            ingots: 1,
            food_spent: 2,
        }),
        "Outpost haul · +12 ore · +1 ingot / -2 food."
    );
}

#[test]
fn automatic_route_notices_name_their_payload_policies() {
    assert_eq!(
        auto_return_notice(),
        "Outpost hold full — cargo returning while scouts remain remote."
    );
    assert_eq!(
        auto_resupply_notice(),
        "Outpost scouts need food — a food-only resupply is on its way."
    );
    assert_eq!(
        auto_load_notice(),
        "Auto-load departed — cargo and available scouts are on the worm road."
    );
}
