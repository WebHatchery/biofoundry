//! UI dispatch for the awakened Outpost upgrade ladder.

use crate::audio::Sfx;
use crate::game::Game;
use crate::simulation;
use crate::state::GameState;

impl Game {
    pub(super) fn apply_outpost_hold_upgrade(&mut self, pos: macroquad_toolkit::grid::TilePos) {
        let mut upgraded = false;
        if let GameState::Warren(session) = &mut self.state {
            if simulation::outposts::upgrade_outpost(session, &self.data, pos) {
                let capacity = session
                    .outposts
                    .iter()
                    .find(|outpost| outpost.pos == pos)
                    .map(|outpost| simulation::outposts::storage_capacity(outpost, &self.data))
                    .unwrap_or(self.data.balance.outpost_storage_cap);
                self.notifications
                    .success(format!("Outpost hold expanded to {capacity} slots."));
                self.audio.play(Sfx::Complete);
                upgraded = true;
            } else {
                self.notifications.warning(format!(
                    "Needs {} banked ingots and an active route.",
                    self.data.balance.outpost_upgrade_ingots
                ));
                self.audio.play(Sfx::Deny);
            }
        }
        if upgraded {
            self.autosave_game();
        }
    }

    pub(super) fn apply_outpost_crew_upgrade(&mut self, pos: macroquad_toolkit::grid::TilePos) {
        let mut upgraded = false;
        if let GameState::Warren(session) = &mut self.state {
            if simulation::outposts::upgrade_outpost_crew(session, &self.data, pos) {
                let capacity = session
                    .outposts
                    .iter()
                    .find(|outpost| outpost.pos == pos)
                    .map(|outpost| simulation::outposts::crew_capacity(outpost, &self.data))
                    .unwrap_or(self.data.balance.outpost_capacity);
                self.notifications
                    .success(format!("Outpost camp expanded to {capacity} crew."));
                self.audio.play(Sfx::Complete);
                upgraded = true;
            } else {
                self.notifications.warning(format!(
                    "Needs {} banked ingots and an active route.",
                    self.data.balance.outpost_crew_upgrade_ingots
                ));
                self.audio.play(Sfx::Deny);
            }
        }
        if upgraded {
            self.autosave_game();
        }
    }

    pub(super) fn apply_outpost_survey_upgrade(&mut self, pos: macroquad_toolkit::grid::TilePos) {
        let mut upgraded = false;
        if let GameState::Warren(session) = &mut self.state {
            if simulation::outposts::upgrade_outpost_survey(session, &self.data, pos) {
                let ore_per_crew = session
                    .outposts
                    .iter()
                    .find(|outpost| outpost.pos == pos)
                    .map(|outpost| simulation::outposts::ore_per_crew(outpost, &self.data))
                    .unwrap_or(self.data.balance.outpost_expedition_ore_per_crew);
                self.notifications
                    .success(format!("Survey rig online · {ore_per_crew} ore/scout."));
                self.audio.play(Sfx::Complete);
                upgraded = true;
            } else {
                self.notifications.warning(format!(
                    "Expand the Outpost hold and camp first, then spend {} ingots.",
                    self.data.balance.outpost_survey_upgrade_ingots
                ));
                self.audio.play(Sfx::Deny);
            }
        }
        if upgraded {
            self.autosave_game();
        }
    }

    pub(super) fn apply_outpost_resonator_upgrade(
        &mut self,
        pos: macroquad_toolkit::grid::TilePos,
    ) {
        let mut upgraded = false;
        if let GameState::Warren(session) = &mut self.state {
            if simulation::outposts::upgrade_outpost_resonator(session, &self.data, pos) {
                let cycle = session
                    .outposts
                    .iter()
                    .find(|outpost| outpost.pos == pos)
                    .map(|outpost| simulation::outposts::expedition_cycle_sec(outpost, &self.data))
                    .unwrap_or(self.data.balance.outpost_expedition_cycle_sec);
                self.notifications
                    .success(format!("Resonance beacon tuned · {cycle:.0}s surveys."));
                self.audio.play(Sfx::Complete);
                upgraded = true;
            } else {
                self.notifications.warning(format!(
                    "Install the survey rig first, then spend {} ingots.",
                    self.data.balance.outpost_resonator_upgrade_ingots
                ));
                self.audio.play(Sfx::Deny);
            }
        }
        if upgraded {
            self.autosave_game();
        }
    }

    pub(super) fn apply_outpost_deep_survey_upgrade(
        &mut self,
        pos: macroquad_toolkit::grid::TilePos,
    ) {
        let mut upgraded = false;
        if let GameState::Warren(session) = &mut self.state {
            if simulation::outposts::upgrade_outpost_deep_survey(session, &self.data, pos) {
                let ore_per_crew = session
                    .outposts
                    .iter()
                    .find(|outpost| outpost.pos == pos)
                    .map(|outpost| simulation::outposts::ore_per_crew(outpost, &self.data))
                    .unwrap_or(self.data.balance.outpost_upgraded_ore_per_crew);
                self.notifications.success(format!(
                    "Deep survey calibrated · {ore_per_crew} ore/scout."
                ));
                self.audio.play(Sfx::Complete);
                upgraded = true;
            } else {
                self.notifications.warning(format!(
                    "Claim the Worm Road Charter and tune the beacon, then spend {} ingots.",
                    self.data.balance.outpost_deep_survey_upgrade_ingots
                ));
                self.audio.play(Sfx::Deny);
            }
        }
        if upgraded {
            self.autosave_game();
        }
    }

    pub(super) fn apply_outpost_signal_cache_upgrade(
        &mut self,
        pos: macroquad_toolkit::grid::TilePos,
    ) {
        let mut upgraded = false;
        if let GameState::Warren(session) = &mut self.state {
            if simulation::outposts::upgrade_outpost_signal_cache(session, &self.data, pos) {
                self.notifications.success(format!(
                    "Signal cache online · +{} ingot per haul.",
                    self.data.balance.outpost_signal_cache_ingots_per_haul
                ));
                self.audio.play(Sfx::Complete);
                upgraded = true;
            } else {
                self.notifications.warning(format!(
                    "Claim Relay and calibrate Deep Survey first, then spend {} ingots.",
                    self.data.balance.outpost_signal_cache_upgrade_ingots
                ));
                self.audio.play(Sfx::Deny);
            }
        }
        if upgraded {
            self.autosave_game();
        }
    }

    pub(super) fn apply_outpost_waypoint_upgrade(&mut self, pos: macroquad_toolkit::grid::TilePos) {
        let mut upgraded = false;
        if let GameState::Warren(session) = &mut self.state {
            if simulation::outposts::upgrade_outpost_waypoint(session, &self.data, pos) {
                let transit_time = session
                    .outposts
                    .iter()
                    .find(|outpost| outpost.pos == pos)
                    .map(|outpost| simulation::outposts::transit_time_sec(outpost, &self.data))
                    .unwrap_or(self.data.balance.outpost_waypoint_transit_time_sec);
                self.notifications.success(format!(
                    "Worm Road Waypoint online · {transit_time:.0}s transit."
                ));
                self.audio.play(Sfx::Complete);
                upgraded = true;
            } else {
                self.notifications.warning(format!(
                    "Clear a Worm Road Convoy first, then spend {} ingots.",
                    self.data.balance.outpost_waypoint_upgrade_ingots
                ));
                self.audio.play(Sfx::Deny);
            }
        }
        if upgraded {
            self.autosave_game();
        }
    }
}
