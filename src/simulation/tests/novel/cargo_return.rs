//! Coverage for cargo-only remote returns.

use super::active_outpost;
use crate::simulation::outposts;
use crate::state::creatures::{Good, Job};

#[test]
fn cargo_only_return_keeps_remote_crew_at_the_outpost() {
    let (data, mut session, outpost_pos) = active_outpost(27);
    session.creatures.clear();
    session.economy.food = 0.0;
    session.economy.ore_stock = 0;
    session.economy.ingots_stock = 0;
    session.spawn_creature(&data, "goblin", Job::Carrier);
    let crew_id = session.creatures[0].id;

    assert!(outposts::start_to_outpost(&mut session, &data, outpost_pos));
    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    session.outposts[0].cargo.insert(Good::Ore, 4);

    assert!(outposts::start_cargo_to_shrine(
        &mut session,
        &data,
        outpost_pos
    ));
    let transit = session
        .worm_transit
        .as_ref()
        .expect("cargo return is in flight");
    assert!(transit.passengers.is_empty());
    assert!(session.outposts[0].crew.contains(&crew_id));
    assert!(session.outposts[0].cargo.is_empty());

    outposts::tick_transit(
        &mut session,
        &data,
        data.balance.worm_transit_time_sec + 0.1,
    );
    assert_eq!(session.economy.ore_stock, 4);
    assert_eq!(session.outposts[0].crew, vec![crew_id]);
    assert_eq!(session.creatures[0].remote_outpost, Some(outpost_pos));
}
