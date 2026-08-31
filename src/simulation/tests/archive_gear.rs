//! Coverage for the Archive Wayfinder progression payoff.

use crate::simulation::{self, outposts};
use crate::state::creatures::Job;

#[test]
fn first_archive_page_unlocks_wayfinder_after_a_save_roundtrip() {
    let (data, mut session, _) = super::novel::active_outpost(171);
    session.outpost_charter_claimed = true;
    session.outposts[0].expeditions_completed = data
        .balance
        .outpost_charter_haul_goal
        .saturating_add(data.balance.outpost_archive_haul_goal);

    assert_eq!(outposts::claim_outpost_archive(&mut session, &data), 1);
    assert_eq!(session.outpost_archive_claims, 1);
    assert!(!session.unlocked.contains("archive_wayfinder"));

    let encoded = serde_json::to_string(&session).expect("Archive page should serialize");
    let mut restored = serde_json::from_str(&encoded).expect("Archive page should deserialize");
    let report = simulation::tick(&mut restored, &data);

    assert!(report
        .wild
        .unlocked
        .iter()
        .any(|name| name == "Archive Wayfinder"));
    let wayfinder = data
        .equipment_def("archive_wayfinder")
        .expect("Archive Wayfinder data");
    assert!(restored.equipment_unlocked(wayfinder));
}

#[test]
fn archive_wayfinder_replaces_a_weaker_carrier_tool() {
    let (data, mut session, _) = super::novel::active_outpost(172);
    let stockpile = session.stockpile_pos();
    let carrier_index = session
        .creatures
        .iter()
        .position(|creature| creature.job == Job::Carrier)
        .expect("the active-outpost fixture has a carrier");
    {
        let carrier = &mut session.creatures[carrier_index];
        carrier.equipment = Some("wormbone_hauling_frame".to_owned());
        carrier.x = stockpile.x as f32 + 0.5;
        carrier.y = stockpile.y as f32 + 0.5;
        carrier.clear_task();
    }
    session.unlocked.insert("archive_wayfinder".to_owned());
    session
        .economy
        .gear_stock
        .insert("archive_wayfinder".to_owned(), 1);

    simulation::tick(&mut session, &data);

    let carrier = &session.creatures[carrier_index];
    assert_eq!(carrier.equipment.as_deref(), Some("archive_wayfinder"));
    assert_eq!(
        session.economy.gear_stock.get("wormbone_hauling_frame"),
        Some(&1)
    );
}
