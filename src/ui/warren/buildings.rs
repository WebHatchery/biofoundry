//! Procedural building sprites that have no authored atlas frame.

use super::sprites::WorldSprites;
use crate::data::GameData;
use crate::state::creatures::Good;
use crate::state::structures::Building;
use crate::state::GameSession;
use macroquad::prelude::*;

/// Draw one finished building. Authored atlas sprites take precedence; the
/// procedural fallback keeps newly added data-driven building kinds visible.
pub(super) fn draw(
    session: &GameSession,
    data: &GameData,
    sprites: &WorldSprites,
    building: &Building,
    tile_size: f32,
) {
    let (x, y) = (
        building.pos.x as f32 * tile_size,
        building.pos.y as f32 * tile_size,
    );
    if super::sprites::draw_building_sprite(sprites, building, x, y, tile_size) {
        return;
    }
    match building.kind.as_str() {
        "farm" => draw_farm(building, x, y, tile_size),
        "cook_pot" => draw_cook_pot(building, x, y, tile_size),
        "blacksmith" => draw_blacksmith(building, x, y, tile_size),
        "mine" => draw_mine(building, data, x, y, tile_size),
        "kiln" => draw_kiln(building, x, y, tile_size),
        "smelter" => draw_smelter(building, x, y, tile_size),
        "feeding_trough" => draw_feeding_trough(building, x, y, tile_size),
        "trap" => draw_trap(x, y, tile_size),
        "study_pen" => draw_study_pen(x, y, tile_size),
        "breeding_pit" => draw_breeding_pit(x, y, tile_size),
        "worm_shrine" => draw_worm_shrine(session, data, building, x, y, tile_size),
        "outpost" => draw_outpost(session, building, x, y, tile_size),
        "stockpile" => draw_stockpile(session, x, y, tile_size),
        _ => {}
    }
}

fn draw_farm(building: &Building, x: f32, y: f32, ts: f32) {
    draw_rectangle(
        x + 2.0,
        y + 2.0,
        ts - 4.0,
        ts - 4.0,
        Color::new(0.16, 0.30, 0.16, 1.0),
    );
    for i in 0..3 {
        let filled = building.stock(Good::Mushroom) >= (i as f32 + 1.0) * 4.0;
        let color = if filled {
            Color::new(0.80, 0.72, 0.50, 1.0)
        } else {
            Color::new(0.30, 0.42, 0.28, 1.0)
        };
        draw_circle(
            x + ts * (0.25 + 0.25 * i as f32),
            y + ts * 0.5,
            ts * 0.11,
            color,
        );
    }
}

fn draw_cook_pot(building: &Building, x: f32, y: f32, ts: f32) {
    let (cx, cy) = (x + ts * 0.5, y + ts * 0.5);
    draw_circle(cx, cy, ts * 0.34, Color::new(0.16, 0.12, 0.10, 1.0));
    draw_circle_lines(cx, cy, ts * 0.34, 2.0, Color::new(0.85, 0.55, 0.25, 1.0));
    if building.stock(Good::Mushroom) >= 1.0 {
        draw_circle(cx, cy, ts * 0.14, Color::new(0.85, 0.75, 0.55, 1.0));
    }
}

fn draw_blacksmith(building: &Building, x: f32, y: f32, ts: f32) {
    // An anvil block on a stone base; pips show the input and output buffers.
    draw_rectangle(
        x + 3.0,
        y + 3.0,
        ts - 6.0,
        ts - 6.0,
        Color::new(0.20, 0.19, 0.22, 1.0),
    );
    draw_rectangle_lines(
        x + 3.0,
        y + 3.0,
        ts - 6.0,
        ts - 6.0,
        2.0,
        Color::new(0.62, 0.64, 0.70, 0.9),
    );
    draw_rectangle(
        x + ts * 0.30,
        y + ts * 0.44,
        ts * 0.40,
        ts * 0.12,
        Color::new(0.45, 0.47, 0.52, 1.0),
    );
    draw_rectangle(
        x + ts * 0.42,
        y + ts * 0.54,
        ts * 0.16,
        ts * 0.14,
        Color::new(0.45, 0.47, 0.52, 1.0),
    );
    if building.stock(Good::Ore) >= 1.0 {
        draw_circle(
            x + ts * 0.28,
            y + ts * 0.30,
            ts * 0.08,
            Color::new(0.75, 0.62, 0.35, 1.0),
        );
    }
    if building.stock(Good::Ingot) >= 1.0 {
        draw_circle(
            x + ts * 0.72,
            y + ts * 0.30,
            ts * 0.08,
            Color::new(0.80, 0.82, 0.88, 1.0),
        );
    }
}

