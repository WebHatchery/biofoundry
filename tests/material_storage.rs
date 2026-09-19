//! Local storage routes, conservation, configuration, and save compatibility.

use biofoundry::data::GameData;
use biofoundry::simulation::{jobs::tick_creatures, storage};
use biofoundry::state::creatures::{Good, Job, Task};
use biofoundry::state::structures::Building;
use biofoundry::state::world::Tile;
use biofoundry::state::GameSession;
use macroquad_toolkit::grid::TilePos;

fn setup() -> (GameData, GameSession, TilePos, TilePos) {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, data.config.world_seed);
    session.creatures.clear();
    let pot = session.buildings_of("cook_pot").next().unwrap().pos;
    session.building_at_mut(pot).unwrap().stocks.clear();
    let pos = session
        .world
        .tiles
        .iter_with_pos()
        .filter(|(p, _)| session.can_place_building(*p))
        .map(|(p, _)| p)
        .min_by_key(|p| (p.manhattan_distance(&pot), p.x, p.y))
        .unwrap();
    session
        .buildings
        .push(Building::new("material_stockpile", pos));
    (data, session, pos, pot)
}

#[test]
fn farm_to_storage_to_cook_produces_food() {
    let (data, mut session, pos, pot) = setup();
    session.economy.food = 0.0;
    let farm = session.buildings_of("farm").next().unwrap().pos;
    session.building_at_mut(farm).unwrap().stocks.clear();
    session
        .building_at_mut(farm)
        .unwrap()
        .add_stock(Good::Mushroom, 12.0);
    session.spawn_creature(&data, "beetle", Job::Carrier);
    let carrier = &mut session.creatures[0];
    carrier.x = farm.x as f32 + 0.5;
    carrier.y = farm.y as f32 + 0.5;
    for _ in 0..3000 {
        tick_creatures(&mut session, &data, 0.1);
        if session.building_at(pos).unwrap().stock(Good::Mushroom) > 0.0 {
            break;
        }
    }
    assert!(session.building_at(pos).unwrap().stock(Good::Mushroom) > 0.0);
    assert_eq!(session.building_at(pot).unwrap().stock(Good::Mushroom), 0.0);
    session.spawn_creature(&data, "goblin", Job::Cook);
    for _ in 0..3000 {
        tick_creatures(&mut session, &data, 0.1);
        if session.economy.food > 0.0 {
            break;
        }
    }
    assert!(
        session.economy.food > 0.0,
        "the cook must fetch from storage and cook"
    );
}

#[test]
fn competing_deliveries_respect_capacity_and_preserve_leftovers() {
    let (data, mut session, pos, _) = setup();
    let capacity = storage::definition(session.building_at(pos).unwrap(), &data)
        .unwrap()
        .capacity;
    session
        .building_at_mut(pos)
        .unwrap()
        .add_stock(Good::Mushroom, capacity as f32 - 1.0);
    for _ in 0..2 {
        session.spawn_creature(&data, "beetle", Job::Carrier);
        let c = session.creatures.last_mut().unwrap();
        c.x = pos.x as f32 + 0.5;
        c.y = pos.y as f32 + 0.5;
        c.task = Task::DeliverTo(pos);
        c.add_carried(Good::Mushroom, 4);
    }
    tick_creatures(&mut session, &data, 0.1);
    assert_eq!(
        session.building_at(pos).unwrap().stock(Good::Mushroom),
        capacity as f32
    );
    assert_eq!(
        session
            .creatures
            .iter()
            .map(|c| c.carried(Good::Mushroom))
            .sum::<u32>(),
        7
    );
    assert_eq!(
        storage::deposit_destination(&session, &data, pos, Good::Mushroom),
        None
    );
}

