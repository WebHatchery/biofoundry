use super::*;
use crate::data::GameData;
use crate::state::creatures::Job;

fn boot() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, 42);
    (data, session)
}

#[test]
fn ledger_matches_tier0_table() {
    let (data, session) = boot();
    let b = &data.balance;

    // Miners and carriers draw base upkeep; cooks draw the cook factor.
    let base = data.species.get("goblin").unwrap().food_per_min;
    let expected = (b.start_miners + b.start_carriers) as f32 * base
        + b.start_cooks as f32 * base * b.cook_upkeep_factor;
    let consumption = consumption_per_min(&session, &data);
    assert!((consumption - expected).abs() < 1e-4);
}

#[test]
fn idle_creatures_draw_reduced_rate() {
    let (data, mut session) = boot();
    let before = consumption_per_min(&session, &data);

    for c in &mut session.creatures {
        if c.job == Job::Miner {
            c.job = Job::Idle;
        }
    }
    let after = consumption_per_min(&session, &data);
    assert!(after < before);
}

#[test]
fn stockpile_drains_and_satiation_recovers_while_fed() {
    let (data, mut session) = boot();
    let food_before = session.economy.food;
    for c in &mut session.creatures {
        c.satiation = 0.5;
    }

    let deserters = tick_hunger(&mut session, &data, 1.0);

    assert!(deserters.is_empty());
    assert!(session.economy.food < food_before);
    assert!(session.creatures.iter().all(|c| c.satiation > 0.5));
}

#[test]
fn empty_stockpile_causes_brownout_then_desertion() {
    let (data, mut session) = boot();
    session.economy.food = 0.0;

    // Brownout: satiation decays, work speed drops.
    let drain = data.balance.satiation_drain_sec;
    let steps = (drain * 0.75 / 0.1) as usize;
    for _ in 0..steps {
        tick_hunger(&mut session, &data, 0.1);
    }
    assert!(session.creatures.iter().all(|c| c.work_speed() < 1.0));

    // Blackout: sustained starvation deserts everyone eventually.
    let total = session.creatures.len();
    let more = ((drain + data.balance.desert_after_starving_sec + 2.0) / 0.1) as usize;
    for _ in 0..more {
        tick_hunger(&mut session, &data, 0.1);
    }
    assert!(session.creatures.is_empty());
    assert_eq!(session.economy.deserted as usize, total);
}

#[test]
fn refeeding_recovers_a_brownout() {
    let (data, mut session) = boot();
    session.economy.food = 0.0;
    for _ in 0..300 {
        tick_hunger(&mut session, &data, 0.1);
    }
    assert!(session.creatures.iter().all(|c| c.work_speed() < 1.0));

    session.economy.food = 50.0;
    for _ in 0..((data.balance.satiation_recover_sec / 0.1) as usize + 10) {
        tick_hunger(&mut session, &data, 0.1);
    }
    assert!(session.creatures.iter().all(|c| c.work_speed() == 1.0));
    assert!(session.creatures.iter().all(|c| c.starving_for == 0.0));
}
