use super::*;

#[test]
fn raid_hint_names_the_controls_for_a_fresh_warren() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(session.job_count(Job::Idle), 0);
    assert_eq!(
        raid_defense_hint(&session),
        "tap − beside Miner, then + beside Guard"
    );
}

#[test]
fn raid_hint_shortens_when_a_worker_is_idle_or_guarded() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);

    session.creatures[0].job = Job::Idle;
    assert_eq!(raid_defense_hint(&session), "tap + beside Guard in Jobs");

    session.creatures[1].job = Job::Guard;
    assert_eq!(raid_defense_hint(&session), "Guards are on watch.");
}

#[test]
fn compact_food_hint_fits_the_top_bar() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(
        compact_food_recovery_hint(&session),
        "tap − Miner, then + Carrier"
    );
}

#[test]
fn compact_raid_hint_names_the_guard_control() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(
        compact_raid_defense_hint(&session),
        "tap − Miner, then + Guard"
    );
}

#[test]
fn compact_raid_hint_stays_short_when_a_worker_is_free() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures[0].job = Job::Idle;

    assert_eq!(compact_raid_defense_hint(&session), "tap + Guard");
}
