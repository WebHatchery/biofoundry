//! Warren world rendering: tiles, buildings, build sites, dig marks, and
//! creatures in camera space. Pure view — reads the session and draws.

use crate::data::GameData;
use crate::state::creatures::{Creature, Job};
use crate::state::structures::Building;
use crate::state::world::Tile;
use crate::state::GameSession;
use crate::ui::UiMode;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::sprite::SpriteAtlas;

const CREATURE_ATLAS_BYTES: &[u8] = include_bytes!("../../assets/sprites/creature-atlas.png");
const BUILDING_ATLAS_BYTES: &[u8] = include_bytes!("../../assets/sprites/building-atlas.png");

/// The hand-painted workers that turn the colony's logistics into something
/// the player can read at a glance. The atlas has three columns and two rows.
#[derive(Debug, Clone)]
pub struct WorldSprites {
    creatures: SpriteAtlas,
    buildings: SpriteAtlas,
}

impl WorldSprites {
    pub fn load() -> Self {
        let texture = Texture2D::from_file_with_format(CREATURE_ATLAS_BYTES, None);
        texture.set_filter(FilterMode::Linear);
        let building_texture = Texture2D::from_file_with_format(BUILDING_ATLAS_BYTES, None);
        building_texture.set_filter(FilterMode::Linear);
        Self {
            creatures: SpriteAtlas::new(texture, 512.0, 512.0),
            buildings: SpriteAtlas::new(building_texture, 512.0, 512.0),
        }
    }
}

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
    draw_tiles(session, tile_size);
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
        draw_creature(creature, sprites, session.tick, tile_size);
    }
    for wild in &session.wilds {
        draw_wild(wild, tile_size);
    }
    if session.worm_awake {
        draw_colossal_worm(session, tile_size);
    }
    draw_tool_ghost(session, tile_size, mode, hover);
}

/// The awakened monument: a vast segmented worm coiling out of the
/// shrine — cosmetic, but it earns its screen space.
fn draw_colossal_worm(session: &GameSession, ts: f32) {
    let Some(shrine) = session.buildings_of("worm_shrine").next() else {
        return;
    };
    let (sx, sy) = (
        shrine.pos.x as f32 * ts + ts * 0.5,
        shrine.pos.y as f32 * ts + ts * 0.5,
    );
    let t = session.tick as f32 * 0.02;
    for i in (0..12).rev() {
        let u = i as f32;
        let angle = t + u * 0.55;
        let radius = ts * (0.6 + u * 0.34);
        let px = sx + angle.cos() * radius;
        let py = sy + angle.sin() * radius * 0.55;
        let size = ts * (0.42 - u * 0.02);
        draw_circle(px, py, size, Color::new(0.42, 0.32, 0.45, 1.0));
        draw_circle_lines(px, py, size, 2.0, Color::new(0.70, 0.55, 0.85, 0.9));
    }
    // The head, nearest the shrine.
    draw_circle(
        sx + t.cos() * ts * 0.6,
        sy + t.sin() * ts * 0.33,
        ts * 0.46,
        Color::new(0.50, 0.38, 0.55, 1.0),
    );
}

fn draw_wild(wild: &crate::state::wildlife::WildCreature, ts: f32) {
    let x = wild.x * ts;
    let y = wild.y * ts;
    match wild.species.as_str() {
        "gnarl" => {
            draw_circle(x, y, ts * 0.30, Color::new(0.55, 0.14, 0.12, 1.0));
            draw_circle_lines(x, y, ts * 0.30, 2.0, Color::new(0.95, 0.35, 0.25, 1.0));
            // Hungry eyes.
            draw_circle(
                x - ts * 0.09,
                y - ts * 0.06,
                ts * 0.045,
                Color::new(1.0, 0.85, 0.3, 1.0),
            );
            draw_circle(
                x + ts * 0.09,
                y - ts * 0.06,
                ts * 0.045,
                Color::new(1.0, 0.85, 0.3, 1.0),
            );
        }
        _ => {
            // Wild beetle: like a hauler, but ringed white (undomesticated).
            draw_circle(x, y, ts * 0.28, Color::new(0.45, 0.30, 0.55, 1.0));
            draw_circle_lines(x, y, ts * 0.28, 2.0, Color::new(0.95, 0.95, 0.95, 0.9));
        }
    }
}

