//! The tutorial: a sequence of data-driven steps (`tutorial.json`) shown
//! in the HUD. Each step completes when the player reaches the corresponding
//! journey beat — enter, stabilize food, close the factory loop, secure the
//! warren, awaken the worm. Pure guidance: it reads state and never touches
//! the sim.

use crate::data::{GameData, TutorialDone, TutorialStepDef};
use crate::state::GameSession;

/// Frame-side signals the session can't see (camera input lives in `Game`).
#[derive(Debug, Default, Clone, Copy)]
pub struct TutorialInputs {
    pub camera_moved: bool,
}

/// The step to display, if the tutorial is active.
pub fn current_step<'a>(session: &GameSession, data: &'a GameData) -> Option<&'a TutorialStepDef> {
    if session.tutorial_dismissed {
        return None;
    }
    data.tutorial.get(session.tutorial_step)
}

/// Steps completed so far out of the total (for the "2/6" chip).
pub fn progress(session: &GameSession, data: &GameData) -> (usize, usize) {
    (session.tutorial_step, data.tutorial.len())
}

/// Advance past every step whose condition is now met. Returns true when
/// at least one step completed this call (so the UI can chirp once).
pub fn advance(session: &mut GameSession, data: &GameData, inputs: TutorialInputs) -> bool {
    let mut advanced = false;
    while let Some(step) = current_step(session, data) {
        if !step_done(&step.done, session, inputs) {
            break;
        }
        session.tutorial_step += 1;
        advanced = true;
    }
    advanced
}

fn step_done(done: &TutorialDone, session: &GameSession, inputs: TutorialInputs) -> bool {
    match done {
        TutorialDone::CameraMoved => inputs.camera_moved,
        TutorialDone::AnyReassign => session.tutorial_reassigned,
        TutorialDone::WarrenSecured => session.won,
        TutorialDone::SitePlaced => session.tutorial_built,
        TutorialDone::BuildingPlaced { building } => {
            session.buildings_of(building).next().is_some()
                || session.build_sites.iter().any(|s| &s.kind == building)
        }
        TutorialDone::MineWorking => {
            use crate::state::creatures::Good;
            session
                .buildings_of("mine")
                .any(|b| b.stock(Good::Ore) >= 1.0)
        }
        TutorialDone::GearCrafted { item } => {
            session.economy.gear_stock.get(item).copied().unwrap_or(0) > 0
                || session
                    .creatures
                    .iter()
                    .any(|c| c.equipment.as_deref() == Some(item.as_str()))
        }
        TutorialDone::WormAwake => session.worm_awake,
    }
}

#[cfg(test)]
mod tests;
