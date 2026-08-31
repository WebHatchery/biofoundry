//! Compact and desktop inspection controls for the Blacksmith.

use super::workstations::{
    blacksmith_equipment_label, blacksmith_queue_available, local_smith_staffed_at,
    local_smith_worker_at,
};
use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::structures::Building;
use crate::state::GameSession;
use crate::ui::hud::widgets::hud_button;
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) struct BlacksmithInspection<'a> {
    pub(super) session: &'a GameSession,
    pub(super) data: &'a GameData,
    pub(super) building: &'a Building,
    pub(super) pos: TilePos,
    pub(super) x: f32,
    pub(super) y: &'a mut f32,
    pub(super) panel_width: f32,
    pub(super) mouse: Vec2,
    pub(super) compact: bool,
    pub(super) button_height: f32,
    pub(super) button_step: f32,
    pub(super) line_step: f32,
    pub(super) actions: &'a mut Vec<UiAction>,
}

pub(super) fn draw_blacksmith_inspection(args: BlacksmithInspection<'_>) {
    let BlacksmithInspection {
        session,
        data,
        building,
        pos,
        x,
        y,
        panel_width,
        mouse,
        compact,
        button_height,
        button_step,
        line_step,
        actions,
    } = args;
    let line = |text: &str, color: Color, y: &mut f32| {
        draw_ui_text_ex(text, x, *y, TextStyle::new(14.0, color).params());
        *y += line_step;
    };
    let working = session
        .creatures
        .iter()
        .any(|c| local_smith_worker_at(c, pos));
    let staffed = session
        .creatures
        .iter()
        .any(|c| local_smith_staffed_at(c, pos));
    line(
        if working {
            "Smith at work"
        } else if staffed {
            "Smith stationed"
        } else {
            "No smith — idle"
        },
        if staffed {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
        y,
    );

    let queue_available = blacksmith_queue_available(building, data);
    line(
        &format!(
            "Ore {:.0}  Ingots {:.0}  Queue {}/{}",
            building.stock(Good::Ore),
            building.stock(Good::Ingot),
            building.orders.len(),
            data.balance.order_queue_size,
        ),
        dark::TEXT,
        y,
    );
    if !queue_available {
        line("Queue full · finish orders first", dark::WARNING, y);
    }

    // On a narrow screen, two columns preserve the complete recipe catalogue
    // below the tutorial card. The labels deliberately keep the recipe name,
    // cost, and compact unlock marker, while the full-width desktop card keeps
    // its original single-column reading order.
    *y += 2.0;
    let button_width = panel_width - 28.0;
    let (column_width, column_gap) = if compact {
        ((button_width - 6.0) * 0.5, 6.0)
    } else {
        (button_width, 0.0)
    };
    for (index, eq) in data.equipment.iter().enumerate() {
        let banked = session.economy.gear_stock.get(&eq.id).copied().unwrap_or(0);
        let queued = building.orders.iter().filter(|o| **o == eq.id).count();
        let unlocked = session.equipment_unlocked(eq);
        let label = blacksmith_equipment_label(data, eq, compact, unlocked, queued, banked);
        let (column, row) = if compact {
            (index % 2, index / 2)
        } else {
            (0, index)
        };
        let button_x = x + column as f32 * (column_width + column_gap);
        let button_y = *y + row as f32 * button_step;
        if hud_button(
            Rect::new(button_x, button_y, column_width, button_height),
            &label,
            queue_available && unlocked,
            mouse,
        ) {
            actions.push(UiAction::QueueOrder(pos, eq.id.clone()));
        }
    }
}
