//! Inspection copy for passive local-room structures.

use crate::data::GameData;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::dark;
use macroquad_toolkit::ui::{draw_ui_text_ex, TextStyle};

pub(super) fn draw_rest_hollow_inspection(data: &GameData, x: f32, y: &mut f32) {
    draw_ui_text_ex(
        &format!(
            "Local capacity +{} workers",
            data.balance.rest_hollow_capacity
        ),
        x,
        *y,
        TextStyle::new(14.0, dark::POSITIVE).params(),
    );
    *y += 20.0;
    for text in ["Adds room without a worker", "More room steadies morale"] {
        draw_ui_text_ex(text, x, *y, TextStyle::new(14.0, dark::TEXT_DIM).params());
        *y += 20.0;
    }
}
