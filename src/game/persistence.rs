//! Save, load, migration, and frame-state recovery for the game shell.

use super::Game;
use crate::data::GameData;
use crate::state::creatures::{Job, Task};
use crate::state::{GameSession, GameState};
use crate::ui::UiMode;
use macroquad_toolkit::notifications::{LoggedNotification, NotificationManager, MAX_HISTORY};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, quarantine_slot, restore_slot_backup,
    save_to_slot_with_version_and_backup, slot_backup_exists, slot_exists,
};
use std::collections::HashSet;

impl Game {
    pub(super) fn save_game(&mut self) {
        let failure_notice = match &self.state {
            GameState::Warren(session) => non_viable_save_notice(session, &self.data),
            _ => None,
        };
        if let Some(notice) = failure_notice {
            self.notifications.warning(notice);
            return;
        }
        self.persist_or_report_save();
    }

    fn persist_or_report_save(&mut self) {
        let had_existing_save = self.save_exists;
        match self.persist_current_session() {
            Ok(()) => {
                self.save_exists = true;
                self.notifications.success("Warren saved.");
            }
            Err(err) => {
                self.notifications
                    .danger(save_failure_notice(false, had_existing_save, &err))
            }
        }
    }

    /// Persist a campaign milestone or direct player decision without
    /// interrupting the player's flow.
    pub(super) fn autosave_game(&mut self) {
        if matches!(&self.state, GameState::Warren(session) if session.is_non_viable(&self.data)) {
            return;
        }
        let had_existing_save = self.save_exists;
        match self.persist_current_session() {
            Ok(()) => {
                self.save_exists = true;
            }
            Err(err) => {
                self.notifications
                    .warning(save_failure_notice(true, had_existing_save, &err))
            }
        }
    }

    fn persist_current_session(&mut self) -> Result<(), String> {
        let history = self.notifications.history().to_vec();
        let GameState::Warren(session) = &mut self.state else {
            return Err("no active Warren".to_owned());
        };
        session.event_history = history;
        let config = &self.data.config;
        save_to_slot_with_version_and_backup(
            &config.game_name,
            &config.save_slot,
            session.as_ref(),
            &config.version,
        )
    }

    pub(super) fn load_game(&mut self) {
        let slot = self.data.config.save_slot.clone();
        match self.load_session_from_slot(&slot) {
            Ok(session) => {
                self.install_loaded_session(session);
                // The slot may have become available after startup (or after
                // a prior failed recovery), so a successful load must restore
                // the title screen's Continue affordance as well.
                self.save_exists = true;
                self.notifications.success("Warren loaded.");
            }
            Err(err) => self.recover_failed_load(&slot, err),
        }
    }

    fn load_session_from_slot(&self, slot: &str) -> Result<GameSession, String> {
        let config = &self.data.config;
        load_from_slot_with_migration(
            &config.game_name,
            slot,
            &config.version,
            |version, value| {
                let payload = value.get("data").cloned().unwrap_or(value);
                let mut session: GameSession = serde_json::from_value(payload)
                    .map_err(|err| format!("Unsupported save {version:?}: {err}"))?;
                migrate_tutorial_progress(&mut session, self.data.tutorial.len());
                validate_loaded_session(&session, &self.data)?;
                Ok(session)
            },
        )
    }

    fn install_loaded_session(&mut self, session: GameSession) {
        let mut session = session;
        restore_notification_history(&mut self.notifications, &session.event_history);
        session.event_history = self.notifications.history().to_vec();
        session.sync_remote_crew_state();
        crate::simulation::outposts::sync_transit_failure_banner(&mut session);
        self.reset_camera_for(&session);
        self.reset_session_view_state();
        self.state = GameState::Warren(Box::new(session));
    }

