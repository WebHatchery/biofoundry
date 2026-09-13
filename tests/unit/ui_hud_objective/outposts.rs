//! Objective coverage for the first awakened Outpost handoffs.

use super::*;
use biofoundry::state::structures::Building;

#[test]
fn completed_objective_names_activation_for_an_empty_inactive_outpost() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.unlocked.insert("worm_transit".to_owned());
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session.buildings.push(Building::new("outpost", pos));
    session.ensure_outpost(pos);

    let objective = CampaignObjective::current(&session, &data);

    assert_eq!(
        objective.next,
        "Next: tap the Worm Outpost, then tap Activate route."
    );
}
