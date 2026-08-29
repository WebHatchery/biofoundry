use super::*;

#[test]
fn awakening_effect_fades_after_its_short_landing_window() {
    assert_eq!(awakening_progress(0), 1.0);
    assert!(awakening_progress(AWAKENING_EFFECT_TICKS / 2) > 0.0);
    assert_eq!(awakening_progress(AWAKENING_EFFECT_TICKS), 0.0);
    assert_eq!(awakening_progress(AWAKENING_EFFECT_TICKS + 20), 0.0);
}
