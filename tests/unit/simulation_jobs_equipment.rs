use biofoundry::data::GameData;
use biofoundry::simulation::jobs::equipment::*;
use biofoundry::state::creatures::{Creature, Job};
use biofoundry::state::GameSession;

#[test]
fn study_adaptation_increases_beetle_capacity_without_touching_goblins() {
    let data = GameData::load().expect("embedded game data");
    let mut session = GameSession::new(&data, 41);
    let tile = session.spawn_tile();
    let beetle = Creature::new(1, "beetle", Job::Carrier, tile);
    let goblin = Creature::new(2, "goblin", Job::Carrier, tile);
    let beetle_species = data.species.get("beetle").expect("beetle species");
    let goblin_species = data.species.get("goblin").expect("goblin species");

    assert_eq!(carry_capacity(&beetle, &session, beetle_species, &data), 5);
    assert_eq!(carry_capacity(&goblin, &session, goblin_species, &data), 1);

    session.unlocked.insert("adaptive_haulers".to_owned());

    assert_eq!(carry_capacity(&beetle, &session, beetle_species, &data), 6);
    assert_eq!(carry_capacity(&goblin, &session, goblin_species, &data), 1);
}
