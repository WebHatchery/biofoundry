//! Factory legibility: derive the at-a-glance status of a workstation and
//! the warren's pending-haul pressure, so a stalled chain link is
//! diagnosable without clicking anything (plan §Phase 9).

use crate::data::GameData;
use crate::simulation::nav;
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
        !o.is_remote() && o.species == "overseer" && {
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
    /// A legacy node whose floor is no longer connected to the stockpile.
    NoValidRoute,
    /// Waiting on input goods it can't get (starved).
    InputStarved,
    /// Output buffer is full and backing up — needs a carrier.
    OutputFull,
    /// A Mine whose deposit has run dry.
    Exhausted,
    /// A Farm sitting at its storage cap, idle until a carrier drains it.
    AwaitingHaul,
    /// A remote route that is turned off and needs activation before use.
    RouteInactive,
    /// An awakened remote route with no crew stationed to scout.
    ExpeditionNoCrew,
    /// An awakened remote route whose scouting has been paused by the player.
    ExpeditionPaused,
    /// A staffed remote route that needs more cooked food before scouting.
    ExpeditionNeedsFood,
    /// A remote cargo hold that needs a return trip before scouting can continue.
    ExpeditionHoldFull,
    /// Worm Shrine offerings were paused by the player.
    ShrineOfferingsPaused,
    /// Worm Shrine offerings are waiting for the food reserve.
    ShrineNeedsFood,
    /// Worm Shrine offerings are waiting for the ingot reserve.
    ShrineNeedsIngots,
    /// Spoiled stores are accumulating faster than they are cleaned.
    WasteOverflow,
}

/// Whether the post-campaign specialist and advanced-building controls are
/// available in the Jobs and Build panels.
pub fn advanced_systems_unlocked(session: &GameSession) -> bool {
    (session.won && session.job_count(Job::Guard) > 0) || session.worm_awake
}

impl BuildingStatus {
    /// A short human label (also the legend text).
    pub fn label(self) -> &'static str {
        match self {
            BuildingStatus::NoWorker => "No worker",
            BuildingStatus::NoValidRoute => "No valid route",
            BuildingStatus::InputStarved => "Starved",
            BuildingStatus::OutputFull => "Backed up",
            BuildingStatus::Exhausted => "Exhausted",
            BuildingStatus::AwaitingHaul => "Awaiting haul",
            BuildingStatus::RouteInactive => "Route inactive",
            BuildingStatus::ExpeditionNoCrew => "No scout crew",
            BuildingStatus::ExpeditionPaused => "Scouting paused",
            BuildingStatus::ExpeditionNeedsFood => "Scout food low",
            BuildingStatus::ExpeditionHoldFull => "Outpost full",
            BuildingStatus::ShrineOfferingsPaused => "Offerings paused",
            BuildingStatus::ShrineNeedsFood => "Food reserve low",
            BuildingStatus::ShrineNeedsIngots => "Ingot reserve low",
            BuildingStatus::WasteOverflow => "Waste accumulating",
        }
    }
}

