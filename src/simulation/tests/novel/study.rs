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

#[test]
fn additional_study_pens_add_observation_throughput() {
    let (data, mut session) = boot(55);
    session.progress.specimens = 2;
    let study_positions: Vec<_> = session
        .world
        .tiles
        .iter_with_pos()
        .filter(|(pos, tile)| tile.walkable() && session.can_place_building(*pos))
        .map(|(pos, _)| pos)
        .take(2)
        .collect();
    assert_eq!(study_positions.len(), 2);

    session
        .buildings
        .push(Building::new("study_pen", study_positions[0]));
    assert_eq!(
        simulation::wildlife::study_rate_per_min(&session, &data),
        2.0
    );

    session
        .buildings
        .push(Building::new("study_pen", study_positions[1]));
    assert_eq!(
        simulation::wildlife::study_rate_per_min(&session, &data),
        4.0
    );
}
