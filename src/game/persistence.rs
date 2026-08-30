//! Save, load, migration, and frame-state recovery for the game shell.

use super::Game;
use crate::data::GameData;
use crate::state::creatures::Job;
use crate::state::{GameSession, GameState};
use crate::ui::UiMode;
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, quarantine_slot, restore_slot_backup,
    save_to_slot_with_version_and_backup, slot_backup_exists, slot_exists,
};

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
        match self.persist_current_session() {
            Ok(()) => {
                self.save_exists = true;
                self.notifications.success("Warren saved.");
            }
            Err(err) => self.notifications.danger(format!("Save failed: {err}")),
        }
    }

    /// Persist a campaign milestone or direct player decision without
    /// interrupting the player's flow.
    pub(super) fn autosave_game(&mut self) {
        if matches!(&self.state, GameState::Warren(session) if session.is_non_viable(&self.data)) {
            return;
        }
        match self.persist_current_session() {
            Ok(()) => {
                self.save_exists = true;
            }
            Err(err) => self
                .notifications
                .warning(format!("Autosave failed — use Save manually: {err}")),
        }
    }

    fn persist_current_session(&self) -> Result<(), String> {
        let GameState::Warren(session) = &self.state else {
            return Err("no active Warren".to_owned());
        };
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
                Ok(session)
            },
        )
    }

    fn install_loaded_session(&mut self, session: GameSession) {
        let mut session = session;
        session.sync_remote_crew_state();
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
        if !slot_exists(&config.game_name, slot) {
            self.notifications.warning(format!("Load failed: {error}"));
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

        if !slot_backup_exists(&config.game_name, slot) {
            self.save_exists = false;
            self.notifications.danger(format!(
                "Save is damaged; preserved it as {quarantine}. Use Menu → New Warren or repair it before loading again."
            ));
            return;
        }

        let backup_slot = format!("{slot}_backup");
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
