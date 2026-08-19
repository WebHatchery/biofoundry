//! Slime Janitors close the food loop: they remove spoiled stores and carry
//! cooked food into visible feeding troughs.

use crate::data::GameData;
use crate::simulation::jobs::routing::send_to;
use crate::state::creatures::{Creature, Good, Task};
use crate::state::GameSession;

pub(super) fn tick_janitor(
    creature: &mut Creature,
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
    work_boost: f32,
) {
    match creature.task.clone() {
        Task::Idle => {
            if let Some(target) = session
                .buildings
                .iter()
                .filter(|b| b.waste > 0.0)
                .min_by_key(|b| (b.pos.manhattan_distance(&creature.tile()), b.pos.x, b.pos.y))
                .map(|b| b.pos)
                .or_else(|| {
                    session
                        .buildings_of("feeding_trough")
                        .filter(|b| {
                            b.stock(Good::CookedFood) < data.balance.trough_food_cap
                                && session.economy.food > data.balance.worm_feed_reserve
                        })
                        .min_by_key(|b| {
                            (b.pos.manhattan_distance(&creature.tile()), b.pos.x, b.pos.y)
                        })
                        .map(|b| b.pos)
                })
            {
                send_to(creature, session, target, Task::GoClean(target));
            }
        }
        Task::GoClean(pos) => {
            if creature.tile() == pos {
                let trough = session
                    .building_at(pos)
                    .is_some_and(|b| b.kind == "feeding_trough");
                creature.task = if trough {
                    Task::Feeding {
                        trough: pos,
                        remaining: data.balance.haul_pickup_sec,
                    }
                } else {
                    Task::Cleaning {
                        building: pos,
                        remaining: data.balance.haul_pickup_sec,
                    }
                };
            } else {
                creature.task = Task::Idle;
            }
        }
        Task::Cleaning {
            building,
            remaining,
        } => {
            let left = remaining - dt * creature.work_speed() * work_boost;
            if left > 0.0 {
                creature.task = Task::Cleaning {
                    building,
                    remaining: left,
                };
                return;
            }
            let removed = session
                .building_at_mut(building)
                .map(|b| {
                    let amount = b.waste.min(data.balance.janitor_clean_per_min * dt / 60.0);
                    b.waste -= amount;
                    amount
                })
                .unwrap_or(0.0);
            session.economy.waste = (session.economy.waste - removed).max(0.0);
            session.economy.waste_processed += removed;
            session.progress.waste_processed = session
                .progress
                .waste_processed
                .max(session.economy.waste_processed.floor() as u32);
            creature.task = Task::Idle;
        }
        Task::Feeding { trough, remaining } => {
            let left = remaining - dt * creature.work_speed() * work_boost;
            if left > 0.0 {
                creature.task = Task::Feeding {
                    trough,
                    remaining: left,
                };
                return;
            }
            let amount = data.balance.trough_feed_per_min * dt / 60.0;
            let fed = amount.min(session.economy.food);
            if let Some(b) = session.building_at_mut(trough) {
                let room = (data.balance.trough_food_cap - b.stock(Good::CookedFood)).max(0.0);
                let moved = fed.min(room);
                if moved > 0.0 {
                    b.add_stock(Good::CookedFood, moved);
                    session.economy.food -= moved;
                }
            }
            creature.task = Task::Idle;
        }
        _ => creature.task = Task::Idle,
    }
}
