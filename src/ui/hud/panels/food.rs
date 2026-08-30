//! The Food Grid's compact calorie, ingredient, and factory ledger.

use crate::data::GameData;
use crate::simulation::food;
use crate::state::GameSession;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, format_mmss};

/// The calorie balance meter — production, consumption, and stockpile,
/// exactly like a power graph.
pub(in crate::ui::hud) fn draw_food_grid_panel(
    session: &GameSession,
    data: &GameData,
    panel: Rect,
) {
    draw_surface_with_title(
        panel,
        Some("Food Grid"),
        &super::panel_style(),
        TextStyle::new(17.0, dark::TEXT),
    );

    let production = session.economy.production_ema_per_min.max(0.0);
    let consumption = food::consumption_per_min(session, data);
    let net = production - consumption;
    let x = panel.x + 14.0;
    let mut y = panel.y + 50.0;

    draw_ui_text_ex(
        &format!(
            "Production  +{production:.1}/min · Raw {:.0}",
            session.economy.raw_food
        ),
        x,
        y,
        TextStyle::new(15.0, dark::POSITIVE).params(),
    );
    y += 21.0;
    draw_ui_text_ex(
        &format!("Upkeep      -{consumption:.1}/min"),
        x,
        y,
        TextStyle::new(15.0, dark::NEGATIVE).params(),
    );
    y += 21.0;
    let net_color = if net >= 0.0 {
        dark::POSITIVE
    } else {
        dark::NEGATIVE
    };
    draw_ui_text_ex(
        &format!("Net         {net:+.1}/min"),
        x,
        y,
        TextStyle::new(15.0, net_color).params(),
    );
    y += 15.0;

    super::meter(
        Rect::new(x, y, panel.w - 28.0, 18.0),
        session.economy.food,
        data.balance.win_food_surplus,
        if session.economy.food > 15.0 {
            dark::POSITIVE
        } else {
            dark::NEGATIVE
        },
        Some(&format!(
            "Cooked {:.0}/{:.0}",
            session.economy.food, data.balance.win_food_surplus
        )),
    );
    y += 30.0;

    let (forecast, forecast_color) = match food::time_to_empty_seconds(session, data) {
        Some(seconds) => (
            format!("Forecast: empty in {}", format_mmss(seconds)),
            if seconds <= 120.0 {
                dark::WARNING
            } else {
                dark::NEGATIVE
            },
        ),
        None if net >= 0.0 => ("Forecast: reserve rising".to_owned(), dark::POSITIVE),
        None => ("Forecast: reserve empty".to_owned(), dark::NEGATIVE),
    };
    draw_ui_text_ex(
        &forecast,
        x,
        y,
        TextStyle::new(14.0, forecast_color).params(),
    );
    y += 20.0;

    // Chain throughput + haul pressure: the food grid generalised to a
    // factory dashboard (plan §Phase 9).
    let hauls = crate::ui::legibility::pending_hauls(session);
    draw_ui_text_ex(
        &format!(
            "Ore +{:.0}/m · Ingots +{:.0}/m · Hauls {hauls}",
            session.economy.ore_ema_per_min.max(0.0),
            session.economy.ingot_ema_per_min.max(0.0),
        ),
        x,
        y,
        TextStyle::new(14.0, dark::TEXT_DIM).params(),
    );
    y += 20.0;
    draw_ui_text_ex(
        &resource_bank_line(session.economy.ore_stock, session.economy.ingots_stock),
        x,
        y,
        TextStyle::new(14.0, dark::TEXT).params(),
    );
}

fn resource_bank_line(ore: u32, ingots: u32) -> String {
    format!("Ore banked {ore} · ingots {ingots}")
}

#[cfg(test)]
mod tests;
