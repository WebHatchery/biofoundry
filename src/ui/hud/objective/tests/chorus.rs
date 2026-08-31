use super::*;
use crate::state::outposts::Outpost;
use macroquad_toolkit::grid::TilePos;

#[test]
fn completed_objective_guides_the_optional_wormsong_chorus() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.outpost_circuit_claimed = true;
    session.outposts = (0..3)
        .map(|index| {
            let mut route = Outpost::new(TilePos::new(4 + index * 2, 4));
            route.active = true;
            route
        })
        .collect();

    let guidance = chorus_contract_guidance(&session, &data).expect("Chorus should guide");

    assert!(guidance.contains("Wormsong Chorus"));
    assert!(guidance.contains("0/3 complete routes"));
    assert!(guidance.contains("+80 ingots"));
    assert!(guidance.contains("+1 ingot/haul"));
}

#[test]
fn claimed_chorus_returns_objective_to_the_repeatable_encore() {
    let (data, mut session) = boot();
    session.worm_awake = true;
    session.outpost_circuit_claimed = true;
    session.outpost_chorus_claimed = true;

    assert!(chorus_contract_guidance(&session, &data).is_none());
    assert!(encore_contract_guidance(&session, &data).is_some());
}
