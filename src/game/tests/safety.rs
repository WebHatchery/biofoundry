use super::super::progression_reaches_safe_beat;
use crate::simulation::TickReport;

#[test]
fn relay_award_is_a_safe_autosave_beat() {
    let report = TickReport {
        outpost_relay_awarded: true,
        ..TickReport::default()
    };

    assert!(progression_reaches_safe_beat(&report));
}