    /// Clear frame-local controls when a campaign crosses the title boundary.
    /// These values belong to the previous view, not to the persisted warren.
    pub(super) fn reset_session_view_state(&mut self) {
        self.accumulator = 0.0;
        self.famine_announced = false;
        self.mode = UiMode::Inspect;
        self.help_open = false;
        self.event_log_open = false;
        self.event_log_page = 0;
        self.routes_open = false;
        self.paused = false;
        self.confirm_new_warren = false;
        self.selected_building = None;
        self.mouse_pan_start = None;
        self.mouse_camera_claimed = false;
        self.camera_input_claimed = false;
        self.touch_camera_claimed = false;
        self.touch_tap = None;
    }

    fn recover_failed_load(&mut self, slot: &str, error: String) {
        let config = &self.data.config;
        let primary_exists = slot_exists(&config.game_name, slot);
        let backup_slot = format!("{slot}_backup");
        let backup_exists = slot_backup_exists(&config.game_name, slot);
        if should_restore_missing_primary(primary_exists, backup_exists) {
            self.recover_missing_primary(slot, &backup_slot);
            return;
        }
        if !primary_exists {
            if no_saved_slot_available(primary_exists, backup_exists) {
                self.save_exists = false;
                self.notifications.danger(missing_save_notice(&error));
            } else {
                self.notifications.warning(format!("Load failed: {error}"));
            }
            return;
        }

        let quarantine = match quarantine_slot(&config.game_name, slot) {
            Ok(name) => name,
            Err(quarantine_error) => {
                self.notifications.danger(format!(
                    "Save could not load ({error}). The original was left untouched; repair it or use Menu → New Warren. ({quarantine_error})"
                ));
                return;
            }
        };

        if !backup_exists {
            self.save_exists = false;
            self.notifications.danger(format!(
                "Save is damaged; preserved it as {quarantine}. Use Menu → New Warren or repair it before loading again."
            ));
            return;
        }

        match self.load_session_from_slot(&backup_slot) {
            Ok(session) => match restore_slot_backup(&config.game_name, slot) {
                Ok(_) => {
                    self.install_loaded_session(session);
                    self.save_exists = true;
                    self.notifications.warning(format!(
                        "Primary save was damaged; preserved it as {quarantine} and restored the previous safe save."
                    ));
                }
                Err(restore_error) => {
                    self.install_loaded_session(session);
                    self.save_exists = false;
                    self.notifications.warning(format!(
                        "Primary save was damaged; loaded the safe backup, but could not restore it ({restore_error}). Use Save now."
                    ));
                }
            },
            Err(backup_error) => {
                self.save_exists = false;
                self.notifications.danger(format!(
                    "Save is damaged; preserved it as {quarantine}. The safe backup also failed ({backup_error}). Use Menu → New Warren or repair the preserved saves."
                ));
            }
        }
    }

    /// Recover the last safe copy when the primary slot vanished before the
    /// load boundary. `restore_slot_backup` creates a new primary in this
    /// branch, and the loaded session remains available even if that write is
    /// rejected by a full or blocked storage backend.
    fn recover_missing_primary(&mut self, slot: &str, backup_slot: &str) {
        let config = &self.data.config;
        match self.load_session_from_slot(backup_slot) {
            Ok(session) => match restore_slot_backup(&config.game_name, slot) {
                Ok(_) => {
                    self.install_loaded_session(session);
                    self.save_exists = true;
                    self.notifications
                        .warning("Primary save was missing; restored the previous safe save.");
                }
                Err(restore_error) => {
                    self.install_loaded_session(session);
                    self.save_exists = false;
                    self.notifications.warning(format!(
                        "Primary save was missing; loaded the safe backup, but could not restore it ({restore_error}). Use Save now."
                    ));
                }
            },
            Err(backup_error) => {
                self.save_exists = false;
                self.notifications.danger(format!(
                    "Primary save is missing; the safe backup also failed ({backup_error}). Use Menu → New Warren or repair the backup."
                ));
            }
        }
    }
}

pub(super) fn should_restore_missing_primary(primary_exists: bool, backup_exists: bool) -> bool {
    !primary_exists && backup_exists
}

