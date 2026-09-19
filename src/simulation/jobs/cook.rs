//! Cooks: the kitchen node. Mushrooms in, calories on the grid out.

use crate::data::GameData;
use crate::simulation::jobs::routing::{nearest_building, nearest_building_where, send_to};
use crate::simulation::storage;
use crate::state::creatures::{Creature, Good, Task};
use crate::state::GameSession;

pub(super) fn tick_cook(
    creature: &mut Creature,
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
    work_boost: f32,
) {
    let batch = (data.balance.cook_batch_mushrooms as f32 * data.balance.raw_recipe_multiplier)
        .ceil() as u32;
    match creature.task.clone() {
        Task::Idle => {
            if creature.carried(Good::Mushroom) > 0 {
                if let Some(pot) = nearest_building(creature, session, "cook_pot") {
                    send_to(creature, session, pot, Task::DeliverTo(pot));
                }
                return;
            }
            // Work the nearest pot with a full batch waiting.
            let stocked = nearest_building_where(creature, session, "cook_pot", |b| {
                b.stock(Good::Mushroom) >= batch as f32
            });
            if let Some(pot) = stocked {
                if creature.tile() == pot {
                    if let Some(building) = session.building_at_mut(pot) {
                        // Ingredients are claimed up front so two cooks
                        // can't share one batch.
                        building.take_stock(Good::Mushroom, batch as f32);
                        creature.task = Task::Cooking {
                            pot,
                            remaining: data.balance.cook_batch_time_sec,
                        };
                    }
                } else {
                    send_to(creature, session, pot, Task::GoCook(pot));
                }
                return;
            }
            // Nothing to cook: wait at the nearest pot.
            if let Some(pot) = nearest_building(creature, session, "cook_pot") {
                if let Some(source) = storage::source_for(session, data, pot, Good::Mushroom) {
                    send_to(creature, session, source, Task::GoFetch(source));
                    return;
                }
                if creature.tile() != pot {
                    send_to(creature, session, pot, Task::GoCook(pot));
                }
            }
        }
        Task::GoCook(_) => creature.task = Task::Idle,
        Task::Cooking { pot, remaining } => {
            let left = remaining - dt * creature.work_speed() * work_boost;
            if left > 0.0 {
                creature.task = Task::Cooking {
                    pot,
                    remaining: left,
                };
            } else {
                session.economy.food +=
                    data.balance.cook_batch_food * data.balance.cooked_recipe_multiplier;
                creature.task = Task::Idle;
            }
        }
        _ => tick_supply(creature, session, data, dt, work_boost, batch),
    }
}

fn tick_supply(
    creature: &mut Creature,
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
    work_boost: f32,
    batch: u32,
) {
    match creature.task.clone() {
        Task::GoFetch(source) => {
            if creature.tile() == source {
                creature.task = Task::Fetching {
                    source,
                    remaining: data.balance.haul_pickup_sec,
                };
            } else {
                creature.task = Task::Idle;
            }
        }
        Task::Fetching { source, remaining } => {
            let left = remaining - dt * creature.work_speed() * work_boost;
            if left > 0.0 {
                creature.task = Task::Fetching {
                    source,
                    remaining: left,
                };
                return;
            }
            if let Some(store) = session.building_at_mut(source) {
                if storage::definition(store, data).is_some()
                    && store.accepted_good() == Good::Mushroom
                {
                    let take = batch.min(store.stock(Good::Mushroom).floor() as u32);
                    store.take_stock(Good::Mushroom, take as f32);
                    creature.add_carried(Good::Mushroom, take);
                }
            }
            creature.task = Task::Idle;
        }
        Task::DeliverTo(pot) => {
            if creature.tile() == pot {
                if let Some(building) = session.building_at_mut(pot) {
                    if building.kind == "cook_pot" {
                        let take = creature.take_carried(Good::Mushroom, u32::MAX);
                        building.add_stock(Good::Mushroom, take as f32);
                    }
                }
            }
            creature.task = Task::Idle;
        }
        _ => creature.task = Task::Idle,
    }
}
