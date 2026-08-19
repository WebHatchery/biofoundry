//! UI intent dispatch kept separate from the top-level frame loop.

use super::Game;
use crate::audio::Sfx;
use crate::simulation;
use crate::state::creatures::Job;
use crate::state::{GameState, StateTransition};
use crate::ui::{UiAction, UiMode};

impl Game {
    pub(super) fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::StartWarren => self.transition(StateTransition::StartWarren),
            UiAction::BackToMenu => self.transition(StateTransition::BackToMenu),
            UiAction::Assign(job) => self.reassign(Job::Idle, job),
            UiAction::Unassign(job) => self.reassign(job, Job::Idle),
            UiAction::AttractBeetle => {
                if let GameState::Warren(session) = &mut self.state {
                    if simulation::try_attract_beetle(session, &self.data) {
                        self.notifications
                            .success("A beetle hauler joins the warren.");
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
                        self.notifications
                            .success("A Slime Janitor bubbles into the warren.");
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
                        self.notifications
                            .success("A Bat Courier takes to the tunnels.");
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
                        self.notifications
                            .success(format!("A {species} emerges from the pit."));
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
            UiAction::AdjustVolume(steps) => {
                let volume = (self.audio.volume() * 10.0 + steps as f32).round() / 10.0;
                self.audio.set_volume(volume);
                self.audio.save_settings(&self.data.config.game_name);
                self.audio.play(Sfx::Select);
            }
            UiAction::ExitGame => macroquad::miniquad::window::quit(),
        }
    }
}