pub(super) fn no_saved_slot_available(primary_exists: bool, backup_exists: bool) -> bool {
    !primary_exists && !backup_exists
}

pub(super) fn missing_save_notice(error: &str) -> String {
    format!("Load failed — no saved warren is available: {error}. Use New Warren to begin again.")
}

pub(super) fn save_failure_notice(autosave: bool, had_existing_save: bool, error: &str) -> String {
    let recovery = match (autosave, had_existing_save) {
        (true, true) => "previous save remains available; use Save to retry",
        (true, false) => "use Save to create a checkpoint",
        (false, true) => "previous save remains available",
        (false, false) => "no new save was written",
    };
    let action = if autosave {
        "Autosave failed"
    } else {
        "Save failed"
    };
    format!("{action} — {recovery}: {error}")
}

/// Rehydrate only the manager's non-timed history. Loading a save should not
/// replay every old toast over the Warren, but the Field Guide must still be
/// able to review those messages after a refresh or relaunch.
pub(super) fn restore_notification_history(
    notifications: &mut NotificationManager,
    history: &[LoggedNotification],
) {
    notifications.clear();
    notifications.clear_history();
    let first = history.len().saturating_sub(MAX_HISTORY);
    for event in history.iter().skip(first) {
        notifications.push_with_duration(event.message.clone(), event.notification_type, 0.0);
    }
    notifications.update(0.0);
}