/// Is any creature of `job` currently working (or waiting) at `pos`?
fn staffed_at(session: &GameSession, pos: TilePos, job: Job) -> bool {
    session.creatures.iter().any(|c| {
        !c.is_remote()
            && (c.job == job || (job == Job::Miner && c.job == Job::Engineer))
            && match &c.task {
                Task::WorkMine(p) | Task::GoMine(p) => *p == pos,
                Task::Smithing { shop, .. } | Task::Crafting { shop, .. } | Task::GoSmith(shop) => {
                    *shop == pos
                }
                Task::Smelting { den, .. } | Task::GoSmelt(den) => *den == pos,
                Task::Cooking { pot, .. } | Task::GoCook(pot) => *pot == pos,
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
    if building.waste > 0.0 {
        return Some(BuildingStatus::WasteOverflow);
    }
    if requires_local_route(&building.kind)
        && nav::find_path(session, session.stockpile_pos(), building.pos).is_none()
    {
        return Some(BuildingStatus::NoValidRoute);
    }
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
            // A queued order is only actionable immediately when its ingot
            // cost is already paid. Otherwise the smith needs an ore batch
            // to forge the missing ingots; expose that stall instead of
            // calling an empty anvil nominally "Working".
            if blacksmith_needs_ore(building, data) {
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
            if !staffed_at(session, pos, Job::Cook) {
                return Some(BuildingStatus::NoWorker);
            }
            let batch = (data.balance.cook_batch_mushrooms as f32
                * data.balance.raw_recipe_multiplier)
                .ceil();
            if building.stock(Good::Mushroom) < batch {
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
        "outpost"
            if session
                .outposts
                .iter()
                .find(|o| o.pos == pos)
                .is_some_and(|o| !o.active) =>
        {
            Some(BuildingStatus::RouteInactive)
        }
        "outpost" => {
            let outpost = session.outposts.iter().find(|o| o.pos == pos)?;
            if !session.worm_awake {
                return None;
            }
            match crate::simulation::outposts::expedition_state(outpost, data) {
                crate::simulation::outposts::ExpeditionState::Paused => {
                    Some(BuildingStatus::ExpeditionPaused)
                }
                crate::simulation::outposts::ExpeditionState::NoCrew => {
                    Some(BuildingStatus::ExpeditionNoCrew)
                }
                crate::simulation::outposts::ExpeditionState::NeedsFood { .. } => {
                    Some(BuildingStatus::ExpeditionNeedsFood)
                }
                crate::simulation::outposts::ExpeditionState::HoldFull => {
                    Some(BuildingStatus::ExpeditionHoldFull)
                }
                crate::simulation::outposts::ExpeditionState::Inactive
                | crate::simulation::outposts::ExpeditionState::Scouting { .. } => None,
            }
        }
        "worm_shrine" if session.worm_awake => None,
        "worm_shrine" if session.worm_feeding_paused => Some(BuildingStatus::ShrineOfferingsPaused),
        "worm_shrine" if shrine_waiting_for_food(session, data) => {
            Some(BuildingStatus::ShrineNeedsFood)
        }
        "worm_shrine" if shrine_waiting_for_ingots(session, data) => {
            Some(BuildingStatus::ShrineNeedsIngots)
        }
        _ => None,
    }
}

fn requires_local_route(kind: &str) -> bool {
    matches!(
        kind,
        "farm" | "mine" | "cook_pot" | "blacksmith" | "kiln" | "smelter" | "feeding_trough"
    )
}

/// Whether the Shrine is below the food reserve needed to keep its offerings
/// running. Kept beside the map status so the map badge and inspection card
/// can share the same blocker definition.
pub(crate) fn shrine_waiting_for_food(session: &GameSession, data: &GameData) -> bool {
    session.worm_fed < data.balance.worm_awaken_at
        && session.economy.food <= data.balance.worm_feed_reserve
}

/// Whether the Shrine has earned another food offering but cannot spend the
/// next ingot without dipping below the protected bank reserve.
pub(crate) fn shrine_waiting_for_ingots(session: &GameSession, data: &GameData) -> bool {
    if session.worm_fed >= data.balance.worm_awaken_at {
        return session.worm_ingots_fed < data.balance.worm_awaken_ingots
            && session.economy.ingots_stock <= data.balance.worm_ingot_reserve;
    }
    let food_per_offering = if data.balance.worm_food_per_offering > 0.0 {
        data.balance.worm_food_per_offering
    } else {
        data.balance.worm_awaken_at / data.balance.worm_awaken_ingots.max(1) as f32
    };
    let completed_offerings = (session.worm_fed / food_per_offering.max(1.0)).floor() as u32;
    let desired_ingots = completed_offerings * data.balance.worm_ingots_per_offering;
    session.worm_ingots_fed < desired_ingots
        && session.economy.ingots_stock <= data.balance.worm_ingot_reserve
}

fn blacksmith_needs_ore(building: &Building, data: &GameData) -> bool {
    let has_ore = building.stock(Good::Ore) >= data.balance.smith_batch_ore as f32;
    if has_ore {
        return false;
    }
    let order_paid = building.orders.first().and_then(|item| {
        data.equipment_def(item)
            .map(|equipment| building.stock(Good::Ingot) >= equipment.cost_ingots as f32)
    });
    building.orders.is_empty() || !order_paid.unwrap_or(false)
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
