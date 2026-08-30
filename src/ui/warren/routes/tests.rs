use super::*;

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
