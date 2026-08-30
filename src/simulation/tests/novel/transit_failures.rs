//! Coverage for failed worm transit reporting and safe route recovery.

use super::active_outpost;
use crate::simulation;
use crate::simulation::outposts;

#[test]
fn failed_transit_is_reported_once_after_payload_recovery() {
    let (data, mut session, outpost_pos) = active_outpost(47);

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    assert!(outposts::start_to_shrine(&mut session, &data, outpost_pos));

    session.outposts[0].active = false;
    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.transit_failed, Some(outpost_pos));
    assert!(report.transit_completed.is_none());
    assert!(session.worm_transit.is_none());
    assert!(session.last_transit_failure.is_some());
    assert!(session.outposts[0].last_failure.is_some());

    let next_report = simulation::tick(&mut session, &data);
    assert_eq!(next_report.transit_failed, None);
}