fn draw_mine(building: &Building, data: &GameData, x: f32, y: f32, ts: f32) {
    // A dark housing with an ore-hued mouth and a three-step buffer meter.
    let frame = if building.reserve <= 0.0 {
        Color::new(0.18, 0.17, 0.18, 1.0)
    } else {
        Color::new(0.26, 0.22, 0.16, 1.0)
    };
    draw_rectangle(x + 2.0, y + 2.0, ts - 4.0, ts - 4.0, frame);
    draw_rectangle_lines(
        x + 2.0,
        y + 2.0,
        ts - 4.0,
        ts - 4.0,
        2.0,
        Color::new(0.60, 0.50, 0.32, 0.9),
    );
    draw_circle(
        x + ts * 0.5,
        y + ts * 0.62,
        ts * 0.16,
        Color::new(0.08, 0.07, 0.06, 1.0),
    );
    let buffered = building.stock(Good::Ore);
    for i in 0..3 {
        if buffered >= (i as f32 + 1.0) * (data.balance.mine_buffer_cap / 3.0) {
            draw_circle(
                x + ts * (0.28 + 0.22 * i as f32),
                y + ts * 0.3,
                ts * 0.08,
                Color::new(0.80, 0.66, 0.36, 1.0),
            );
        }
    }
}

fn draw_kiln(building: &Building, x: f32, y: f32, ts: f32) {
    draw_rectangle(
        x + 3.0,
        y + 3.0,
        ts - 6.0,
        ts - 6.0,
        Color::new(0.22, 0.20, 0.20, 1.0),
    );
    let mouth = if building.stock(Good::Wood) > 0.0 {
        Color::new(0.95, 0.55, 0.20, 1.0)
    } else {
        Color::new(0.35, 0.30, 0.28, 1.0)
    };
    draw_circle(x + ts * 0.5, y + ts * 0.6, ts * 0.16, mouth);
    if building.stock(Good::Charcoal) >= 1.0 {
        draw_circle(
            x + ts * 0.75,
            y + ts * 0.28,
            ts * 0.10,
            Color::new(0.15, 0.15, 0.16, 1.0),
        );
    }
}

fn draw_smelter(building: &Building, x: f32, y: f32, ts: f32) {
    draw_rectangle(
        x + 3.0,
        y + 3.0,
        ts - 6.0,
        ts - 6.0,
        Color::new(0.30, 0.16, 0.13, 1.0),
    );
    draw_rectangle_lines(
        x + 3.0,
        y + 3.0,
        ts - 6.0,
        ts - 6.0,
        2.0,
        Color::new(0.85, 0.45, 0.25, 0.9),
    );
    if building.stock(Good::Ore) >= 1.0 {
        draw_circle(
            x + ts * 0.3,
            y + ts * 0.32,
            ts * 0.09,
            Color::new(0.75, 0.62, 0.35, 1.0),
        );
    }
    if building.stock(Good::Charcoal) >= 1.0 {
        draw_circle(
            x + ts * 0.7,
            y + ts * 0.32,
            ts * 0.09,
            Color::new(0.15, 0.15, 0.16, 1.0),
        );
    }
    if building.stock(Good::Ingot) >= 1.0 {
        draw_circle(
            x + ts * 0.5,
            y + ts * 0.68,
            ts * 0.09,
            Color::new(0.80, 0.82, 0.88, 1.0),
        );
    }
}

fn draw_feeding_trough(building: &Building, x: f32, y: f32, ts: f32) {
    draw_rectangle(
        x + 3.0,
        y + ts * 0.38,
        ts - 6.0,
        ts * 0.30,
        Color::new(0.46, 0.28, 0.16, 1.0),
    );
    draw_rectangle_lines(
        x + 3.0,
        y + ts * 0.38,
        ts - 6.0,
        ts * 0.30,
        2.0,
        Color::new(0.82, 0.55, 0.28, 0.9),
    );
    if building.stock(Good::CookedFood) > 0.0 {
        draw_rectangle(
            x + ts * 0.20,
            y + ts * 0.46,
            ts * 0.60,
            ts * 0.12,
            Color::new(0.84, 0.72, 0.46, 1.0),
        );
    }
}

