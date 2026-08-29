//! UI intent dispatch kept separate from the top-level frame loop.

use super::Game;
use crate::audio::Sfx;
use crate::data::GameData;
use crate::simulation;
use crate::state::creatures::Job;
use crate::state::{GameState, StateTransition};
use crate::ui::{UiAction, UiMode};
use macroquad::prelude::*;

impl Game {
    pub(super) fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::StartWarren => self.transition(StateTransition::StartWarren),
            UiAction::RequestNewWarren => {
                if matches!(&self.state, GameState::Menu) && self.save_exists {
                    self.confirm_new_warren = true;
                    self.audio.play(Sfx::Select);
                } else {
                    self.transition(StateTransition::StartWarren);
                }
            }
            UiAction::CancelNewWarren => {
                self.confirm_new_warren = false;
                self.audio.play(Sfx::Select);
            }
            UiAction::BackToMenu => self.transition(StateTransition::BackToMenu),
            UiAction::Assign(job) => self.reassign(Job::Idle, job),
            UiAction::Unassign(job) => self.reassign(job, Job::Idle),
            UiAction::AttractBeetle => {
                if let GameState::Warren(session) = &mut self.state {
                    if simulation::try_attract_beetle(session, &self.data) {
                        self.notifications.success(recruitment_notice(
                            &self.data,
                            "beetle",
                            "A beetle hauler joins the warren",
                        ));
                        self.audio.play(Sfx::Capture);
                    } else {
                        self.notifications.warning("Not enough ore banked.");
                        self.audio.play(Sfx::Deny);
                    }
                }
            }
            UiAction::AttractSalamander => {
                if let GameState::Warren(session) = &mut self.state {
                    if simulation::try_attract_salamander(session, &self.data) {
                        self.notifications
                            .success("A salamander curls into the smelter den.");
                        self.audio.play(Sfx::Capture);
                    } else {
                        self.notifications
                            .warning("Needs a Smelter Den and enough banked ore.");
                        self.audio.play(Sfx::Deny);
                    }
                }
            }
            UiAction::AttractSlimeJanitor => {
                if let GameState::Warren(session) = &mut self.state {
                    if simulation::try_attract_slime_janitor(session, &self.data) {
                        self.notifications.success(recruitment_notice(
                            &self.data,
                            "slime_janitor",
                            "A Slime Janitor bubbles into the warren",
                        ));
                        self.audio.play(Sfx::Capture);
                    } else {
                        self.notifications
                            .warning("The slime janitor is not unlocked yet.");
                        self.audio.play(Sfx::Deny);
                    }
                }
            }
            UiAction::AttractBatCourier => {
                if let GameState::Warren(session) = &mut self.state {
                    if simulation::try_attract_bat_courier(session, &self.data) {
                        self.notifications.success(recruitment_notice(
                            &self.data,
                            "bat_courier",
                            "A Bat Courier takes to the tunnels",
                        ));
                        self.audio.play(Sfx::Capture);
                    } else {
                        self.notifications
                            .warning("The bat courier is not unlocked yet.");
                        self.audio.play(Sfx::Deny);
                    }
                }
            }
            UiAction::DismissVictory => {
                if let GameState::Warren(session) = &mut self.state {
                    session.victory_shown = true;
                }
            }
            UiAction::DismissFactory => {
                if let GameState::Warren(session) = &mut self.state {
                    session.factory_shown = true;
                }
            }
            UiAction::DismissWorm => {
                if let GameState::Warren(session) = &mut self.state {
                    session.worm_shown = true;
                }
            }
            UiAction::SkipTutorial => {
                if let GameState::Warren(session) = &mut self.state {
                    session.tutorial_dismissed = true;
                    self.audio.play(Sfx::Select);
                }
            }
            UiAction::SetMode(mode) => {
                self.mode = if self.mode == mode {
                    UiMode::Inspect
                } else {
                    mode
                };
                self.audio.play(Sfx::Select);
            }
            UiAction::Breed(species) => {
                if let GameState::Warren(session) = &mut self.state {
                    let ok = match species.as_str() {
                        "hobgoblin" => simulation::try_breed_hobgoblin(session, &self.data),
                        "overseer" => simulation::try_breed_overseer(session, &self.data),
                        "engineer" => simulation::try_breed_engineer(session, &self.data),
                        _ => false,
                    };
                    if ok {
                        self.notifications.success(recruitment_notice(
                            &self.data,
                            &species,
                            &format!("A {species} emerges from the pit"),
                        ));
                        self.audio.play(Sfx::Capture);
                    } else {
                        self.notifications
                            .warning("Needs the unlock, a breeding pit, and banked ingots.");
                        self.audio.play(Sfx::Deny);
                    }
                }
            }
            UiAction::ToggleShrineFeeding(pos) => {
                if let GameState::Warren(session) = &mut self.state {
                    if session
                        .building_at(pos)
                        .is_some_and(|b| b.kind == "worm_shrine")
                    {
                        session.worm_feeding_paused = !session.worm_feeding_paused;
                        self.notifications.info(if session.worm_feeding_paused {
                            "Shrine offerings paused."
                        } else {
                            "Shrine offerings resumed."
                        });
                    }
                }
            }
            UiAction::ActivateOutpost(pos) => {
                if let GameState::Warren(session) = &mut self.state {
                    if simulation::outposts::activate_outpost(session, pos) {
                        self.notifications.info("The worm route is now active.");
                    } else {
                        self.notifications
                            .warning("This outpost cannot reach the shrine yet.");
                    }
                }
            }
            UiAction::TransitToOutpost(pos) | UiAction::TransitToShrine(pos) => {
                if let GameState::Warren(session) = &mut self.state {
                    let ok = match action {
                        UiAction::TransitToOutpost(_) => {
                            simulation::outposts::start_to_outpost(session, &self.data, pos)
                        }
                        UiAction::TransitToShrine(_) => {
                            simulation::outposts::start_to_shrine(session, &self.data, pos)
                        }
                        _ => false,
                    };
                    if ok {
                        self.notifications
                            .info("The worm carries the route's cargo.");
                    } else {
                        self.notifications
                            .warning("No valid cargo or route is ready.");
                    }
                }
            }
            UiAction::WorldClick(tile) => self.world_click(tile),
            UiAction::QueueOrder(pos, item) => {
                let cap = self.data.balance.order_queue_size;
                if let GameState::Warren(session) = &mut self.state {
                    if let Some(b) = session.building_at_mut(pos) {
                        if b.kind == "blacksmith" && b.orders.len() < cap {
                            b.orders.push(item);
                            self.notifications.info("Order queued.");
                            self.audio.play(Sfx::Select);
                        } else {
                            self.audio.play(Sfx::Deny);
                        }
                    }
                }
            }
            UiAction::Save => self.save_game(),
            UiAction::Load => self.load_game(),
            UiAction::ToggleSettings => {
                self.settings_open = !self.settings_open;
                self.audio.play(Sfx::Select);
            }
            UiAction::ToggleHelp => {
                self.help_open = !self.help_open;
                self.audio.play(Sfx::Select);
            }
            UiAction::TogglePause => {
                if matches!(&self.state, GameState::Warren(_)) {
                    self.paused = !self.paused;
                    // A partial frame should not be carried across a pause;
                    // resuming starts cleanly on the next fixed timestep.
                    self.accumulator = 0.0;
                    self.notifications.info(if self.paused {
                        "Simulation paused — tap Resume to continue."
                    } else {
                        "Simulation resumed."
                    });
                    self.audio.play(Sfx::Select);
                }
            }
            UiAction::AdjustVolume(steps) => {
                let volume = (self.audio.volume() * 10.0 + steps as f32).round() / 10.0;
                self.audio.set_volume(volume);
                self.audio.save_settings(&self.data.config.game_name);
                self.audio.play(Sfx::Select);
            }
            UiAction::ZoomCamera(direction) => {
                let factor = if direction >= 0 {
                    self.camera.config.zoom_in_factor
                } else {
                    self.camera.config.zoom_out_factor
                };
                self.camera
                    .zoom_at(factor, vec2(screen_width(), screen_height()) * 0.5);
            }
            UiAction::ExitGame => macroquad::miniquad::window::quit(),
        }
    }
}

/// Call out the food cost of optional recruits at the moment they join, so a
/// successful growth choice cannot quietly turn the Food Grid negative.
fn recruitment_notice(data: &GameData, species: &str, joined: &str) -> String {
    let upkeep = data
        .species
        .get(species)
        .map(|definition| definition.food_per_min)
        .unwrap_or(0.0);
    if upkeep > 0.0 {
        format!("{joined} — Upkeep +{upkeep:.1} food/min.")
    } else {
        format!("{joined}.")
    }
}

#[cfg(test)]
#[path = "game_actions/tests.rs"]
mod tests;
