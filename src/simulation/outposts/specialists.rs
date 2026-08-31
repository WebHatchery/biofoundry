//! Remote-route bonuses supplied by stationed Wormsong specialists.

use crate::data::GameData;
use crate::state::outposts::Outpost;
use crate::state::GameSession;

/// The live contribution of Wormsong kits currently stationed at one route.
///
/// The values are deliberately derived from `equipment.json`, so future kits
/// can join the route system without another balance table in Rust.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WormsongRouteBonus {
    pub storage_slots: u32,
    pub ore: u32,
    pub ingots: u32,
    pub cycle_reduction: f32,
    pub carriers: u32,
    pub miners: u32,
    pub smiths: u32,
    pub guards: u32,
}

impl WormsongRouteBonus {
    pub fn is_empty(self) -> bool {
        self.storage_slots == 0 && self.ore == 0 && self.ingots == 0 && self.cycle_reduction <= 0.0
    }
}

/// Sum the route-facing fields of the kits worn by this Outpost's crew.
pub fn wormsong_route_bonus(
    session: &GameSession,
    data: &GameData,
    outpost: &Outpost,
) -> WormsongRouteBonus {
    let mut bonus = WormsongRouteBonus::default();
    for creature_id in &outpost.crew {
        let Some(creature) = session.creatures.iter().find(|c| c.id == *creature_id) else {
            continue;
        };
        let Some(equipment_id) = creature.equipment.as_deref() else {
            continue;
        };
        let Some(equipment) = data.equipment_def(equipment_id) else {
            continue;
        };
        bonus.storage_slots = bonus
            .storage_slots
            .saturating_add(equipment.remote_storage_bonus);
        bonus.ore = bonus.ore.saturating_add(equipment.remote_ore_bonus);
        bonus.ingots = bonus.ingots.saturating_add(equipment.remote_ingot_bonus);
        bonus.cycle_reduction += equipment.remote_cycle_reduction.max(0.0);
        if equipment.remote_storage_bonus > 0 {
            bonus.carriers = bonus.carriers.saturating_add(1);
        }
        if equipment.remote_ore_bonus > 0 {
            bonus.miners = bonus.miners.saturating_add(1);
        }
        if equipment.remote_ingot_bonus > 0 {
            bonus.smiths = bonus.smiths.saturating_add(1);
        }
        if equipment.remote_cycle_reduction > 0.0 {
            bonus.guards = bonus.guards.saturating_add(1);
        }
    }
    bonus
}

/// Current hold capacity after applying stationed carrier kits.
pub fn route_storage_capacity(session: &GameSession, data: &GameData, outpost: &Outpost) -> u32 {
    crate::simulation::outposts::storage_capacity(outpost, data)
        .saturating_add(wormsong_route_bonus(session, data, outpost).storage_slots)
}

/// Current scouting cycle after applying the route's installed beacon and
/// stationed guard kits.
pub fn route_expedition_cycle_sec(
    session: &GameSession,
    data: &GameData,
    outpost: &Outpost,
) -> f32 {
    let base = crate::simulation::outposts::expedition_cycle_sec(outpost, data);
    (base - wormsong_route_bonus(session, data, outpost).cycle_reduction).max(0.5)
}

/// Total ore returned by the next haul, including Wormsong miner kits.
pub fn route_expedition_ore(session: &GameSession, data: &GameData, outpost: &Outpost) -> u32 {
    let crew = outpost.crew.len() as u32;
    crew.saturating_mul(crate::simulation::outposts::ore_per_crew(outpost, data))
        .saturating_add(wormsong_route_bonus(session, data, outpost).ore)
}

/// Total ingots returned by the next haul when a Signal Cache is installed.
pub fn route_signal_cache_ingots(session: &GameSession, data: &GameData, outpost: &Outpost) -> u32 {
    if !outpost.signal_cache_upgraded {
        return 0;
    }
    data.balance
        .outpost_signal_cache_ingots_per_haul
        .saturating_add(wormsong_route_bonus(session, data, outpost).ingots)
}

/// A compact explanation of the active remote effects for the route card.
pub fn route_bonus_summary(
    session: &GameSession,
    data: &GameData,
    outpost: &Outpost,
    compact: bool,
) -> Option<String> {
    let bonus = wormsong_route_bonus(session, data, outpost);
    if bonus.is_empty() {
        return None;
    }
    let mut effects = Vec::new();
    if bonus.storage_slots > 0 {
        effects.push(format!("hold +{}", bonus.storage_slots));
    }
    if bonus.ore > 0 {
        effects.push(format!("ore +{}", bonus.ore));
    }
    if bonus.ingots > 0 {
        let suffix = if outpost.signal_cache_upgraded {
            ""
        } else {
            " ready"
        };
        effects.push(format!("cache +{}{suffix}", bonus.ingots));
    }
    if bonus.cycle_reduction > 0.0 {
        effects.push(format!("cycle -{:.0}s", bonus.cycle_reduction));
    }
    if compact {
        let compact_effects = effects
            .iter()
            .map(|effect| effect.replace(" ", ""))
            .collect::<Vec<_>>()
            .join(" ");
        Some(format!("Wormsong · {compact_effects}"))
    } else {
        Some(format!("Wormsong crew · {}", effects.join(" · ")))
    }
}
