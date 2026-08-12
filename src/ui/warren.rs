//! Warren world rendering: tiles, buildings, build sites, dig marks, and
//! creatures in camera space. Pure view — reads the session and draws.

use crate::data::GameData;
use crate::state::structures::Building;
use crate::state::world::Tile;
use crate::state::GameSession;
use crate::ui::UiMode;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
mod sprites;

pub use sprites::WorldSprites;

/// Draw the world in camera space. `hover` is the tile under the cursor
/// when the pointer is free (used for build/dig ghosts).
pub fn draw_world(
    session: &GameSession,
    data: &GameData,
    sprites: &WorldSprites,
    tile_size: f32,
    mode: &UiMode,
    hover: Option<TilePos>,
) {
    draw_tiles(session, sprites, tile_size);
    draw_dig_marks(session, tile_size);
    for building in &session.buildings {
        draw_building(session, data, sprites, building, tile_size);
    }
    for building in &session.buildings {
        if let Some(status) = crate::ui::legibility::building_status(session, data, building) {
            draw_status_badge(building.pos, tile_size, status);
        }
    }
    draw_build_sites(session, tile_size);
    // Overseer auras: a faint ring shows the work-speed field it radiates.
    for c in &session.creatures {
        if c.species == "overseer" {
            draw_circle_lines(
                c.x * tile_size,
                c.y * tile_size,
                data.balance.overseer_aura_radius * tile_size,
                2.0,
                Color::new(0.90, 0.75, 0.35, 0.28),
            );
        }
    }
    for creature in &session.creatures {
        sprites::draw_creature(creature, sprites, session.tick, tile_size);
    }
    for wild in &session.wilds {
        sprites::draw_wild(wild, sprites, tile_size);
    }
    if session.worm_awake {
        draw_colossal_worm(session, sprites, tile_size);
    }
    draw_tool_ghost(session, tile_size, mode, hover);
}

/// The awakened monument rises from its shrine as a proper illustrated
/// landmark, rather than a procedural ring of segments.
fn draw_colossal_worm(session: &GameSession, sprites: &WorldSprites, ts: f32) {
    let Some(shrine) = session.buildings_of("worm_shrine").next() else {
        return;
    };
    let (sx, sy) = (
        shrine.pos.x as f32 * ts + ts * 0.5,
        shrine.pos.y as f32 * ts + ts * 0.5,
    );
    sprites::draw_colossal_worm(sprites, vec2(sx, sy), session.tick, ts);
}

fn draw_tiles(session: &GameSession, sprites: &WorldSprites, ts: f32) {
    for (pos, tile) in session.world.tiles.iter_with_pos() {
        let x = pos.x as f32 * ts;
        let y = pos.y as f32 * ts;
        draw_rectangle(x, y, ts, ts, tile_color(*tile));
        sprites::draw_terrain_tile(sprites, *tile, pos, ts);

        match tile {
            Tile::MushroomPatch if session.patch_regrow.get(&pos).is_some_and(|t| *t > 0.0) => {
                draw_rectangle(x, y, ts, ts, Color::new(0.09, 0.07, 0.08, 0.42));
            }
            Tile::Sporewood if session.sporewood_regrow.get(&pos).is_some_and(|t| *t > 0.0) => {
                draw_rectangle(x, y, ts, ts, Color::new(0.09, 0.07, 0.08, 0.42));
            }
            _ => {}
        }
    }
}

fn tile_color(tile: Tile) -> Color {
    match tile {
        Tile::Rock => Color::new(0.13, 0.12, 0.14, 1.0),
        Tile::Floor => Color::new(0.24, 0.20, 0.17, 1.0),
        Tile::Water => Color::new(0.16, 0.30, 0.42, 1.0),
        Tile::MushroomPatch => Color::new(0.24, 0.20, 0.17, 1.0),
        Tile::OreVein => Color::new(0.20, 0.17, 0.16, 1.0),
        Tile::Sporewood => Color::new(0.22, 0.21, 0.16, 1.0),
    }
}