fn draw_tiles(session: &GameSession, ts: f32) {
    for (pos, tile) in session.world.tiles.iter_with_pos() {
        let x = pos.x as f32 * ts;
        let y = pos.y as f32 * ts;
        draw_rectangle(x, y, ts, ts, tile_color(*tile));

        match tile {
            Tile::MushroomPatch => {
                let grown = session
                    .patch_regrow
                    .get(&pos)
                    .is_none_or(|regrow| *regrow <= 0.0);
                let color = if grown {
                    Color::new(0.85, 0.75, 0.55, 1.0)
                } else {
                    Color::new(0.45, 0.40, 0.32, 1.0)
                };
                draw_circle(x + ts * 0.5, y + ts * 0.5, ts * 0.2, color);
            }
            Tile::OreVein => {
                let inset = ts * 0.28;
                draw_rectangle(
                    x + inset,
                    y + inset,
                    ts - inset * 2.0,
                    ts - inset * 2.0,
                    Color::new(0.75, 0.62, 0.35, 1.0),
                );
            }
            Tile::Sporewood => {
                let grown = session
                    .sporewood_regrow
                    .get(&pos)
                    .is_none_or(|regrow| *regrow <= 0.0);
                let color = if grown {
                    Color::new(0.42, 0.55, 0.30, 1.0)
                } else {
                    Color::new(0.30, 0.34, 0.24, 1.0)
                };
                // A stubby fungal trunk with a cap.
                draw_rectangle(x + ts * 0.42, y + ts * 0.4, ts * 0.16, ts * 0.4, color);
                draw_circle(x + ts * 0.5, y + ts * 0.38, ts * 0.22, color);
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

/// Returns whether the building has a painted counterpart in the atlas. The
/// remaining structures keep their procedural icon until their sprite pass.
fn draw_building_sprite(
    sprites: &WorldSprites,
    building: &Building,
    x: f32,
    y: f32,
    ts: f32,
) -> bool {
    let Some(frame) = (match building.kind.as_str() {
        "farm" => Some(0),
        "cook_pot" => Some(1),
        "blacksmith" => Some(2),
        "mine" => Some(3),
        "smelter" => Some(4),
        "stockpile" => Some(5),
        _ => None,
    }) else {
        return false;
    };
    let size = ts * 1.54;
    let center = vec2(x + ts * 0.5, y + ts * 0.51);
    draw_ellipse(
        center.x,
        center.y + size * 0.24,
        size * 0.34,
        size * 0.10,
        0.0,
        Color::new(0.02, 0.015, 0.02, 0.38),
    );
    sprites
        .buildings
        .draw_frame(frame, center, vec2(size, size), false, WHITE);
    true
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
    if draw_building_sprite(sprites, building, x, y, ts) {
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

fn draw_creature(creature: &Creature, sprites: &WorldSprites, tick: u64, ts: f32) {
    let x = creature.x * ts;
    let y = creature.y * ts;
    let (frame, size) = creature_sprite(creature, ts);
    let moving = !creature.path.is_empty();
    let phase = tick as f32 * 0.22 + creature.id as f32 * 1.7;
    let bob = if moving {
        phase.sin() * ts * 0.045
    } else {
        phase.sin() * ts * 0.018
    };
    let radius = size.x * 0.30;

    // Soft shadows put the workers into the cave rather than on top of it.
    draw_ellipse(
        x,
        y + size.y * 0.28,
        size.x * 0.31,
        size.y * 0.10,
        0.0,
        Color::new(0.02, 0.015, 0.02, 0.42),
    );
    sprites
        .creatures
        .draw_frame(frame, vec2(x, y + bob), size, false, WHITE);

    // Hunger telegraph: amber ring when hungry, red when starving.
    if creature.satiation <= 0.33 {
        draw_circle_lines(x, y, radius + 2.0, 2.0, dark::NEGATIVE);
    } else if creature.satiation <= 0.66 {
        draw_circle_lines(x, y, radius + 2.0, 2.0, dark::WARNING);
    }

    // Equipped gear: a bright glint on the shoulder marks an upgraded worker.
    if creature.equipment.is_some() {
        draw_circle(
            x + radius * 0.72,
            y - radius * 0.78 + bob,
            ts * 0.07,
            Color::new(0.95, 0.9, 0.6, 1.0),
        );
        draw_circle_lines(
            x + radius * 0.72,
            y - radius * 0.78 + bob,
            ts * 0.07,
            1.0,
            Color::new(0.5, 0.42, 0.2, 0.9),
        );
    }

    // Cargo remains physical at game scale: bright goods swell into an
    // oversized bundle rather than disappearing into a number in the HUD.
    if let Some((good, _)) = creature.carrying {
        draw_cargo(x, y + bob, radius, good, ts);
    }
}

fn creature_sprite(creature: &Creature, ts: f32) -> (usize, Vec2) {
    let frame = match creature.species.as_str() {
        "beetle" => 2,
        "salamander" => 3,
        "hobgoblin" => 4,
        "overseer" => 5,
        _ if creature.job == Job::Miner => 1,
        _ => 0,
    };
    let scale = match frame {
        2 | 4 => 1.62,
        1 | 3 => 1.42,
        _ => 1.28,
    };
    (frame, vec2(ts * scale, ts * scale))
}

fn draw_cargo(x: f32, y: f32, radius: f32, good: crate::state::creatures::Good, ts: f32) {
    use crate::state::creatures::Good;
    let (cx, cy) = (x + radius * 0.56, y - radius * 0.48);
    match good {
        Good::Mushroom => {
            for (dx, dy, cap) in [(-0.12, 0.04, 0.11), (0.06, -0.05, 0.13), (0.17, 0.06, 0.09)] {
                draw_rectangle(
                    cx + dx * ts - ts * 0.022,
                    cy + dy * ts,
                    ts * 0.044,
                    ts * 0.11,
                    Color::new(0.78, 0.68, 0.46, 1.0),
                );
                draw_circle(
                    cx + dx * ts,
                    cy + dy * ts,
                    ts * cap,
                    Color::new(0.48, 0.22, 0.58, 1.0),
                );
            }
        }
        Good::Ore | Good::Charcoal | Good::Ingot => {
            let color = match good {
                Good::Ore => Color::new(0.74, 0.60, 0.34, 1.0),
                Good::Charcoal => Color::new(0.12, 0.12, 0.14, 1.0),
                Good::Ingot => Color::new(0.70, 0.76, 0.82, 1.0),
                _ => unreachable!(),
            };
            for (dx, dy) in [(-0.10, 0.03), (0.08, -0.04), (0.18, 0.07)] {
                draw_circle(cx + dx * ts, cy + dy * ts, ts * 0.105, color);
            }
        }
        Good::Wood => {
            for dy in [-0.04, 0.08] {
                draw_rectangle(
                    cx - ts * 0.12,
                    cy + dy * ts,
                    ts * 0.42,
                    ts * 0.09,
                    Color::new(0.46, 0.28, 0.14, 1.0),
                );
                draw_circle(
                    cx + ts * 0.30,
                    cy + dy * ts + ts * 0.045,
                    ts * 0.045,
                    Color::new(0.72, 0.50, 0.25, 1.0),
                );
            }
        }
    }
}
