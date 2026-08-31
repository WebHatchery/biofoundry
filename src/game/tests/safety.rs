use super::super::{format_expedition_completion, progression_reaches_safe_beat};
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
