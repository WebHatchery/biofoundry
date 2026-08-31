use super::boot;
use crate::simulation;
use crate::state::structures::Building;

#[test]
fn study_pen_grants_adaptive_haulers_when_observation_crosses_threshold() {
    let (data, mut session) = boot(53);
    let study_pos = session
        .world
        .tiles
        .iter_with_pos()
        .find(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .expect("a walkable study pen location");
    session
        .buildings
        .push(Building::new("study_pen", study_pos));
    session.progress.specimens = 2;
    session.progress.knowledge = 11.8;

    for _ in 0..120 {
        simulation::tick(&mut session, &data);
    }

    assert!(session.progress.knowledge >= 12.0);
    assert!(session.unlocked.contains("adaptive_haulers"));
}

#[test]
fn brood_memory_shortens_the_breeding_cycle() {
    let (data, mut session) = boot(54);
    assert_eq!(
        simulation::wildlife::breeding_interval_sec(&session, &data),
        data.balance.breed_interval_sec
    );

    session.unlocked.insert("brood_memory".to_owned());

    assert_eq!(
        simulation::wildlife::breeding_interval_sec(&session, &data),
        data.balance.breed_interval_sec * 0.8
    );
}
