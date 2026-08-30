//! Shape and value checks for sessions before they cross into the live game.

use crate::data::GameData;
use crate::state::creatures::{Good, Task};
use crate::state::outposts::Outpost;
use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;
use std::collections::HashSet;

pub(super) fn validate_outpost_cargo(outpost: &Outpost, data: &GameData) -> Result<(), String> {
    for good in outpost.cargo.keys() {
        if !matches!(good, Good::Ore | Good::Ingot | Good::CookedFood) {
            return Err(format!(
                "outpost contains unsupported cargo {good:?} at {:?}",
                outpost.pos
            ));
        }
    }
    let cargo_total = outpost
        .cargo
        .values()
        .try_fold(0u32, |total, amount| total.checked_add(*amount))
        .ok_or_else(|| format!("outpost cargo total overflows at {:?}", outpost.pos))?;
    let capacity = crate::simulation::outposts::storage_capacity(outpost, data);
    if cargo_total > capacity {
        return Err(format!(
            "outpost cargo exceeds its {capacity}-slot hold at {:?}",
            outpost.pos
        ));
    }
    Ok(())
}

pub(super) fn validate_nonnegative_finite(value: f32, field: &str) -> Result<(), String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!("{field} is not a finite non-negative value"));
    }
    Ok(())
}

pub(super) fn validate_map_position(
    pos: TilePos,
    width: usize,
    height: usize,
    field: &str,
) -> Result<(), String> {
    if !pos.in_bounds(width, height) {
        return Err(format!("{field} is outside the map at {pos:?}"));
    }
    Ok(())
}

pub(super) fn validate_walkable_position(
    session: &GameSession,
    pos: TilePos,
    field: &str,
) -> Result<(), String> {
    validate_map_position(
        pos,
        session.world.tiles.width,
        session.world.tiles.height,
        field,
    )?;
    if !session
        .world
        .tiles
        .get(pos)
        .is_some_and(|tile| tile.walkable())
    {
        return Err(format!("{field} is not on walkable floor at {pos:?}"));
    }
    Ok(())
}

pub(super) fn validate_map_timer(
    pos: TilePos,
    remaining: f32,
    width: usize,
    height: usize,
    field: &str,
) -> Result<(), String> {
    validate_map_position(pos, width, height, field)?;
    validate_nonnegative_finite(remaining, field)
}

pub(super) fn validate_actor_position(
    session: &GameSession,
    x: f32,
    y: f32,
    field: &str,
) -> Result<(), String> {
    if !x.is_finite()
        || !y.is_finite()
        || x < 0.0
        || y < 0.0
        || x >= session.world.tiles.width as f32
        || y >= session.world.tiles.height as f32
    {
        return Err(format!("{field} position is outside the map"));
    }
    Ok(())
}

pub(super) fn validate_unique_ids(
    ids: impl IntoIterator<Item = u32>,
    next_id: u32,
    kind: &str,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    let mut max_id = 0;
    for id in ids {
        if id == 0 || !seen.insert(id) {
            return Err(format!("{kind} ids are missing or duplicated"));
        }
        max_id = max_id.max(id);
    }
    if next_id == 0 || next_id <= max_id {
        return Err(format!("next {kind} id does not follow the roster"));
    }
    Ok(())
}

pub(super) fn validate_task_positions(
    session: &GameSession,
    task: &Task,
    path: &[TilePos],
) -> Result<(), String> {
    for path_pos in path {
        validate_map_position(
            *path_pos,
            session.world.tiles.width,
            session.world.tiles.height,
            "creature path",
        )?;
    }
    let task_pos = match task {
        Task::GoMine(pos)
        | Task::WorkMine(pos)
        | Task::GoFetch(pos)
        | Task::GoDig(pos)
        | Task::DeliverTo(pos)
        | Task::GoCook(pos)
        | Task::GoSmelt(pos)
        | Task::GoSmith(pos)
        | Task::GoClean(pos) => Some(*pos),
        Task::Cooking { pot, .. } => Some(*pot),
        Task::Smelting { den, .. } => Some(*den),
        Task::Smithing { shop, .. } => Some(*shop),
        Task::Cleaning { building, .. } => Some(*building),
        Task::Digging { mark, .. } => Some(*mark),
        Task::Fetching { source, .. } => Some(*source),
        Task::Feeding { trough, .. } => Some(*trough),
        Task::Crafting { shop, .. } => Some(*shop),
        Task::Hunt { .. }
        | Task::Idle
        | Task::DeliverOre
        | Task::DeliverIngot
        | Task::GoPickupOre
        | Task::PickingUpOre { .. }
        | Task::GoEquip => None,
    };
    if let Some(pos) = task_pos {
        validate_map_position(
            pos,
            session.world.tiles.width,
            session.world.tiles.height,
            "creature task",
        )?;
    }
    Ok(())
}

pub(super) fn validate_wild_behavior(
    session: &GameSession,
    behavior: &crate::state::wildlife::WildBehavior,
) -> Result<(), String> {
    match behavior {
        crate::state::wildlife::WildBehavior::Wander { next_move_in } => {
            validate_nonnegative_finite(*next_move_in, "wild movement timer")?;
        }
        crate::state::wildlife::WildBehavior::Raid { origin, eaten, .. } => {
            validate_map_position(
                *origin,
                session.world.tiles.width,
                session.world.tiles.height,
                "raid origin",
            )?;
            validate_nonnegative_finite(*eaten, "raid food eaten")?;
        }
    }
    Ok(())
}
