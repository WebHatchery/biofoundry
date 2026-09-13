//! The building inspection card: what the clicked node is doing right now,
//! plus its per-kind verbs (blacksmith production orders, pit breeding).

use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::GameSession;
use crate::ui::hud::widgets::{hud_button, panel_style};
use crate::ui::legibility::{shrine_waiting_for_food, shrine_waiting_for_ingots, BuildingStatus};
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub mod blacksmith;
pub mod breeding;
pub mod layout;
pub mod outpost;
pub mod rooms;
pub mod status;
pub mod study;
pub mod workstations;

use blacksmith::{draw_blacksmith_inspection, BlacksmithInspection};
pub use breeding::{breed_button_label, breed_label, breeding_unlock_hint};
pub use layout::{blacksmith_recipe_rows, inspection_button_metrics, InspectionLayout};
pub use outpost::{
    compact_archive_summary, outpost_archive_summary, outpost_signal_cache_summary,
    outpost_waypoint_summary,
};
use outpost::{draw_compact_route_controls, draw_full_route_controls, FullRouteContext};
use rooms::draw_rest_hollow_inspection;
pub use status::inspect_status;
pub use status::{
    local_mine_staffed_at, local_mine_worker_at, mine_staffing_label,
    outpost_cargo_only_return_label, outpost_expedition_hint, outpost_expedition_hint_with_session,
    outpost_has_loadable_payload, outpost_load_hint, outpost_return_label, transit_destination,
    transit_payload_line, waste_inspection_hint,
};
pub use study::{study_adaptation_line, study_rate_per_min};
pub use workstations::{
    blacksmith_equipment_label, blacksmith_input_hint, blacksmith_queue_available,
    cook_pot_input_hint, equipment_lock_label, kiln_input_hint, local_smelter_staffed_at,
    local_smelter_worker_at, local_smith_staffed_at, local_smith_worker_at, smelter_input_hint,
};

pub const LOCKED_SPECIALIST_MARKER: &str = "[L]";

/// First-pass building inspection (plan §Phase 6): what a clicked building
/// is doing right now. Phase 9 grows this into the full legibility layer.
/// Returns its rect while a building is selected.
mod draw;
pub(crate) use draw::draw_inspect_panel;