/// Reject saves that deserialize into a shape the simulation cannot safely
/// operate on. Serde checks the field types, but it cannot know that a map
/// position must be unique, a content id must exist in the embedded registry,
/// or a timer must be finite. Keeping this check at the load boundary means a
/// damaged primary can take the normal quarantine/backup path instead of
/// poisoning the live session.
pub(super) fn validate_loaded_session(
    session: &GameSession,
    data: &GameData,
) -> Result<(), String> {
    let tiles = &session.world.tiles;
    let expected_cells = tiles
        .width
        .checked_mul(tiles.height)
        .ok_or_else(|| "world dimensions overflowed".to_owned())?;
    if tiles.width == 0 || tiles.height == 0 || tiles.data().len() != expected_cells {
        return Err("world grid dimensions do not match its stored tiles".to_owned());
    }
    if !session.world.spawn.in_bounds(tiles.width, tiles.height) {
        return Err("world spawn is outside the stored map".to_owned());
    }

    let mut occupied = HashSet::new();
    for building in &session.buildings {
        if data.buildings.get(&building.kind).is_none() {
            return Err(format!("unknown building id {:?}", building.kind));
        }
        validate_walkable_position(
            session,
            building.pos,
            &format!("building {:?}", building.kind),
        )?;
        if !occupied.insert(building.pos) {
            return Err(format!("multiple buildings occupy {:?}", building.pos));
        }
        validate_nonnegative_finite(building.reserve, "building reserve")?;
        validate_nonnegative_finite(building.waste, "building waste")?;
        for (good, amount) in &building.stocks {
            validate_nonnegative_finite(*amount, &format!("building stock {good:?}"))?;
        }
        for order in &building.orders {
            if data.equipment_def(order).is_none() {
                return Err(format!("unknown equipment order id {order:?}"));
            }
        }
    }
    for equipment in session.economy.gear_stock.keys() {
        if data.equipment_def(equipment).is_none() {
            return Err(format!("unknown stored equipment id {equipment:?}"));
        }
    }

    for site in &session.build_sites {
        if data.buildings.get(&site.kind).is_none() {
            return Err(format!("unknown construction id {:?}", site.kind));
        }
        validate_walkable_position(session, site.pos, "construction site")?;
        if !occupied.insert(site.pos) {
            return Err(format!(
                "construction overlaps an occupied tile {:?}",
                site.pos
            ));
        }
        if site.ore_needed == 0 || site.ore_delivered > site.ore_needed {
            return Err(format!("invalid construction progress at {:?}", site.pos));
        }
    }

    for pos in &session.dig_marks {
        if !pos.in_bounds(tiles.width, tiles.height)
            || !session.world.tiles.get(*pos).is_some_and(|tile| {
                matches!(
                    tile,
                    crate::state::world::Tile::Rock | crate::state::world::Tile::OreVein
                )
            })
        {
            return Err(format!("invalid dig designation at {pos:?}"));
        }
    }
    for (pos, remaining) in &session.patch_regrow {
        validate_map_timer(
            *pos,
            *remaining,
            tiles.width,
            tiles.height,
            "mushroom regrow",
        )?;
    }
    for (pos, remaining) in &session.sporewood_regrow {
        validate_map_timer(
            *pos,
            *remaining,
            tiles.width,
            tiles.height,
            "sporewood regrow",
        )?;
    }
    for pos in session.vein_ore.keys() {
        if !pos.in_bounds(tiles.width, tiles.height) {
            return Err(format!("ore vein state is outside the map at {pos:?}"));
        }
    }

    validate_unique_ids(
        session.creatures.iter().map(|creature| creature.id),
        session.next_creature_id,
        "creature",
    )?;
    for creature in &session.creatures {
        if data.species.get(&creature.species).is_none() {
            return Err(format!("unknown creature species {:?}", creature.species));
        }
        validate_actor_position(session, creature.x, creature.y, "creature")?;
        validate_nonnegative_finite(creature.starving_for, "creature starvation timer")?;
        validate_nonnegative_finite(creature.hp, "creature health")?;
        validate_nonnegative_finite(creature.morale_stress_for, "creature morale timer")?;
        if !creature.satiation.is_finite() || !creature.morale.is_finite() {
            return Err("creature wellbeing contains a non-finite value".to_owned());
        }
        if let Some(equipment) = &creature.equipment {
            if data.equipment_def(equipment).is_none() {
                return Err(format!("unknown equipped item id {equipment:?}"));
            }
        }
        validate_task_positions(session, &creature.task, &creature.path)?;
        if let Some((_, amount)) = creature.carrying {
            if amount == 0 {
                return Err(format!("creature {} carries an empty load", creature.id));
            }
        }
    }

    validate_unique_ids(
        session.wilds.iter().map(|wild| wild.id),
        session.next_wild_id,
        "wild creature",
    )?;
    for wild in &session.wilds {
        if data.species.get(&wild.species).is_none() {
            return Err(format!("unknown wild species {:?}", wild.species));
        }
        validate_actor_position(session, wild.x, wild.y, "wild creature")?;
        validate_nonnegative_finite(wild.hp, "wild creature health")?;
        validate_wild_behavior(session, &wild.behavior)?;
        for path_pos in &wild.path {
            validate_map_position(*path_pos, tiles.width, tiles.height, "wild creature path")?;
        }
    }

    for outpost in &session.outposts {
        validate_walkable_position(session, outpost.pos, "outpost")?;
        if session
            .outposts
            .iter()
            .filter(|other| other.pos == outpost.pos)
            .count()
            > 1
        {
            return Err(format!("multiple outpost records occupy {:?}", outpost.pos));
        }
        if !session
            .buildings
            .iter()
            .any(|building| building.pos == outpost.pos && building.kind == "outpost")
        {
            return Err(format!(
                "outpost record has no matching building at {:?}",
                outpost.pos
            ));
        }
        validate_nonnegative_finite(outpost.expedition_progress, "outpost expedition progress")?;
    }
    if let Some(transit) = &session.worm_transit {
        if !session
            .outposts
            .iter()
            .any(|outpost| outpost.pos == transit.outpost)
        {
            return Err(format!(
                "transit targets an unknown outpost {:?}",
                transit.outpost
            ));
        }
        validate_nonnegative_finite(transit.remaining, "worm transit timer")?;
        validate_nonnegative_finite(transit.food, "worm transit food")?;
    }

    for (name, value) in [
        ("food", session.economy.food),
        ("raw food", session.economy.raw_food),
        ("cooked food", session.economy.cooked_food),
        ("waste", session.economy.waste),
        ("processed waste", session.economy.waste_processed),
        (
            "food production rate",
            session.economy.production_ema_per_min,
        ),
        ("ore production rate", session.economy.ore_ema_per_min),
        ("ingot production rate", session.economy.ingot_ema_per_min),
        ("worm fed food", session.worm_fed),
    ] {
        validate_nonnegative_finite(value, name)?;
    }
    for (name, value) in [
        ("wild spawn timer", session.wild_spawn_in),
        ("raid timer", session.raid_in),
        ("breeding timer", session.breed_in),
        ("progress knowledge", session.progress.knowledge),
        ("progress waste generated", session.progress.waste_generated),
    ] {
        validate_nonnegative_finite(value, name)?;
    }
    Ok(())
}

