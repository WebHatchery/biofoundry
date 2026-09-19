//! Shared geometry and recipe rows for building inspection cards.

use crate::ui::{LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::Rect;

pub struct InspectionLayout {
    pub panel: Rect,
    pub compact_outpost: bool,
    pub content_x: f32,
    pub content_y: f32,
    pub outpost_button_height: f32,
    pub outpost_button_step: f32,
    pub line_step: f32,
}

impl InspectionLayout {
    pub fn new(
        kind: &str,
        compact: bool,
        equipment_count: usize,
        inspect_button_step: f32,
        top: f32,
    ) -> Self {
        let compact_outpost = compact && kind == "outpost";
        let height = inspection_panel_height(kind, compact, equipment_count, inspect_button_step);
        let panel = if compact_outpost {
            Rect::new(20.0, 0.0, LOGICAL_WIDTH - 40.0, height)
        } else if kind == "outpost" {
            // The late-game route ladder is substantially taller than other
            // inspections. Start below the top bar and own the full right
            // edge so every upgrade and recovery action stays on-canvas.
            Rect::new(LOGICAL_WIDTH - 262.0, 66.0, 250.0, LOGICAL_HEIGHT - 66.0)
        } else {
            Rect::new(LOGICAL_WIDTH - 262.0, top, 250.0, height)
        };
        let (outpost_button_height, outpost_button_step, line_step) = if compact_outpost {
            (72.0, 72.0, 14.0)
        } else if compact {
            (30.0, 30.0, 16.0)
        } else {
            (24.0, 26.0, 20.0)
        };

        Self {
            panel,
            compact_outpost,
            content_x: panel.x + if compact_outpost { 24.0 } else { 14.0 },
            content_y: panel.y + if compact_outpost { 70.0 } else { 50.0 },
            outpost_button_height,
            outpost_button_step,
            line_step,
        }
    }
}

pub fn inspection_panel_height(
    kind: &str,
    compact: bool,
    equipment_count: usize,
    inspect_button_step: f32,
) -> f32 {
    match kind {
        "material_stockpile" => 280.0,
        "blacksmith" if compact => {
            124.0 + blacksmith_recipe_rows(equipment_count) as f32 * inspect_button_step
        }
        "blacksmith" => 194.0 + equipment_count as f32 * inspect_button_step,
        "breeding_pit" if compact => 350.0,
        "breeding_pit" => 280.0,
        "worm_shrine" if compact => 250.0,
        "worm_shrine" => 240.0,
        "outpost" if compact => 720.0,
        "outpost" => 510.0,
        _ => 152.0,
    }
}

pub fn inspection_button_metrics(kind: &str, compact: bool, equipment_count: usize) -> (f32, f32) {
    if kind == "material_stockpile" {
        (72.0, 76.0)
    } else if compact && kind == "worm_shrine" {
        // The shrine is a critical-path handoff. Its pause/resume action must
        // remain a full touch target even when the canvas is 800x450.
        (72.0, 76.0)
    } else if compact && kind == "blacksmith" && equipment_count > 4 {
        (46.0, 48.0)
    } else if compact && matches!(kind, "blacksmith" | "breeding_pit") {
        (36.0, 40.0)
    } else if !compact && kind == "breeding_pit" {
        (38.0, 42.0)
    } else if compact {
        (30.0, 34.0)
    } else {
        (24.0, 26.0)
    }
}

pub fn blacksmith_recipe_rows(equipment_count: usize) -> usize {
    equipment_count.div_ceil(2)
}
