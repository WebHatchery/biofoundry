//! Mushroom delivery must finish before a carrier starts another haul.

use biofoundry::data::GameData;
use biofoundry::simulation::jobs::tick_creatures;
use biofoundry::state::creatures::{Good, Job, Task};
use biofoundry::state::GameSession;

#[test]
fn partial_mushroom_loads_reach_the_pot_at_every_food_priority() {
    let data = GameData::load().unwrap();
    for species in ["goblin", "beetle", "bat_courier"] {
        for food in [0.0, data.balance.carrier_food_reserve, 500.0] {
            let mut session = GameSession::new(&data, data.config.world_seed);
            session.creatures.clear();
            session.spawn_creature(&data, species, Job::Carrier);
            session.economy.food = food;
            session.economy.ore_stock = 50;
            let farm = session.buildings_of("farm").next().unwrap().pos;
            session
                .building_at_mut(farm)
                .unwrap()
                .add_stock(Good::Mushroom, 20.0);
            let pot = session.buildings_of("cook_pot").next().unwrap().pos;
            session.building_at_mut(pot).unwrap().stocks.clear();
            let carrier = &mut session.creatures[0];
            carrier.clear_task();
            carrier.add_carried(Good::Mushroom, 1);

            tick_creatures(&mut session, &data, 0.1);
            assert_eq!(
                session.creatures[0].task,
                Task::DeliverTo(pot),
                "{species}, food={food}"
            );
            for _ in 0..2000 {
                tick_creatures(&mut session, &data, 0.1);
                if session.creatures[0].carried(Good::Mushroom) == 0 {
                    break;
                }
            }
            assert_eq!(session.creatures[0].carried(Good::Mushroom), 0);
            assert_eq!(session.building_at(pot).unwrap().stock(Good::Mushroom), 1.0);
        }
    }
}

#[test]
fn partial_farm_harvest_is_delivered_before_gathering_more() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data, data.config.world_seed);
    session.creatures.clear();
    session.spawn_creature(&data, "beetle", Job::Carrier);
    session.economy.food = 0.0;
    let farm = session.buildings_of("farm").next().unwrap().pos;
    session.building_at_mut(farm).unwrap().stocks.clear();
    session
        .building_at_mut(farm)
        .unwrap()
        .add_stock(Good::Mushroom, 1.0);
    let pot = session.buildings_of("cook_pot").next().unwrap().pos;
    let carrier = &mut session.creatures[0];
    carrier.x = farm.x as f32 + 0.5;
    carrier.y = farm.y as f32 + 0.5;
    carrier.task = Task::Fetching {
        source: farm,
        remaining: 0.0,
    };

    tick_creatures(&mut session, &data, 0.1);
    assert_eq!(session.creatures[0].carried(Good::Mushroom), 1);
    session
        .building_at_mut(farm)
        .unwrap()
        .add_stock(Good::Mushroom, 1.0);
    tick_creatures(&mut session, &data, 0.1);
    assert_eq!(session.creatures[0].task, Task::DeliverTo(pot));
}