fn validate_nonnegative_finite(value: f32, field: &str) -> Result<(), String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!("{field} is not a finite non-negative value"));
    }
    Ok(())
}

fn validate_map_position(
    pos: macroquad_toolkit::grid::TilePos,
    width: usize,
    height: usize,
    field: &str,
) -> Result<(), String> {
    if !pos.in_bounds(width, height) {
        return Err(format!("{field} is outside the map at {pos:?}"));
    }
    Ok(())
}

fn validate_walkable_position(
    session: &GameSession,
    pos: macroquad_toolkit::grid::TilePos,
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

fn validate_map_timer(
    pos: macroquad_toolkit::grid::TilePos,
    remaining: f32,
    width: usize,
    height: usize,
    field: &str,
) -> Result<(), String> {
    validate_map_position(pos, width, height, field)?;
    validate_nonnegative_finite(remaining, field)
}

fn validate_actor_position(
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

fn validate_unique_ids(
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

fn validate_task_positions(
    session: &GameSession,
    task: &Task,
    path: &[macroquad_toolkit::grid::TilePos],
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

fn validate_wild_behavior(
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

pub(super) fn non_viable_save_notice(
    session: &GameSession,
    data: &GameData,
) -> Option<&'static str> {
    if !session.is_non_viable(data) {
        return None;
    }
    Some(if session.creatures.is_empty() {
        "This warren has fallen silent. Load a safe save or start a new warren instead."
    } else {
        "This warren cannot staff the Guard post. Load a safe save or start a new warren instead."
    })
}

/// Reconcile the old seven-step tutorial index with the current five-beat
/// sequence using facts that are actually persisted in a campaign save.
///
/// The old index alone is ambiguous: its Mine, Blacksmith, and famine lessons
/// no longer have one-to-one replacements. Session milestones give a safe
/// forward-only mapping without making a returning player repeat completed
/// factory or campaign work.
pub(super) fn migrate_tutorial_progress(session: &mut GameSession, tutorial_count: usize) {
    let old_step = session.tutorial_step;
    let mut step = usize::from(old_step > 0);

    // Current saves set `tutorial_built` as soon as a site is placed. Do not
    // skip the Food lesson on a reload while that site is still waiting for
    // ore; completion needs both the migration marker and the second Farm.
    if session.tutorial_build_completed && session.buildings_of("farm").nth(1).is_some() {
        step = step.max(2);
    } else if old_step >= 4 && session.tutorial_built {
        // Older saves had no completion marker, but their later tutorial
        // index is enough evidence that the early construction beat passed.
        step = step.max(2);
    }
    if session
        .economy
        .gear_stock
        .get("iron_pickaxe")
        .copied()
        .unwrap_or(0)
        > 0
        || session
            .creatures
            .iter()
            .any(|creature| creature.equipment.as_deref() == Some("iron_pickaxe"))
    {
        step = step.max(3);
    }
    if session.won && session.job_count(Job::Guard) > 0 {
        step = step.max(4);
    }
    if session.worm_awake {
        step = tutorial_count;
    }

    session.tutorial_step = step.min(tutorial_count);
}
