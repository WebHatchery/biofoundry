use super::{session, unlock_notice};

#[test]
fn archive_wayfinder_unlock_notice_points_to_the_blacksmith() {
    let (data, session) = session();

    assert_eq!(
        unlock_notice(&data, &session, "Archive Wayfinder"),
        "Unlocked: Archive Wayfinder — queue it at the Blacksmith."
    );
}