fn draw_dig_marks(session: &GameSession, ts: f32) {
    for mark in &session.dig_marks {
        let x = mark.x as f32 * ts;
        let y = mark.y as f32 * ts;
        draw_rectangle(x, y, ts, ts, Color::new(0.95, 0.75, 0.35, 0.22));
        draw_rectangle_lines(
            x + 2.0,
            y + 2.0,
            ts - 4.0,
            ts - 4.0,
            2.0,
            Color::new(0.95, 0.75, 0.35, 0.8),
        );
    }
}

fn draw_building(
    session: &GameSession,
    data: &GameData,
    sprites: &WorldSprites,
    building: &Building,
    ts: f32,
) {
    use crate::state::creatures::Good;
    let (x, y) = (building.pos.x as f32 * ts, building.pos.y as f32 * ts);
    if sprites::draw_building_sprite(sprites, building, x, y, ts) {
        return;
    }
    match building.kind.as_str() {
        "farm" => {
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
        "cook_pot" => {
            let (cx, cy) = (x + ts * 0.5, y + ts * 0.5);
            draw_circle(cx, cy, ts * 0.34, Color::new(0.16, 0.12, 0.10, 1.0));
            draw_circle_lines(cx, cy, ts * 0.34, 2.0, Color::new(0.85, 0.55, 0.25, 1.0));
            if building.stock(Good::Mushroom) >= 1.0 {
                draw_circle(cx, cy, ts * 0.14, Color::new(0.85, 0.75, 0.55, 1.0));
            }
        }
        "blacksmith" => {
            // An anvil block on a stone base; an ore pip and an ingot pip
            // show the input/output buffers.
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
            // Anvil silhouette.
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
        "mine" => {
            // A dark stone housing with an ore-hued mouth; a pile of dots
            // shows the local buffer, greyed out once the deposit runs dry.
            let spent = building.reserve <= 0.0;
            let frame = if spent {
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
            // Dark shaft mouth.
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
        "kiln" => {
            draw_rectangle(
                x + 3.0,
                y + 3.0,
                ts - 6.0,
                ts - 6.0,
                Color::new(0.22, 0.20, 0.20, 1.0),
            );
            // Mouth glows while wood smoulders inside.
            let glowing = building.stock(Good::Wood) > 0.0;
            let mouth = if glowing {
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
        "smelter" => {
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
        "trap" => {
            // Snare jaws: two arcs facing each other.
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
        "study_pen" => {
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
        "breeding_pit" => {
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
            draw_circle(
                x + ts * 0.40,
                y + ts * 0.52,
                ts * 0.08,
                Color::new(0.62, 0.40, 0.75, 1.0),
            );
            draw_circle(
                x + ts * 0.60,
                y + ts * 0.48,
                ts * 0.08,
                Color::new(0.62, 0.40, 0.75, 1.0),
            );
        }
        "worm_shrine" => {
            let (cx, cy) = (x + ts * 0.5, y + ts * 0.5);
            // A ring of standing stones around a dark maw.
            draw_circle(cx, cy, ts * 0.38, Color::new(0.10, 0.08, 0.12, 1.0));
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.0;
                draw_circle(
                    cx + a.cos() * ts * 0.34,
                    cy + a.sin() * ts * 0.34,
                    ts * 0.07,
                    Color::new(0.55, 0.50, 0.62, 1.0),
                );
            }
            // Offering progress ring.
            let frac = (session.worm_fed / data.balance.worm_awaken_at).clamp(0.0, 1.0);
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
        "stockpile" => {
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
        _ => {}
    }
}

/// A distinct shape + colour badge at the building's top-right corner, so a
/// stalled node is diagnosable at a glance (plan §Phase 9 — the equivalent
/// of Factorio's no-power icon). Shape and colour both encode the problem.
fn draw_status_badge(pos: TilePos, ts: f32, status: crate::ui::legibility::BuildingStatus) {
    use crate::ui::legibility::BuildingStatus as St;
    let bx = pos.x as f32 * ts + ts * 0.80;
    let by = pos.y as f32 * ts + ts * 0.20;
    let r = ts * 0.17;
    // Dark backing disc for contrast against any tile.
    draw_circle(bx, by, r + 1.5, Color::new(0.08, 0.08, 0.10, 0.92));

    let color = match status {
        St::NoWorker => Color::new(0.95, 0.85, 0.30, 1.0),
        St::InputStarved => Color::new(0.95, 0.55, 0.20, 1.0),
        St::OutputFull => Color::new(0.92, 0.32, 0.26, 1.0),
        St::Exhausted => Color::new(0.60, 0.60, 0.66, 1.0),
        St::AwaitingHaul => Color::new(0.40, 0.80, 0.92, 1.0),
    };
    let s = r * 0.85;
    match status {
        // Backed up: a full up-triangle.
        St::OutputFull => draw_triangle(
            vec2(bx, by - s),
            vec2(bx - s, by + s * 0.7),
            vec2(bx + s, by + s * 0.7),
            color,
        ),
        // Starved: an empty down-triangle.
        St::InputStarved => draw_triangle(
            vec2(bx, by + s),
            vec2(bx - s, by - s * 0.7),
            vec2(bx + s, by - s * 0.7),
            color,
        ),
        // No worker: an empty ring (a vacant post).
        St::NoWorker => draw_circle_lines(bx, by, s, 2.0, color),
        // Exhausted: a cross.
        St::Exhausted => {
            draw_line(bx - s, by - s, bx + s, by + s, 2.0, color);
            draw_line(bx - s, by + s, bx + s, by - s, 2.0, color);
        }
        // Awaiting haul: a little crate.
        St::AwaitingHaul => {
            draw_rectangle(bx - s * 0.75, by - s * 0.75, s * 1.5, s * 1.5, color);
            draw_rectangle_lines(
                bx - s * 0.75,
                by - s * 0.75,
                s * 1.5,
                s * 1.5,
                1.0,
                Color::new(0.1, 0.2, 0.25, 1.0),
            );
        }
    }
}

fn draw_build_sites(session: &GameSession, ts: f32) {
    for site in &session.build_sites {
        let x = site.pos.x as f32 * ts;
        let y = site.pos.y as f32 * ts;
        draw_rectangle(x, y, ts, ts, Color::new(0.55, 0.75, 0.95, 0.15));
        draw_rectangle_lines(
            x + 2.0,
            y + 2.0,
            ts - 4.0,
            ts - 4.0,
            2.0,
            Color::new(0.55, 0.75, 0.95, 0.9),
        );
        // Delivery progress: a fill bar along the bottom edge.
        let frac = if site.ore_needed == 0 {
            1.0
        } else {
            site.ore_delivered as f32 / site.ore_needed as f32
        };
        draw_rectangle(
            x + 3.0,
            y + ts - 7.0,
            (ts - 6.0) * frac.clamp(0.0, 1.0),
            4.0,
            Color::new(0.55, 0.85, 0.55, 0.95),
        );
    }
}

fn draw_tool_ghost(session: &GameSession, ts: f32, mode: &UiMode, hover: Option<TilePos>) {
    let Some(tile) = hover else {
        return;
    };
    let x = tile.x as f32 * ts;
    let y = tile.y as f32 * ts;
    match mode {
        UiMode::Build(kind) => {
            let ok = session.can_place_kind(kind, tile);
            let color = if ok {
                Color::new(0.45, 0.9, 0.5, 0.9)
            } else {
                Color::new(0.9, 0.35, 0.3, 0.9)
            };
            draw_rectangle_lines(x + 1.0, y + 1.0, ts - 2.0, ts - 2.0, 3.0, color);
        }
        UiMode::Dig => {
            let diggable = session
                .world
                .tiles
                .get(tile)
                .is_some_and(|t| matches!(t, Tile::Rock | Tile::OreVein));
            let color = if diggable {
                Color::new(0.95, 0.75, 0.35, 0.9)
            } else {
                Color::new(0.6, 0.6, 0.6, 0.6)
            };
            draw_rectangle_lines(x + 1.0, y + 1.0, ts - 2.0, ts - 2.0, 3.0, color);
        }
        UiMode::Inspect => {}
    }
}
