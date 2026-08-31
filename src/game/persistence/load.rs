//! Primary and safe-backup load requests for the game shell.

use super::Game;
use crate::state::GameState;
use macroquad_toolkit::persistence::{restore_slot_backup, slot_backup_exists};

impl Game {
    pub(in crate::game) fn load_game(&mut self) {
        // A confirmed load consumes the modal before storage work begins. If
        // the slot is damaged or unavailable, the error notification must be
        // readable over the live Warren rather than leaving the player trapped
        // behind a stale confirmation overlay.
        self.confirm_load = false;
        let slot = self.data.config.save_slot.clone();
        let backup_slot = format!("{slot}_backup");
        let load_safe_backup = matches!(
            &self.state,
            GameState::Warren(session) if super::should_load_safe_backup(
                session,
                &self.data,
                slot_backup_exists(&self.data.config.game_name, &slot),
            )
        );
        let load_slot = if load_safe_backup {
            backup_slot.as_str()
        } else {
            slot.as_str()
        };
        match self.load_session_from_slot(load_slot) {
            Ok(session) => {
                self.install_loaded_session(session);
                // The slot may have become available after startup (or after
                // a prior failed recovery), so a successful load must restore
                // the title screen's Continue affordance as well.
                self.save_exists = true;
                let notice = if load_safe_backup {
                    match restore_slot_backup(&self.data.config.game_name, &slot) {
                        Ok(_) => {
                            self.checkpoint_warning = None;
                            Some("Safe checkpoint loaded and restored.")
                        }
                        Err(err) => {
                            self.checkpoint_warning = Some(super::save_recovery_failure_banner());
                            self.notifications.warning(format!(
                                "Safe checkpoint loaded, but could not restore it ({err}). Use Save now."
                            ));
                            None
                        }
                    }
                } else {
                    self.checkpoint_warning = None;
                    Some("Warren loaded.")
                };
                if let Some(notice) = notice {
                    self.notifications.success(notice);
                }
            }
            Err(err) if load_safe_backup => self.notifications.danger(format!(
                "Safe checkpoint could not load: {err}. Use Start New Warren instead."
            )),
            Err(err) => self.recover_failed_load(&slot, err),
        }
    }
}
