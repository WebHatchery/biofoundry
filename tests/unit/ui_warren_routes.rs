use biofoundry::data::GameData;
use biofoundry::state::outposts::{Outpost, TransitDirection};
use biofoundry::ui::warren::routes::*;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;

#[test]
fn outbound_transit_pulse_starts_at_the_shrine_and_reaches_the_outpost() {
    assert_eq!(
        transit_route_fraction(30.0, 30.0, TransitDirection::ToOutpost),
        0.0
    );
    assert_eq!(
        transit_route_fraction(0.0, 30.0, TransitDirection::ToOutpost),
        1.0
    );
}

#[test]
fn return_transit_pulse_starts_at_the_outpost_and_reaches_the_shrine() {
    assert_eq!(
        transit_route_fraction(30.0, 30.0, TransitDirection::ToShrine),
        1.0
    );
    assert_eq!(
        transit_route_fraction(0.0, 30.0, TransitDirection::ToShrine),
        0.0
    );
}

#[test]
fn route_pulse_progress_clamps_malformed_remaining_time() {
    assert_eq!(
        transit_route_fraction(-2.0, 30.0, TransitDirection::ToOutpost),
        1.0
    );
    assert_eq!(
        transit_route_fraction(40.0, 30.0, TransitDirection::ToShrine),
        1.0
    );
    assert_eq!(
        transit_route_fraction(4.0, 0.0, TransitDirection::ToOutpost),
        1.0
    );
}

#[test]
fn route_point_interpolates_between_shrine_and_outpost() {
    let point = route_point(vec2(10.0, 20.0), vec2(30.0, 60.0), 0.25);
    assert_eq!(point, vec2(15.0, 30.0));
}

#[test]
fn waypoint_route_pulse_uses_the_shorter_transit_time() {
    let data = GameData::load().expect("embedded game data");
    let mut outpost = Outpost::new(TilePos::new(4, 4));
    outpost.waypoint_upgraded = true;

    assert_eq!(
        transit_total_sec(&outpost, &data),
        data.balance.outpost_waypoint_transit_time_sec
    );
    assert_eq!(
        transit_route_fraction(
            data.balance.outpost_waypoint_transit_time_sec,
            transit_total_sec(&outpost, &data),
            TransitDirection::ToOutpost,
        ),
        0.0
    );
}
