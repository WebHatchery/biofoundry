//! Colony pressure and per-building spoilage. These are kept separate from
//! job AI so morale and waste remain deterministic regardless of job order.

use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::GameSession;

pub fn tick_colony_pressure(session: &mut GameSession, data: &GameData, dt: f32) {
    let ratio = session.overcrowding_ratio(data);
    let target = if ratio > 1.0 {
        (1.0 - (ratio - 1.0) * data.balance.overcrowding_work_penalty).clamp(0.35, 1.0)
    } else {
        1.0
    };
    for creature in &mut session.creatures {
        if creature.morale < target {
            creature.morale =
                (creature.morale + data.balance.morale_recovery_per_sec * dt).min(target);
        } else {
            creature.morale =
                (creature.morale - data.balance.morale_recovery_per_sec * dt).max(target);
        }
        if creature.morale < 0.45 || ratio > 1.0 {
            creature.morale_stress_for += dt;
        } else {
            creature.morale_stress_for = (creature.morale_stress_for - dt * 2.0).max(0.0);
        }
    }
}

pub fn tick_spoilage(session: &mut GameSession, data: &GameData, dt: f32) {
    let raw_rate = data.balance.raw_spoilage_per_min / 60.0 * dt;
    let cooked_rate = data.balance.cooked_spoilage_per_min / 60.0 * dt;
    let mut raw_food = 0.0;
    for building in &mut session.buildings {
        raw_food += building.stock(Good::Mushroom);
        let raw = building.stock(Good::Mushroom);
        let spoiled = (raw * raw_rate).min(raw);
        if spoiled > 0.0 {
            building.take_stock(Good::Mushroom, spoiled);
            add_waste(building, spoiled, data);
        }
        if building.kind == "feeding_trough" {
            let cooked = building.stock(Good::CookedFood);
            let spoiled = (cooked * cooked_rate).min(cooked);
            if spoiled > 0.0 {
                building.take_stock(Good::CookedFood, spoiled);
                add_waste(building, spoiled, data);
            }
        }
        let decayed = (data.balance.waste_decay_per_min / 60.0 * dt).min(building.waste);
        building.waste -= decayed;
        session.economy.waste = (session.economy.waste - decayed).max(0.0);
    }
    session.economy.raw_food = raw_food;
    let mut total_waste: f32 = session.buildings.iter().map(|b| b.waste).sum();
    if total_waste > data.balance.waste_storage_cap {
        let mut excess = total_waste - data.balance.waste_storage_cap;
        for building in &mut session.buildings {
            let removed = excess.min(building.waste);
            building.waste -= removed;
            excess -= removed;
            if excess <= 0.0 {
                break;
            }
        }
        total_waste = data.balance.waste_storage_cap;
    }
    session.economy.waste = total_waste.max(0.0);
    // `food` is retained as the stable save/API name; this mirror gives new
    // UI and tests an explicit cooked-food resource without breaking saves.
    session.economy.cooked_food = session.economy.food;
}

fn add_waste(building: &mut crate::state::structures::Building, spoiled: f32, data: &GameData) {
    building.waste = (building.waste + spoiled * data.balance.waste_production_per_min)
        .min(data.balance.waste_storage_cap);
}

pub fn morale_status(session: &GameSession, data: &GameData) -> &'static str {
    if session.overcrowding_ratio(data) > 1.0 {
        "Overcrowded"
    } else if session.creatures.iter().any(|c| c.morale < 0.45) {
        "Low morale"
    } else {
        "Stable"
    }
}
