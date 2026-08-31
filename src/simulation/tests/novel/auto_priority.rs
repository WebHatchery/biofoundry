//! Coverage for choosing which automatic job receives the shared Worm first.

use super::active_outpost;
use crate::simulation::{self, outposts};
use crate::state::creatures::Good;
use crate::state::outposts::AutoRoutePriority;
use crate::state::structures::Building;

#[test]
fn load_first_can_precede_a_ready_automatic_return() {
    let (data, mut session, _) = active_outpost(174);
    let second_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a second walkable outpost location");
    session.buildings.push(Building::new("outpost", second_pos));
    session.ensure_outpost(second_pos);

    let capacity = data.balance.outpost_storage_cap;
    session.outposts[0].auto_return_cargo = true;
    session.outposts[0].cargo.insert(Good::Ore, capacity);
    session.outposts[1].active = true;
    session.outposts[1].auto_load = true;
    session.economy.ore_stock = 1;
    session.auto_route_priority = AutoRoutePriority::Load;

    let preview = outposts::automatic_route_preview(&session, &data)
        .expect("load-first preview should find the second route");
    assert_eq!(preview.outpost, second_pos);
    assert_eq!(preview.priority, AutoRoutePriority::Load);

    let report = simulation::tick(&mut session, &data);

    assert_eq!(report.auto_return_started, None);
    assert_eq!(report.auto_load_started, Some(second_pos));
    assert_eq!(
        session.worm_transit.as_ref().map(|transit| transit.outpost),
        Some(second_pos)
    );
}
