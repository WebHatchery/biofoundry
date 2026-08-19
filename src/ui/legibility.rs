//! Factory legibility: derive the at-a-glance status of a workstation and
//! the warren's pending-haul pressure, so a stalled chain link is
//! diagnosable without clicking anything (plan §Phase 9).

use crate::data::GameData;
use crate::simulation::wildlife;
use crate::state::creatures::{Creature, Good, Job, Task};
use crate::state::structures::Building;
use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;

/// A worker's species × Overseer-aura work multiplier, for display (the sim
/// applies the same factor via `jobs::overseer_aura`). Equipment folds in
/// separately at each work site.
pub fn work_multiplier(creature: &Creature, session: &GameSession, data: &GameData) -> f32 {
    let species_mult = data
        .species
        .get(&creature.species)
        .map(|s| s.work_mult)
        .unwrap_or(1.0);
    let r2 = data.balance.overseer_aura_radius * data.balance.overseer_aura_radius;
    let in_aura = session.creatures.iter().any(|o| {
        o.species == "overseer" && {
            let dx = o.x - creature.x;
            let dy = o.y - creature.y;
            dx * dx + dy * dy <= r2
        }
    });
    species_mult
        * if in_aura {
            data.balance.overseer_aura_mult
        } else {
            1.0
        }
}

/// What's wrong with a workstation right now — the in-world status icon.
/// `None` (from [`building_status`]) means the node is running nominally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingStatus {
    /// A workstation with no creature working it (stopped node).
    NoWorker,
    /// Waiting on input goods it can't get (starved).
    InputStarved,
    /// Output buffer is full and backing up — needs a carrier.
    OutputFull,
    /// A Mine whose deposit has run dry.
    Exhausted,
    /// A Farm sitting at its storage cap, idle until a carrier drains it.
    AwaitingHaul,
    /// Spoiled stores are accumulating faster than they are cleaned.
    WasteOverflow,
}

impl BuildingStatus {
    /// A short human label (also the legend text).
    pub fn label(self) -> &'static str {
        match self {
            BuildingStatus::NoWorker => "No worker",
            BuildingStatus::InputStarved => "Starved",
            BuildingStatus::OutputFull => "Backed up",
            BuildingStatus::Exhausted => "Exhausted",
            BuildingStatus::AwaitingHaul => "Awaiting haul",
            BuildingStatus::WasteOverflow => "Waste accumulating",
        }
    }
}

/// Is any creature of `job` currently working (or waiting) at `pos`?
fn staffed_at(session: &GameSession, pos: TilePos, job: Job) -> bool {
    session.creatures.iter().any(|c| {
        c.job == job
            && match &c.task {
                Task::WorkMine(p) | Task::GoMine(p) => *p == pos,
                Task::Smithing { shop, .. } | Task::Crafting { shop, .. } | Task::GoSmith(shop) => {
                    *shop == pos
                }
                Task::Smelting { den, .. } | Task::GoSmelt(den) => *den == pos,
                // A creature idling on the tile also counts as manning it.
                _ => c.tile() == pos,
            }
    })
}

/// The status icon to show over `building`, or `None` when it's nominal.
pub fn building_status(
    session: &GameSession,
    data: &GameData,
    building: &Building,
) -> Option<BuildingStatus> {
    let pos = building.pos;
    match building.kind.as_str() {
        "mine" => {
            if building.reserve <= 0.0 {
                return Some(BuildingStatus::Exhausted);
            }
            if !staffed_at(session, pos, Job::Miner) {
                return Some(BuildingStatus::NoWorker);
            }
            if building.stock(Good::Ore) >= data.balance.mine_buffer_cap - 0.5 {
                return Some(BuildingStatus::OutputFull);
            }
            None
        }
        "blacksmith" => {
            if !staffed_at(session, pos, Job::Smith) {
                return Some(BuildingStatus::NoWorker);
            }
            // Idle for lack of ore, and nothing queued to justify waiting.
            if building.stock(Good::Ore) < data.balance.smith_batch_ore as f32
                && building.orders.is_empty()
            {
                return Some(BuildingStatus::InputStarved);
            }
            None
        }
        "smelter" => {
            if !staffed_at(session, pos, Job::Smelter) {
                return Some(BuildingStatus::NoWorker);
            }
            if building.stock(Good::Ore) < data.balance.smelt_batch_ore as f32
                || building.stock(Good::Charcoal) < data.balance.smelt_batch_charcoal
            {
                return Some(BuildingStatus::InputStarved);
            }
            None
        }
        "farm" => {
            let cap = wildlife::farm_cap(session, data);
            if building.stock(Good::Mushroom) >= cap - 0.5 {
                return Some(BuildingStatus::AwaitingHaul);
            }
            None
        }
        "cook_pot" => {
            if building.stock(Good::Mushroom)
                < data.balance.cook_batch_mushrooms as f32 * data.balance.raw_recipe_multiplier
            {
                return Some(BuildingStatus::InputStarved);
            }
            None
        }
        "kiln" => {
            if building.stock(Good::Wood) <= 0.0 {
                return Some(BuildingStatus::InputStarved);
            }
            None
        }
        "feeding_trough" if building.waste > 0.0 => Some(BuildingStatus::WasteOverflow),
        "outpost"
            if session
                .outposts
                .iter()
                .find(|o| o.pos == pos)
                .is_some_and(|o| !o.active) =>
        {
            Some(BuildingStatus::InputStarved)
        }
        _ => None,
    }
}

/// Rough count of pending haul jobs — pickup points holding goods that want
/// moving, plus open construction. Turns "should I add a carrier?" into a
/// read instead of a guess.
pub fn pending_hauls(session: &GameSession) -> usize {
    let mut n = 0;
    for b in &session.buildings {
        match b.kind.as_str() {
            "mine" if b.stock(Good::Ore) >= 1.0 => n += 1,
            "farm" if b.stock(Good::Mushroom) >= 1.0 => n += 1,
            "smelter" if b.stock(Good::Ingot) >= 1.0 => n += 1,
            "blacksmith" if b.stock(Good::Ingot) >= 1.0 && b.orders.is_empty() => n += 1,
            _ => {}
        }
    }
    // Open construction is haul demand too.
    n += session
        .build_sites
        .iter()
        .filter(|s| s.remaining() > 0)
        .count();
    n
}

#[cfg(test)]
mod tests;
