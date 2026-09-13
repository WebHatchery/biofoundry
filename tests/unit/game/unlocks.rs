use super::{session, unlock_notice};

#[test]
fn archive_wayfinder_unlock_notice_points_to_the_blacksmith() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Archive Wayfinder"),
        "Unlocked: Archive Wayfinder — tap Archive Wayfinder at the Blacksmith."
    );
}

#[test]
fn resonance_forging_unlock_notice_points_to_the_blacksmith() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Resonance Forging"),
        "Unlocked: Resonance Forging — tap Resonance Forging at the Blacksmith."
    );
}

#[test]
fn adaptive_haulers_unlock_notice_names_the_biological_payoff() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Adaptive Haulers"),
        "Unlocked: Adaptive Haulers — Beetle Haulers carry +20%."
    );
}

#[test]
fn brood_memory_unlock_notice_names_the_faster_hatch_cycle() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Brood Memory"),
        "Unlocked: Brood Memory — Breeding Pits hatch 20% sooner."
    );
}
