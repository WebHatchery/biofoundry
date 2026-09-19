//! Local material buffers and deterministic, reachable supply routes.

use crate::data::{GameData, StorageDef};
use crate::simulation::nav;
use crate::state::creatures::Good;
use crate::state::structures::Building;
use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;

pub fn definition<'a>(building: &Building, data: &'a GameData) -> Option<&'a StorageDef> {
    data.buildings.get(&building.kind)?.storage.as_ref()
}

pub fn free_space(building: &Building, data: &GameData) -> u32 {
    definition(building, data)
        .map(|def| {
            (def.capacity as f32 - building.stocks.values().sum::<f32>())
                .max(0.0)
                .floor() as u32
        })
        .unwrap_or(0)
}

pub fn cycle_filter(session: &mut GameSession, data: &GameData, pos: TilePos) -> bool {
    let Some(building) = session.building_at_mut(pos) else {
        return false;
    };
    if definition(building, data).is_none() || building.stocks.values().sum::<f32>() >= 1.0 {
        return false;
    }
    // Spoilage can leave an unhaulable fraction forever. Reconfiguration
    // sweeps those scraps into waste rather than silently deleting them.
    let scraps = building.stocks.values().sum::<f32>();
    building.waste += scraps;
    building.stocks.clear();
    building.storage_good = Some(match building.accepted_good() {
        Good::Mushroom => Good::Wood,
        Good::Wood => Good::Charcoal,
        _ => Good::Mushroom,
    });
    session.progress.waste_generated += scraps;
    session.economy.waste += scraps;
    session.economy.raw_food = session
        .buildings
        .iter()
        .map(|b| b.stock(Good::Mushroom))
        .sum();
    true
}

pub fn consumer_kind(good: Good) -> Option<&'static str> {
    match good {
        Good::Mushroom => Some("cook_pot"),
        Good::Wood => Some("kiln"),
        Good::Charcoal => Some("smelter"),
        _ => None,
    }
}

fn connected(session: &GameSession, from: TilePos, to: TilePos) -> bool {
    nav::find_path(session, from, to).is_some()
}

pub fn serves(session: &GameSession, data: &GameData, store: &Building, target: TilePos) -> bool {
    definition(store, data).is_some_and(|def| {
        store.pos.manhattan_distance(&target) as u32 <= def.supply_radius
            && connected(session, store.pos, target)
    })
}

/// Only fill buffers that can supply a matching production building.
pub fn deposit_destination(
    session: &GameSession,
    data: &GameData,
    from: TilePos,
    good: Good,
) -> Option<TilePos> {
    let kind = consumer_kind(good)?;
    session
        .buildings
        .iter()
        .filter(|store| {
            definition(store, data).is_some()
                && store.accepted_good() == good
                && free_space(store, data) > 0
                && connected(session, from, store.pos)
                && session
                    .buildings_of(kind)
                    .any(|target| serves(session, data, store, target.pos))
        })
        .map(|store| store.pos)
        .min_by_key(|p| (p.manhattan_distance(&from), p.x, p.y))
}

pub fn source_for(
    session: &GameSession,
    data: &GameData,
    target: TilePos,
    good: Good,
) -> Option<TilePos> {
    session
        .buildings
        .iter()
        .filter(|store| {
            definition(store, data).is_some()
                && store.accepted_good() == good
                && store.stock(good) >= 1.0
                && serves(session, data, store, target)
        })
        .map(|store| store.pos)
        .min_by_key(|p| (p.manhattan_distance(&target), p.x, p.y))
}

pub fn consumer_for(
    session: &GameSession,
    data: &GameData,
    source: TilePos,
    good: Good,
) -> Option<TilePos> {
    let store = session.building_at(source)?;
    let kind = consumer_kind(good)?;
    let target_stock = match good {
        Good::Wood => data.balance.kiln_wood_cap,
        Good::Charcoal => data.balance.smelt_batch_charcoal,
        _ => return None,
    };
    session
        .buildings_of(kind)
        .filter(|target| {
            target.stock(good) < target_stock && serves(session, data, store, target.pos)
        })
        .map(|target| target.pos)
        .min_by_key(|p| (p.manhattan_distance(&source), p.x, p.y))
}
