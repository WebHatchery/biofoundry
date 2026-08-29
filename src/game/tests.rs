use super::*;
use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::outposts::{TransitCompletion, TransitDirection};
use crate::state::GameSession;

fn session() -> (GameData, GameSession) {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data, 42);
    (data, session)
}

#[test]
fn old_tutorial_index_does_not_skip_the_new_factory_lesson() {
    let (data, mut session) = session();
    session.tutorial_step = 4; // old Blacksmith/famine territory
    session.tutorial_built = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, 2);
}

#[test]
fn tutorial_migration_uses_completed_campaign_facts() {
    let (data, mut session) = session();
    session.tutorial_step = 6;
    session.tutorial_built = true;
    session.won = true;
    session.worm_awake = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());

    assert_eq!(session.tutorial_step, data.tutorial.len());
}

#[test]
fn tutorial_migration_keeps_unwitnessed_guard_lesson_visible() {
    let (data, mut session) = session();
    session.tutorial_built = true;
    session
        .economy
        .gear_stock
        .insert("iron_pickaxe".to_owned(), 1);
    session.won = true;

    migrate_tutorial_progress(&mut session, data.tutorial.len());
    assert_eq!(session.tutorial_step, 3);

    session.creatures[0].job = Job::Guard;
    migrate_tutorial_progress(&mut session, data.tutorial.len());
    assert_eq!(session.tutorial_step, 4);
}

#[test]
fn secure_threshold_notice_waits_for_the_guard_handoff() {
    let (_data, mut session) = session();

    assert_eq!(
        warren_secured_notice(&session),
        "The reserve gate is secure — assign a Guard to finish onboarding."
    );

    session.creatures[0].job = Job::Guard;
    assert_eq!(
        warren_secured_notice(&session),
        "The warren is secure — onboarding complete."
    );
}

#[test]
fn transit_completion_notice_names_a_crew_only_arrival() {
    let completion = TransitCompletion {
        direction: TransitDirection::ToOutpost,
        cargo_units: 0,
        passenger_count: 2,
    };

    assert_eq!(
        transit_completion_notice(completion),
        "The worm reaches the outpost — crew delivered."
    );
}

#[test]
fn transit_completion_notice_names_mixed_payloads() {
    let completion = TransitCompletion {
        direction: TransitDirection::ToShrine,
        cargo_units: 3,
        passenger_count: 1,
    };

    assert_eq!(
        transit_completion_notice(completion),
        "The worm returns to the shrine — cargo and crew delivered."
    );
}