#[test]
fn filters_persist_reject_occupied_changes_and_keep_old_buildings_loadable() {
    let (data, mut session, pos, _) = setup();
    for good in [Good::Wood, Good::Charcoal, Good::Mushroom] {
        assert!(storage::cycle_filter(&mut session, &data, pos));
        assert_eq!(session.building_at(pos).unwrap().accepted_good(), good);
    }
    session
        .building_at_mut(pos)
        .unwrap()
        .add_stock(Good::Mushroom, 1.5);
    assert!(!storage::cycle_filter(&mut session, &data, pos));
    let encoded = serde_json::to_string(&session).unwrap();
    let restored: GameSession = serde_json::from_str(&encoded).unwrap();
    let saved = restored.building_at(pos).unwrap();
    assert_eq!(saved.accepted_good(), Good::Mushroom);
    assert_eq!(saved.stock(Good::Mushroom), 1.5);
    biofoundry::game::persistence::validate_loaded_session(&restored, &data).unwrap();
    session
        .building_at_mut(pos)
        .unwrap()
        .take_stock(Good::Mushroom, 1.0);
    assert!(storage::cycle_filter(&mut session, &data, pos));
    assert_eq!(session.building_at(pos).unwrap().waste, 0.5);
    assert_eq!(session.building_at(pos).unwrap().stock(Good::Mushroom), 0.0);
    let mut legacy = serde_json::to_value(Building::new("farm", pos)).unwrap();
    legacy.as_object_mut().unwrap().remove("storage_good");
    assert!(serde_json::from_value::<Building>(legacy).is_ok());
    assert!(!storage::cycle_filter(
        &mut session,
        &data,
        TilePos::new(0, 0)
    ));
}

#[test]
fn storage_requires_a_reachable_nearby_consumer_and_matching_goods() {
    let (data, mut session, pos, pot) = setup();
    assert_eq!(
        storage::deposit_destination(&session, &data, pot, Good::Mushroom),
        Some(pos)
    );
    assert_eq!(
        storage::deposit_destination(&session, &data, pot, Good::Wood),
        None
    );
    session
        .building_at_mut(pos)
        .unwrap()
        .add_stock(Good::Mushroom, 3.0);
    assert_eq!(
        storage::source_for(&session, &data, pot, Good::Mushroom),
        Some(pos)
    );
    assert_eq!(
        storage::source_for(&session, &data, TilePos::new(0, 0), Good::Mushroom),
        None
    );
    // Block every approach to the pot while leaving its own tile intact.
    for p in [
        TilePos::new(pot.x - 1, pot.y),
        TilePos::new(pot.x + 1, pot.y),
        TilePos::new(pot.x, pot.y - 1),
        TilePos::new(pot.x, pot.y + 1),
    ] {
        session.world.tiles.set(p, Tile::Rock);
    }
    assert_eq!(
        storage::source_for(&session, &data, pot, Good::Mushroom),
        None
    );
    assert_eq!(
        storage::deposit_destination(&session, &data, pos, Good::Mushroom),
        None
    );
}

#[test]
fn carriers_supply_industry_from_storage_without_putting_the_load_back() {
    for (good, kind) in [(Good::Wood, "kiln"), (Good::Charcoal, "smelter")] {
        let (data, mut session, pos, pot) = setup();
        session.economy.food = 500.0;
        session.economy.ore_stock = 0;
        let target = session.building_at_mut(pot).unwrap();
        target.kind = kind.to_owned();
        let store = session.building_at_mut(pos).unwrap();
        store.storage_good = Some(good);
        store.add_stock(good, 2.5);
        session.spawn_creature(&data, "beetle", Job::Carrier);
        let carrier = &mut session.creatures[0];
        carrier.x = pos.x as f32 + 0.5;
        carrier.y = pos.y as f32 + 0.5;
        for _ in 0..1000 {
            tick_creatures(&mut session, &data, 0.1);
            if session.building_at(pot).unwrap().stock(good) >= 2.0 {
                break;
            }
        }
        assert_eq!(session.building_at(pot).unwrap().stock(good), 2.0);
        assert_eq!(session.building_at(pos).unwrap().stock(good), 0.5);
    }
}
