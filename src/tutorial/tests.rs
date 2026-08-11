use super::*;
use crate::state::creatures::Job;

fn boot() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, 42);
    (data, session)
}

#[test]
fn tutorial_data_loads_in_teaching_order() {
    let (data, _) = boot();
    assert!(data.tutorial.len() >= 4, "expected a real tutorial");
    assert_eq!(data.tutorial.first().unwrap().id, "welcome");
    assert!(matches!(
        data.tutorial.last().unwrap().done,
        TutorialDone::Won
    ));
}

#[test]
fn steps_complete_from_player_actions() {
    use crate::state::creatures::Good;
    let (data, mut session) = boot();
    let none = TutorialInputs::default();

    // 1. Look around.
    assert_eq!(current_step(&session, &data).unwrap().id, "welcome");
    assert!(!advance(&mut session, &data, none));
    assert!(advance(
        &mut session,
        &data,
        TutorialInputs { camera_moved: true }
    ));
    assert_eq!(current_step(&session, &data).unwrap().id, "food_grid");

    // 2. Place a build site.
    assert!(!advance(&mut session, &data, none));
    session.tutorial_built = true;
    assert!(advance(&mut session, &data, none));

    // 3. Meet the Mine — completes once it has extracted ore.
    assert_eq!(current_step(&session, &data).unwrap().id, "mine");
    assert!(!advance(&mut session, &data, none));
    let mine = session.buildings_of("mine").next().unwrap().pos;
    session
        .building_at_mut(mine)
        .unwrap()
        .add_stock(Good::Ore, 2.0);
    assert!(advance(&mut session, &data, none));

    // 4. Place the Blacksmith.
    assert_eq!(current_step(&session, &data).unwrap().id, "blacksmith");
    assert!(!advance(&mut session, &data, none));
    let spot = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, _)| session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .unwrap();
    session
        .buildings
        .push(crate::state::structures::Building::new("blacksmith", spot));
    assert!(advance(&mut session, &data, none));

    // 5. Weather the famine (reassign + positive balance).
    assert_eq!(current_step(&session, &data).unwrap().id, "famine");
    session.reassign(Job::Miner, Job::Carrier, |_| true);
    session.economy.production_ema_per_min = 999.0;
    assert!(advance(&mut session, &data, none));

    // 6. Craft a pickaxe.
    assert_eq!(current_step(&session, &data).unwrap().id, "pickaxe");
    assert!(!advance(&mut session, &data, none));
    session
        .economy
        .gear_stock
        .insert("iron_pickaxe".to_owned(), 1);
    assert!(advance(&mut session, &data, none));

    // 7. Win finishes the tutorial.
    assert_eq!(current_step(&session, &data).unwrap().id, "goals");
    session.won = true;
    assert!(advance(&mut session, &data, none));
    assert!(current_step(&session, &data).is_none(), "tutorial finished");
    let (done, total) = progress(&session, &data);
    assert_eq!(done, total);
}

#[test]
fn famine_step_also_clears_after_recovery() {
    let (data, mut session) = boot();
    let none = TutorialInputs::default();
    // Jump to the famine step (index 4 in the seven-step flow).
    session.tutorial_step = data.tutorial.iter().position(|s| s.id == "famine").unwrap();
    assert_eq!(current_step(&session, &data).unwrap().id, "famine");

    // No extra carriers, no production — only riding out the crisis
    // window with a healthy larder counts.
    session.economy.food = 50.0;
    assert!(!advance(&mut session, &data, none), "too early to count");
    session.tick = (400.0 / simulation::SIM_DT) as u64;
    assert!(advance(&mut session, &data, none));
    assert_eq!(current_step(&session, &data).unwrap().id, "pickaxe");
}

#[test]
fn dismissed_tutorial_shows_nothing() {
    let (data, mut session) = boot();
    session.tutorial_dismissed = true;
    assert!(current_step(&session, &data).is_none());
    assert!(!advance(
        &mut session,
        &data,
        TutorialInputs { camera_moved: true }
    ));
}