fn draw_trap(x: f32, y: f32, ts: f32) {
    let (cx, cy) = (x + ts * 0.5, y + ts * 0.5);
    draw_circle_lines(cx, cy, ts * 0.26, 2.0, Color::new(0.75, 0.75, 0.78, 0.95));
    draw_line(
        cx - ts * 0.26,
        cy,
        cx + ts * 0.26,
        cy,
        2.0,
        Color::new(0.75, 0.75, 0.78, 0.7),
    );
}

fn draw_study_pen(x: f32, y: f32, ts: f32) {
    draw_rectangle_lines(
        x + 3.0,
        y + 3.0,
        ts - 6.0,
        ts - 6.0,
        2.0,
        Color::new(0.45, 0.65, 0.85, 0.95),
    );
    draw_circle(
        x + ts * 0.5,
        y + ts * 0.5,
        ts * 0.12,
        Color::new(0.45, 0.65, 0.85, 0.7),
    );
}

fn draw_breeding_pit(x: f32, y: f32, ts: f32) {
    draw_circle(
        x + ts * 0.5,
        y + ts * 0.5,
        ts * 0.34,
        Color::new(0.35, 0.20, 0.28, 1.0),
    );
    draw_circle_lines(
        x + ts * 0.5,
        y + ts * 0.5,
        ts * 0.34,
        2.0,
        Color::new(0.85, 0.45, 0.65, 0.95),
    );
    for (dx, dy) in [(0.40, 0.52), (0.60, 0.48)] {
        draw_circle(
            x + ts * dx,
            y + ts * dy,
            ts * 0.08,
            Color::new(0.62, 0.40, 0.75, 1.0),
        );
    }
}

fn draw_worm_shrine(
    session: &GameSession,
    data: &GameData,
    _building: &Building,
    x: f32,
    y: f32,
    ts: f32,
) {
    let (cx, cy) = (x + ts * 0.5, y + ts * 0.5);
    draw_circle(cx, cy, ts * 0.38, Color::new(0.10, 0.08, 0.12, 1.0));
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        draw_circle(
            cx + angle.cos() * ts * 0.34,
            cy + angle.sin() * ts * 0.34,
            ts * 0.07,
            Color::new(0.55, 0.50, 0.62, 1.0),
        );
    }
    let food_frac = session.worm_fed / data.balance.worm_awaken_at;
    let ingot_frac = session.worm_ingots_fed as f32 / data.balance.worm_awaken_ingots.max(1) as f32;
    let frac = food_frac.min(ingot_frac).clamp(0.0, 1.0);
    if frac > 0.0 {
        draw_circle_lines(
            cx,
            cy,
            ts * 0.46,
            3.0,
            Color::new(0.75, 0.55, 0.95, 0.35 + 0.6 * frac),
        );
    }
}

fn draw_outpost(session: &GameSession, building: &Building, x: f32, y: f32, ts: f32) {
    let active = session
        .outposts
        .iter()
        .find(|outpost| outpost.pos == building.pos)
        .is_some_and(|outpost| outpost.active);
    draw_rectangle(
        x + 4.0,
        y + 4.0,
        ts - 8.0,
        ts - 8.0,
        if active {
            Color::new(0.22, 0.30, 0.42, 1.0)
        } else {
            Color::new(0.20, 0.20, 0.24, 1.0)
        },
    );
    draw_circle(
        x + ts * 0.5,
        y + ts * 0.5,
        ts * 0.17,
        if active {
            Color::new(0.55, 0.82, 0.95, 1.0)
        } else {
            Color::new(0.48, 0.48, 0.54, 1.0)
        },
    );
}

fn draw_stockpile(session: &GameSession, x: f32, y: f32, ts: f32) {
    draw_rectangle_lines(
        x + 3.0,
        y + 3.0,
        ts - 6.0,
        ts - 6.0,
        2.0,
        Color::new(0.65, 0.66, 0.70, 0.9),
    );
    let dots = (session.economy.ore_stock.min(9) as usize).div_euclid(3);
    for i in 0..=dots {
        if session.economy.ore_stock > 0 {
            draw_circle(
                x + ts * (0.3 + 0.2 * i as f32),
                y + ts * 0.65,
                ts * 0.08,
                Color::new(0.75, 0.62, 0.35, 1.0),
            );
        }
    }
}
