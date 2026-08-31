//! Fixed-step scouting work for active Worm Road routes.

use super::encore;
use super::{add_cargo, route_storage_capacity, take_cargo};
use super::{route_expedition_cycle_sec, route_expedition_ore, route_signal_cache_ingots};
use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::outposts::ExpeditionCompletion;
use crate::state::GameSession;

pub fn tick_expeditions(
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
) -> Vec<ExpeditionCompletion> {
    if !session.worm_awake {
        return Vec::new();
    }
    let food_per_crew = data.balance.outpost_expedition_food_per_crew;
    let mut completed = Vec::new();
    for index in 0..session.outposts.len() {
        let snapshot = session.outposts[index].clone();
        if !snapshot.active || snapshot.expedition_paused || snapshot.crew.is_empty() {
            continue;
        }
        let cycle = route_expedition_cycle_sec(session, data, &snapshot);
        let crew = snapshot.crew.len() as u32;
        let room =
            route_storage_capacity(session, data, &snapshot).saturating_sub(snapshot.cargo_total());
        let food_cost = crew.saturating_mul(food_per_crew);
        let food_available = snapshot.cargo.get(&Good::CookedFood).copied().unwrap_or(0);
        if room == 0 || food_available < food_cost {
            continue;
        }
        let ore = route_expedition_ore(session, data, &snapshot).min(room);
        let remaining_room = room.saturating_sub(ore);
        let ingots = if snapshot.signal_cache_upgraded {
            route_signal_cache_ingots(session, data, &snapshot).min(remaining_room)
        } else {
            0
        };
        let outpost = &mut session.outposts[index];
        outpost.expedition_progress = (outpost.expedition_progress.max(0.0) + dt).min(cycle);
        if outpost.expedition_progress < cycle {
            continue;
        }
        take_cargo(outpost, Good::CookedFood, food_cost);
        add_cargo(outpost, Good::Ore, ore);
        add_cargo(outpost, Good::Ingot, ingots);
        outpost.expedition_progress -= cycle;
        outpost.expeditions_completed = outpost.expeditions_completed.saturating_add(1);
        outpost.ore_scouted = outpost.ore_scouted.saturating_add(ore);
        outpost.signal_cache_ingots = outpost.signal_cache_ingots.saturating_add(ingots);
        completed.push(ExpeditionCompletion {
            outpost: outpost.pos,
            ore,
            ingots,
            food_spent: food_cost,
        });
        encore::record_concord_haul(session, data, &snapshot);
    }
    completed
}
