//! Painted atlas loading and world-entity sprite drawing.

use crate::state::creatures::{Creature, Good, Job};
use crate::state::structures::Building;
use crate::state::wildlife::WildCreature;
use crate::state::world::Tile;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::dark;
use macroquad_toolkit::sprite::SpriteAtlas;

const CREATURE_ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/sprites/creature-atlas.png");
const BUILDING_ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/sprites/building-atlas.png");
const TERRAIN_ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/sprites/terrain-atlas.png");
const SPECIAL_ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/sprites/special-atlas.png");

/// Hand-painted workers and production props, packed as three-by-two atlases.
#[derive(Debug, Clone)]
pub struct WorldSprites {
    creatures: SpriteAtlas,
    buildings: SpriteAtlas,
    terrain: SpriteAtlas,
    special: SpriteAtlas,
}

impl WorldSprites {
    pub fn load() -> Self {
        let creatures = load_atlas(CREATURE_ATLAS_BYTES);
        let buildings = load_atlas(BUILDING_ATLAS_BYTES);
        let terrain = load_atlas(TERRAIN_ATLAS_BYTES);
        let special = load_atlas(SPECIAL_ATLAS_BYTES);
        Self {
            creatures,
            buildings,
            terrain,
            special,
        }
    }
}

/// Overlay the hand-painted terrain component over the quiet logical tile.
/// Floor repeats cleanly; resource frames stay sparse and high-contrast.
pub fn draw_terrain_tile(sprites: &WorldSprites, tile: Tile, pos: TilePos, ts: f32) {
    let (frame, scale) = match tile {
        Tile::Water => (2, 1.18),
        Tile::MushroomPatch => (3, 1.02),
        Tile::OreVein => (4, 1.04),
        Tile::Sporewood => (5, 1.06),
        Tile::Rock => return,
        Tile::Floor => return,
    };
    let center = vec2((pos.x as f32 + 0.5) * ts, (pos.y as f32 + 0.5) * ts);
    sprites.terrain.draw_frame(
        frame,
        center,
        vec2(ts * scale, ts * scale),
        pos.x.rem_euclid(2) == 0,
        WHITE,
    );
}

fn load_atlas(bytes: &[u8]) -> SpriteAtlas {
    let texture = Texture2D::from_file_with_format(bytes, None);
    texture.set_filter(FilterMode::Linear);
    SpriteAtlas::new(texture, 512.0, 512.0)
}

/// Returns whether the building has a painted counterpart in the atlas.
pub fn draw_building_sprite(
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
        "kiln" => Some(6),
        "trap" => Some(7),
        "study_pen" => Some(8),
        "breeding_pit" => Some(9),
        "worm_shrine" => Some(10),
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
    let atlas = if frame < 6 {
        &sprites.buildings
    } else {
        &sprites.special
    };
    atlas.draw_frame(frame % 6, center, vec2(size, size), false, WHITE);
    true
}

pub fn draw_wild(wild: &WildCreature, sprites: &WorldSprites, ts: f32) {
    let center = vec2(wild.x * ts, wild.y * ts);
    let (atlas, frame, size, ring) = match wild.species.as_str() {
        "gnarl" => (
            &sprites.special,
            5,
            ts * 1.34,
            Color::new(0.95, 0.35, 0.25, 0.95),
        ),
        _ => (
            &sprites.creatures,
            2,
            ts * 1.32,
            Color::new(0.95, 0.95, 0.95, 0.90),
        ),
    };
    draw_ellipse(
        center.x,
        center.y + size * 0.28,
        size * 0.30,
        size * 0.09,
        0.0,
        Color::new(0.02, 0.015, 0.02, 0.44),
    );
    atlas.draw_frame(frame, center, vec2(size, size), false, WHITE);
    draw_circle_lines(center.x, center.y, size * 0.30, 2.0, ring);
}

pub fn draw_creature(creature: &Creature, sprites: &WorldSprites, tick: u64, ts: f32) {
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

    if creature.satiation <= 0.33 {
        draw_circle_lines(x, y, radius + 2.0, 2.0, dark::NEGATIVE);
    } else if creature.satiation <= 0.66 {
        draw_circle_lines(x, y, radius + 2.0, 2.0, dark::WARNING);
    }
    if creature.equipment.is_some() {
        draw_gear_glint(x, y + bob, radius, ts);
    }
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

fn draw_gear_glint(x: f32, y: f32, radius: f32, ts: f32) {
    let (gx, gy) = (x + radius * 0.72, y - radius * 0.78);
    draw_circle(gx, gy, ts * 0.07, Color::new(0.95, 0.9, 0.6, 1.0));
    draw_circle_lines(gx, gy, ts * 0.07, 1.0, Color::new(0.5, 0.42, 0.2, 0.9));
}

fn draw_cargo(x: f32, y: f32, radius: f32, good: Good, ts: f32) {
    let (cx, cy) = (x + radius * 0.56, y - radius * 0.48);
    match good {
        Good::Mushroom => draw_mushrooms(cx, cy, ts),
        Good::Ore | Good::Charcoal | Good::Ingot => draw_minerals(cx, cy, good, ts),
        Good::Wood => draw_logs(cx, cy, ts),
    }
}

fn draw_mushrooms(cx: f32, cy: f32, ts: f32) {
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

fn draw_minerals(cx: f32, cy: f32, good: Good, ts: f32) {
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

fn draw_logs(cx: f32, cy: f32, ts: f32) {
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
