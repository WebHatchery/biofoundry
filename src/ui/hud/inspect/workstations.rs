//! Shared inspection helpers for staffed production workstations.

use crate::data::GameData;
use crate::state::creatures::{Creature, Good, Job, Task};
use crate::state::structures::Building;
use crate::ui::hud::requirements::unlock_requirement;
use macroquad_toolkit::grid::TilePos;

pub(super) fn equipment_lock_label(
    data: &GameData,
    equipment: &crate::data::EquipmentDef,
) -> String {
    match equipment.requires_unlock.as_deref() {
        Some(crate::data::OUTPOST_CHARTER_UNLOCK) => "Charter required".to_owned(),
        Some(unlock_id) => unlock_requirement(data, unlock_id)
            .map(|phrase| format!("Needs {phrase}"))
            .unwrap_or_else(|| "Unlock required".to_owned()),
        None => "Unlock required".to_owned(),
    }
}

pub(super) fn blacksmith_queue_available(building: &Building, data: &GameData) -> bool {
    building.orders.len() < data.balance.order_queue_size
}

pub(super) fn blacksmith_input_hint(building: &Building, data: &GameData) -> String {
    let ore_needed = (data.balance.smith_batch_ore as f32 - building.stock(Good::Ore))
        .max(0.0)
        .ceil() as u32;
    if let Some(item) = building.orders.first() {
        if let Some(equipment) = data.equipment_def(item) {
            return format!("Needs {ore_needed} ore · next {}", equipment.name);
        }
    }
    format!("Needs {ore_needed} ore · next ingot")
}

pub(super) fn smelter_input_hint(building: &Building, data: &GameData) -> String {
    let ore_needed = (data.balance.smelt_batch_ore as f32 - building.stock(Good::Ore))
        .max(0.0)
        .ceil() as u32;
    let charcoal_needed = (data.balance.smelt_batch_charcoal - building.stock(Good::Charcoal))
        .max(0.0)
        .ceil() as u32;
    let mut missing = Vec::new();
    if ore_needed > 0 {
        missing.push(format!("{ore_needed} ore"));
    }
    if charcoal_needed > 0 {
        missing.push(format!("{charcoal_needed} charcoal"));
    }
    format!("Needs {}", missing.join(" + "))
}

pub(super) fn cook_pot_input_hint(building: &Building, data: &GameData) -> String {
    let batch = data.balance.cook_batch_mushrooms as f32 * data.balance.raw_recipe_multiplier;
    let mushrooms_needed = (batch - building.stock(Good::Mushroom)).max(0.0).ceil() as u32;
    format!("Needs {mushrooms_needed} mushrooms")
}

pub(super) fn kiln_input_hint() -> String {
    "Needs 1 wood".to_owned()
}

pub(super) fn local_smith_worker_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote()
        && (matches!(&creature.task, Task::Smithing { shop, .. } if *shop == pos)
            || matches!(&creature.task, Task::Crafting { shop, .. } if *shop == pos))
}

pub(super) fn local_smith_staffed_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote()
        && creature.job == Job::Smith
        && match &creature.task {
            Task::Smithing { shop, .. } | Task::Crafting { shop, .. } | Task::GoSmith(shop) => {
                *shop == pos
            }
            _ => creature.tile() == pos,
        }
}

pub(super) fn local_smelter_worker_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote() && matches!(&creature.task, Task::Smelting { den, .. } if *den == pos)
}

pub(super) fn local_smelter_staffed_at(creature: &Creature, pos: TilePos) -> bool {
    !creature.is_remote()
        && creature.job == Job::Smelter
        && match &creature.task {
            Task::Smelting { den, .. } | Task::GoSmelt(den) => *den == pos,
            _ => creature.tile() == pos,
        }
}
