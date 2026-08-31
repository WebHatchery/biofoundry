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

    pub fn has_all_roles(self) -> bool {
        self.carriers > 0 && self.miners > 0 && self.smiths > 0 && self.guards > 0
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

/// Whether this active route has the complete four-role crew after the
/// one-time Concord has been claimed.
pub fn route_concord_active(session: &GameSession, data: &GameData, outpost: &Outpost) -> bool {
    session.outpost_concord_claimed
        && outpost.active
        && wormsong_route_bonus(session, data, outpost).has_all_roles()
}

/// The ongoing route-wide benefit of keeping all four Wormsong roles together.
pub fn concord_route_bonus(
    session: &GameSession,
    data: &GameData,
    outpost: &Outpost,
) -> (u32, f32) {
    if route_concord_active(session, data, outpost) {
        (
            data.balance.outpost_concord_ore_bonus,
            data.balance.outpost_concord_cycle_reduction,
        )
    } else {
        (0, 0.0)
    }
}

/// Current scouting cycle after applying the route's installed beacon and
/// stationed guard kits.
pub fn route_expedition_cycle_sec(
    session: &GameSession,
    data: &GameData,
    outpost: &Outpost,
) -> f32 {
    let base = crate::simulation::outposts::expedition_cycle_sec(outpost, data);
    let specialist_reduction = wormsong_route_bonus(session, data, outpost).cycle_reduction;
    let concord_reduction = concord_route_bonus(session, data, outpost).1;
    (base - specialist_reduction - concord_reduction).max(0.5)
}

/// Total ore returned by the next haul, including Wormsong miner kits.
pub fn route_expedition_ore(session: &GameSession, data: &GameData, outpost: &Outpost) -> u32 {
    let crew = outpost.crew.len() as u32;
    let concord_ore = concord_route_bonus(session, data, outpost).0;
    crew.saturating_mul(crate::simulation::outposts::ore_per_crew(outpost, data))
        .saturating_add(wormsong_route_bonus(session, data, outpost).ore)
        .saturating_add(concord_ore)
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

/// Count active routes that currently carry every Wormsong voice.
pub fn complete_wormsong_route_count(session: &GameSession, data: &GameData) -> u32 {
    session
        .outposts
        .iter()
        .filter(|outpost| {
            outpost.active && wormsong_route_bonus(session, data, outpost).has_all_roles()
        })
        .count() as u32
}

/// Whether this route is part of a currently sustained three-route Chorus.
/// The reward is deliberately live: withdrawing one voice pauses the extra
/// haul until the network is whole again.
pub fn chorus_route_active(session: &GameSession, data: &GameData, outpost: &Outpost) -> bool {
    session.outpost_chorus_claimed
        && data.balance.outpost_chorus_route_goal > 0
        && complete_wormsong_route_count(session, data) >= data.balance.outpost_chorus_route_goal
        && outpost.active
        && wormsong_route_bonus(session, data, outpost).has_all_roles()
}

/// Total Chorus ingots returned by this route's next haul.
pub fn route_chorus_ingots(session: &GameSession, data: &GameData, outpost: &Outpost) -> u32 {
    if chorus_route_active(session, data, outpost) {
        data.balance.outpost_chorus_ingots_per_haul
    } else {
        0
    }
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
    let (concord_ore, concord_cycle) = concord_route_bonus(session, data, outpost);
    if concord_ore > 0 || concord_cycle > 0.0 {
        effects.push(format!(
            "concord +{} ore/-{:.0}s",
            concord_ore, concord_cycle
        ));
    }
    let chorus_ingots = route_chorus_ingots(session, data, outpost);
    if chorus_ingots > 0 {
        effects.push(format!("chorus +{} ingot/haul", chorus_ingots));
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
