//! Coverage for dedicated Engineer Mine staffing and slot ownership.

use super::boot;
use biofoundry::simulation;
use biofoundry::state::creatures::{Job, Task};
use biofoundry::state::GameSession;
use macroquad_toolkit::grid::TilePos;

fn place_at_mine(session: &mut GameSession, index: usize, mine: TilePos) {
    let creature = &mut session.creatures[index];
    creature.x = mine.x as f32 + 0.5;
    creature.y = mine.y as f32 + 0.5;
}

#[test]
fn engineer_waits_when_every_mine_slot_is_claimed() {
    let (data, mut session) = boot(501);
    let mine = session.buildings_of("mine").next().unwrap().pos;
    let slots = data
        .buildings
        .get("mine")
        .unwrap()
        .workstation
        .as_ref()
        .unwrap()
        .slots as usize;
    session.economy.food = 1_000.0;
    session.creatures.clear();

    for _ in 0..slots {
        session.spawn_creature(&data, "goblin", Job::Miner);
        let index = session.creatures.len() - 1;
        place_at_mine(&mut session, index, mine);
        session.creatures[index].task = Task::WorkMine(mine);
    }
    session.spawn_creature(&data, "engineer", Job::Engineer);
    let engineer = session.creatures.len() - 1;
    place_at_mine(&mut session, engineer, mine);

    simulation::tick(&mut session, &data);

    assert_eq!(session.creatures[engineer].task, Task::Idle);
    assert_eq!(
        session
            .creatures
            .iter()
            .filter(|creature| matches!(creature.task, Task::WorkMine(pos) if pos == mine))
            .count(),
        slots
    );
}

#[test]
fn engineer_claims_a_free_mine_slot_without_overbooking_it() {
    let (data, mut session) = boot(502);
    let mine = session.buildings_of("mine").next().unwrap().pos;
    let slots = data
        .buildings
        .get("mine")
        .unwrap()
        .workstation
        .as_ref()
        .unwrap()
        .slots as usize;
    session.economy.food = 1_000.0;
    session.creatures.clear();

    for _ in 0..slots.saturating_sub(1) {
        session.spawn_creature(&data, "goblin", Job::Miner);
        let index = session.creatures.len() - 1;
        place_at_mine(&mut session, index, mine);
        session.creatures[index].task = Task::WorkMine(mine);
    }
    session.spawn_creature(&data, "engineer", Job::Engineer);
    let engineer = session.creatures.len() - 1;
    place_at_mine(&mut session, engineer, mine);

    simulation::tick(&mut session, &data);
    simulation::tick(&mut session, &data);

    assert_eq!(session.creatures[engineer].task, Task::WorkMine(mine));
    assert_eq!(
        session
            .creatures
            .iter()
            .filter(|creature| matches!(creature.task, Task::WorkMine(pos) if pos == mine))
            .count(),
        slots
    );
}
