use super::{session, validate_loaded_session};
use biofoundry::state::outposts::{TransitDirection, WormTransit};
use biofoundry::state::structures::Building;

#[test]
fn loaded_session_validation_rejects_in_flight_cargo_that_overfills_stored_hold() {
    let (data, mut session) = session();
    let positions: Vec<_> = session
        .world
        .tiles
        .iter_with_pos()
        .filter(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .take(2)
        .collect();
    assert_eq!(
        positions.len(),
        2,
        "transit test needs two free floor tiles"
    );
    session
        .buildings
        .push(Building::new("worm_shrine", positions[0]));
    session
        .buildings
        .push(Building::new("outpost", positions[1]));
    session.ensure_outpost(positions[1]);
    session.worm_awake = true;
    session.outposts[0].cargo.insert(
        biofoundry::state::creatures::Good::Ore,
        data.balance.outpost_storage_cap - 1,
    );
    session.worm_transit = Some(WormTransit {
        outpost: positions[1],
        direction: TransitDirection::ToOutpost,
        remaining: 1.0,
        ore: 2,
        ingots: 0,
        food: 0.0,
        passengers: Vec::new(),
    });

    let error = validate_loaded_session(&session, &data)
        .expect_err("stored and in-flight cargo cannot overfill the destination hold");

    assert!(error.contains("when combined with stored cargo"));
}
