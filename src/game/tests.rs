use super::*;
use crate::data::GameData;
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
