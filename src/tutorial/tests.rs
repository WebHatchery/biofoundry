use super::*;

fn boot() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, 42);
    (data, session)
}

#[test]
fn tutorial_data_loads_in_teaching_order() {
    let (data, _) = boot();
    assert_eq!(
        data.tutorial.len(),
        5,
        "tutorial follows the five journey beats"
    );
    assert_eq!(data.tutorial.first().unwrap().id, "enter");
    assert!(matches!(
        data.tutorial.last().unwrap().done,
        TutorialDone::WormAwake
    ));
}

#[test]
fn steps_complete_from_player_actions() {
    let (data, mut session) = boot();
    let none = TutorialInputs::default();

    // 1. Look around.
    assert_eq!(current_step(&session, &data).unwrap().id, "enter");
    assert!(!advance(&mut session, &data, none));
    assert!(advance(
        &mut session,
        &data,
        TutorialInputs { camera_moved: true }
    ));
    assert_eq!(current_step(&session, &data).unwrap().id, "food");

    // 2. Place a build site.
    assert!(!advance(&mut session, &data, none));
    session.tutorial_built = true;
    assert!(advance(&mut session, &data, none));

    // 3. Close the living factory loop with the pickaxe.
    assert_eq!(current_step(&session, &data).unwrap().id, "factory");
    assert!(!advance(&mut session, &data, none));
    session
        .economy
        .gear_stock
        .insert("iron_pickaxe".to_owned(), 1);
    assert!(advance(&mut session, &data, none));

    // 4. Secure the warren.
    assert_eq!(current_step(&session, &data).unwrap().id, "secure");
    assert!(!advance(&mut session, &data, none));
    session.won = true;
    assert!(advance(&mut session, &data, none));

    // 5. The awakened worm finishes the tutorial.
    assert_eq!(current_step(&session, &data).unwrap().id, "worm");
    session.worm_awake = true;
    assert!(advance(&mut session, &data, none));
    assert!(current_step(&session, &data).is_none(), "tutorial finished");
    let (done, total) = progress(&session, &data);
    assert_eq!(done, total);
}

#[test]
fn final_tutorial_waits_for_the_worm() {
    let (data, mut session) = boot();
    let none = TutorialInputs::default();
    session.tutorial_step = data.tutorial.iter().position(|s| s.id == "worm").unwrap();
    session.won = true;

    assert!(!advance(&mut session, &data, none));
    session.worm_awake = true;
    assert!(advance(&mut session, &data, none));
    assert!(current_step(&session, &data).is_none());
}

#[test]
fn secure_step_waits_for_the_campaign_goal() {
    let (data, mut session) = boot();
    let none = TutorialInputs::default();
    let secure_index = data.tutorial.iter().position(|s| s.id == "secure").unwrap();
    let secure = &data.tutorial[secure_index];
    session.tutorial_step = secure_index;
    assert_eq!(current_step(&session, &data).unwrap().id, "secure");
    assert!(secure.body.contains("Guard in Jobs"));
    assert!(secure.body.contains("ends onboarding"));
    assert!(!advance(&mut session, &data, none));
    session.won = true;
    assert!(advance(&mut session, &data, none));
    assert_eq!(current_step(&session, &data).unwrap().id, "worm");
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
