use super::{session, unlock_notice};

#[test]
fn archive_wayfinder_unlock_notice_points_to_the_blacksmith() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Archive Wayfinder"),
        "Unlocked: Archive Wayfinder — queue it at the Blacksmith."
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
