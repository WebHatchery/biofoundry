use super::*;

#[test]
fn circuit_outpost_inspection_adds_encore_progress_to_archive_line() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 46);
    session.outpost_charter_claimed = true;
    session.outpost_circuit_claimed = true;
    session.outpost_concord_hauls = 1;
    let outpost = biofoundry::state::outposts::Outpost::new(TilePos::new(4, 4));
    session.outposts.push(outpost);

    assert_eq!(
        outpost_archive_summary(&session, &data).as_deref(),
        Some("Archive pages 0 · Next 0/5 · Encore 1/3 · +20 ingots")
    );
}

#[test]
fn compact_circuit_outpost_summary_keeps_archive_and_encore_visible() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 47);
    session.outpost_charter_claimed = true;
    session.outpost_circuit_claimed = true;
    session.outpost_concord_hauls = 2;

    assert_eq!(
        biofoundry::ui::hud::inspect::outpost::compact_archive_summary(&session, &data).as_deref(),
        Some("Archive 0/5 · Encore 2/3")
    );
}
