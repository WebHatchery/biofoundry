//! UI is a pure view layer: it reads state, draws, and returns `UiAction`
//! intents. It never mutates game state — `Game::apply_action` dispatches.

pub mod hud;
pub mod legibility;
pub mod menu;
pub mod warren;

use crate::state::creatures::Job;
use macroquad_toolkit::grid::TilePos;

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

/// What a world click means right now.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UiMode {
    #[default]
    Inspect,
    /// Placing a ghost of this building kind (id into `buildings.json`).
    Build(String),
    /// Toggling dig designations on rock.
    Dig,
}

impl UiMode {
    /// Building is a one-shot map action; return to inspection after a valid
    /// placement so the next tap can select a building.
    pub fn after_successful_placement(self) -> Self {
        match self {
            Self::Build(_) => Self::Inspect,
            mode => mode,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    StartWarren,
    /// Ask for confirmation before replacing an existing campaign save.
    RequestNewWarren,
    /// Close the new-warren confirmation without changing the save.
    CancelNewWarren,
    BackToMenu,
    /// Move one idle goblin into this job.
    Assign(Job),
    /// Move one goblin out of this job into the idle pool.
    Unassign(Job),
    AttractBeetle,
    AttractSalamander,
    AttractSlimeJanitor,
    AttractBatCourier,
    /// Breed a special creature (species id: "hobgoblin"/"overseer") at the
    /// Breeding Pit.
    Breed(String),
    ToggleShrineFeeding(TilePos),
    ActivateOutpost(TilePos),
    /// Rotate the outbound cargo fill order for an awakened worm route.
    CycleOutpostCargo(TilePos),
    /// Buy the one-time expanded cargo hold for an awakened worm route.
    UpgradeOutpost(TilePos),
    /// Cycle the number of new scouts sent on an awakened worm route.
    CycleOutpostCrew(TilePos),
    /// Pause or resume remote scouting without closing the worm route.
    ToggleOutpostExpedition(TilePos),
    /// Toggle automatic cargo-only returns when an awakened hold is full.
    ToggleOutpostAutoReturn(TilePos),
    /// Toggle automatic food-only resupply for an awakened remote crew.
    ToggleOutpostAutoResupply(TilePos),
    TransitToOutpost(TilePos),
    TransitToShrine(TilePos),
    /// Return an outpost's cargo while leaving its remote crew in place.
    TransitCargoToShrine(TilePos),
    DismissVictory,
    DismissFactory,
    DismissWorm,
    SkipTutorial,
    /// Toggle a tool mode (clicking the active mode returns to Inspect).
    SetMode(UiMode),
    /// Open or close the post-awakening route ledger.
    ToggleRoutes,
    /// Select a building from a HUD shortcut such as the route ledger.
    SelectBuilding(TilePos),
    /// The player clicked this world tile with the active tool.
    WorldClick(TilePos),
    /// Queue an equipment craft (item id) at the blacksmith at this tile.
    QueueOrder(TilePos, String),
    Save,
    Load,
    /// Open/close the settings panel on the title menu.
    ToggleSettings,
    /// Open/close the revisitable warren field guide.
    ToggleHelp,
    /// Pause or resume the fixed-timestep simulation while leaving the HUD
    /// and camera interactive.
    TogglePause,
    /// Nudge the sound volume by this many 10% steps.
    AdjustVolume(i8),
    /// Zoom the warren camera around the screen centre.
    ZoomCamera(i8),
    ExitGame,
}

/// One frame of HUD output.
pub struct HudFrame {
    pub actions: Vec<UiAction>,
    /// True when the pointer is over HUD chrome — world clicks should be
    /// ignored while true.
    pub pointer_over_ui: bool,
}

#[cfg(test)]
mod tests;
