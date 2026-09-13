//! Save, load, migration, and frame-state recovery for the game shell.

use super::Game;
use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::outposts::TransitDirection;
use crate::state::{GameSession, GameState};
use crate::ui::UiMode;
use macroquad_toolkit::notifications::{LoggedNotification, NotificationManager, MAX_HISTORY};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, quarantine_slot, restore_slot_backup,
    save_to_slot_with_version_and_backup, slot_backup_exists, slot_exists,
};
use std::collections::HashSet;

mod load;
mod validation;

use validation::{
    validate_actor_position, validate_map_position, validate_map_timer,
    validate_nonnegative_finite, validate_outpost_cargo, validate_outpost_milestones,
    validate_task_positions, validate_unique_ids, validate_walkable_position,
    validate_wild_behavior,
};

impl Game {
    pub fn save_game(&mut self) {
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
                self.checkpoint_warning = None;
                self.notifications.success("Warren saved.");
            }
            Err(err) => {
                self.checkpoint_warning = Some(save_failure_banner(had_existing_save));
                self.notifications
                    .danger(save_failure_notice(false, had_existing_save, &err))
            }
        }
    }

    /// Persist a campaign milestone or direct player decision without
    /// interrupting the player's flow.
    pub fn autosave_game(&mut self) -> bool {
        if matches!(&self.state, GameState::Warren(session) if session.is_non_viable(&self.data)) {
            return false;
        }
        if matches!(&self.state, GameState::Warren(session) if !autosave_checkpoint_is_safe(session))
        {
            return true;
        }
        let had_existing_save = self.save_exists;
        match self.persist_current_session() {
            Ok(()) => {
                self.save_exists = true;
                self.checkpoint_warning = None;
                true
            }
            Err(err) => {
                self.checkpoint_warning = Some(save_failure_banner(had_existing_save));
                self.notifications
                    .warning(save_failure_notice(true, had_existing_save, &err));
                false
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

    fn load_session_from_slot(&self, slot: &str) -> Result<GameSession, String> {
        let config = &self.data.config;
        let session = load_from_slot_with_migration(
            &config.game_name,
            slot,
            &config.version,
            |version, value| {
                let payload = value.get("data").cloned().unwrap_or(value);
                let mut session: GameSession = serde_json::from_value(payload)
                    .map_err(|err| format!("Unsupported save {version:?}: {err}"))?;
                migrate_tutorial_progress(&mut session, self.data.tutorial.len());
                Ok(session)
            },
        )?;
        // The toolkit returns current-version saves through a fast path that
        // does not invoke the migration callback. Keep integrity validation at
        // this project-owned boundary so current and migrated saves receive
        // the same map, roster, content, and finite-value checks.
        validate_loaded_session_boundary(session, &self.data)
    }

    pub fn install_loaded_session(&mut self, session: GameSession) {
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
    pub fn reset_session_view_state(&mut self) {
        self.accumulator = 0.0;
        self.famine_announced = false;
        self.mode = UiMode::Inspect;
        self.help_open = false;
        self.event_log_open = false;
        self.event_log_page = 0;
        self.routes_open = false;
        self.hud_panel = None;
        self.paused = false;
        super::clear_replacement_confirmations(
            &mut self.confirm_new_warren,
            &mut self.confirm_load,
        );
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
                    self.checkpoint_warning = None;
                    self.notifications.warning(format!(
                        "Primary save was damaged; preserved it as {quarantine} and restored the previous safe save."
                    ));
                }
                Err(restore_error) => {
                    self.install_loaded_session(session);
                    // The validated backup is still a usable Continue source
                    // even when storage rejects recreating the primary slot.
                    self.save_exists = true;
                    self.checkpoint_warning = Some(save_recovery_failure_banner());
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
                    self.checkpoint_warning = None;
                    self.notifications
                        .warning("Primary save was missing; restored the previous safe save.");
                }
                Err(restore_error) => {
                    self.install_loaded_session(session);
                    // Keep Continue available because the backup remains
                    // valid; the banner asks the player to repair the primary.
                    self.save_exists = true;
                    self.checkpoint_warning = Some(save_recovery_failure_banner());
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

pub fn should_restore_missing_primary(primary_exists: bool, backup_exists: bool) -> bool {
    !primary_exists && backup_exists
}

/// Whether the title screen should offer Continue. A surviving backup is a
/// recoverable save even when the primary slot disappeared between launches.
pub fn save_slot_available(primary_exists: bool, backup_exists: bool) -> bool {
    primary_exists || backup_exists
}

pub fn should_load_safe_backup(
    session: &GameSession,
    data: &GameData,
    backup_exists: bool,
) -> bool {
    backup_exists && session.is_non_viable(data)
}

pub fn autosave_checkpoint_is_safe(session: &GameSession) -> bool {
    session.worm_awake || session.economy.food > 0.0
}

pub fn no_saved_slot_available(primary_exists: bool, backup_exists: bool) -> bool {
    !save_slot_available(primary_exists, backup_exists)
}

pub fn missing_save_notice(error: &str) -> String {
    format!("Load failed — no saved warren is available: {error}. Use New Warren to begin again.")
}

pub fn save_failure_notice(autosave: bool, had_existing_save: bool, error: &str) -> String {
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

/// Persistent compact banner for a rejected save. The full storage error is
/// retained in the transient toast and Recent Events; this banner stays short
/// enough to sit beside the visible Save button at the compact layout.
pub fn save_failure_banner(had_existing_save: bool) -> &'static str {
    if had_existing_save {
        "SAVE FAILED · checkpoint safe"
    } else {
        "SAVE FAILED · tap Save"
    }
}

/// Persistent banner for a valid backup that loaded but could not recreate
/// the primary slot. Continue remains available through the backup, while
/// Save is the visible repair action.
pub fn save_recovery_failure_banner() -> &'static str {
    "RECOVERY FAILED · tap Save"
}

/// Rehydrate only the manager's non-timed history. Loading a save should not
/// replay every old toast over the Warren, but the Field Guide must still be
/// able to review those messages after a refresh or relaunch.
pub fn restore_notification_history(
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
mod session_validation;
pub use session_validation::{validate_loaded_session, validate_loaded_session_boundary};

pub fn non_viable_save_notice(session: &GameSession, data: &GameData) -> Option<&'static str> {
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
pub fn migrate_tutorial_progress(session: &mut GameSession, tutorial_count: usize) {
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
