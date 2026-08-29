use super::*;

#[test]
fn compact_raid_hint_names_the_controls_for_a_fresh_warren() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(session.job_count(Job::Idle), 0);
    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "tap − Miner, then + Guard"
    );
}

#[test]
fn compact_raid_hint_shortens_when_a_worker_is_idle_or_guarded() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);

    session.creatures[0].job = Job::Idle;
    assert_eq!(compact_raid_defense_hint(&session, &data), "tap + Guard");

    session.creatures[1].job = Job::Guard;
    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "guards on watch"
    );
}

#[test]
fn compact_raid_hint_uses_an_available_specialist_job() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);
    session.spawn_creature(&data, "goblin", Job::Smith);

    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "tap − Smith, then + Guard"
    );
}

#[test]
fn compact_food_hint_fits_the_top_bar() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 7);

    assert_eq!(
        compact_food_recovery_hint(&session, &data),
        "tap − Miner, then + Carrier"
    );
}

#[test]
fn compact_raid_hint_stays_short_when_a_worker_is_free() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures[0].job = Job::Idle;

    assert_eq!(compact_raid_defense_hint(&session, &data), "tap + Guard");
}

#[test]
fn hints_do_not_promise_reassignment_of_a_specialist() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 7);
    session.creatures.clear();
    session.spawn_creature(&data, "overseer", Job::Idle);

    assert_eq!(reassignable_job_count(&session, &data, Job::Idle), 0);
    assert_eq!(
        compact_raid_defense_hint(&session, &data),
        "free a worker, then + Guard"
    );
    assert_eq!(
        compact_food_recovery_hint(&session, &data),
        "free a worker, then + Carrier"
    );
}
