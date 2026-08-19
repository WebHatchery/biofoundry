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
const COLOSSAL_WORM_BYTES: &[u8] = include_bytes!("../../../assets/sprites/colossal-worm.png");
const CARGO_ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/sprites/cargo-atlas.png");
const ROLE_PROP_ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/sprites/role-prop-atlas.png");
const GROUND_ATLAS_BYTES: &[u8] = include_bytes!("../../../assets/sprites/ground-atlas.png");

/// Hand-painted workers and production props, packed as three-by-two atlases.
#[derive(Debug, Clone)]
pub struct WorldSprites {
    creatures: SpriteAtlas,
    buildings: SpriteAtlas,
    terrain: SpriteAtlas,
    special: SpriteAtlas,
    colossal_worm: Texture2D,
    cargo: SpriteAtlas,
    role_props: SpriteAtlas,
    ground: SpriteAtlas,
}

impl WorldSprites {
    pub fn load() -> Self {
        let creatures = load_atlas(CREATURE_ATLAS_BYTES);
        let buildings = load_atlas(BUILDING_ATLAS_BYTES);
        let terrain = load_atlas(TERRAIN_ATLAS_BYTES);
        let special = load_atlas(SPECIAL_ATLAS_BYTES);
        let colossal_worm = Texture2D::from_file_with_format(COLOSSAL_WORM_BYTES, None);
        colossal_worm.set_filter(FilterMode::Linear);
        let cargo = load_atlas(CARGO_ATLAS_BYTES);
        let role_props = load_atlas(ROLE_PROP_ATLAS_BYTES);
        let ground = load_atlas(GROUND_ATLAS_BYTES);
        Self {
            creatures,
            buildings,
            terrain,
            special,
            colossal_worm,
            cargo,
            role_props,
            ground,
        }
    }
}

/// Draw the awakened worm as a single monumental illustration, with only a
/// small breathing motion so it remains a landmark rather than visual noise.
pub fn draw_colossal_worm(sprites: &WorldSprites, center: Vec2, tick: u64, ts: f32) {
    let pulse = (tick as f32 * 0.028).sin();
    let size = ts * (6.35 + pulse * 0.10);
    draw_texture_ex(
        &sprites.colossal_worm,
        center.x - size * 0.5,
        center.y - size * 0.53 + pulse * ts * 0.06,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
}

/// Overlay hand-painted terrain over the quiet logical tile. The base fill
/// supplies a stable grid for interaction; the atlas breaks its prototype
/// rectangles into cracked earth and irregular cave faces.
pub fn draw_terrain_tile(sprites: &WorldSprites, tile: Tile, pos: TilePos, ts: f32) {
    let (frame, scale) = match tile {
        // Ground cells are authored as repeatable textures. Keep them inside
        // their logical tile so one cell cannot cover its neighbour's edge.
        Tile::Rock => (3, 1.0),
        Tile::Floor => (0, 1.0),
        Tile::Water => (2, 1.18),
        Tile::MushroomPatch => (3, 1.02),
        Tile::OreVein => (4, 1.04),
        Tile::Sporewood => (5, 1.06),
    };
    let center = vec2((pos.x as f32 + 0.5) * ts, (pos.y as f32 + 0.5) * ts);
    let is_ground = matches!(tile, Tile::Rock | Tile::Floor);
    let substrate_frame = match tile {
        Tile::Water => Some(3),
        Tile::OreVein => Some(3),
        Tile::MushroomPatch | Tile::Sporewood => Some(0),
        _ => None,
    };
    let atlas = if is_ground {
        &sprites.ground
    } else {
        &sprites.terrain
    };
    let tint = match tile {
        Tile::Rock | Tile::Floor => Color::new(1.0, 1.0, 1.0, 0.72),
        _ => WHITE,
    };
    if let Some(substrate_frame) = substrate_frame {
        draw_ground_patch(
            &sprites.ground,
            substrate_frame,
            pos,
            center,
            ts,
            Color::new(1.0, 1.0, 1.0, 0.72),
        );
    }
    if is_ground {
        draw_ground_patch(atlas, frame, pos, center, ts, tint);
    } else {
        let flip_x = (pos.x + pos.y).rem_euclid(2) == 0;
        atlas.draw_frame(frame, center, vec2(ts * scale, ts * scale), flip_x, tint);
    }
}

/// Sample a small, world-positioned patch from a ground cell. Sampling the
/// full 512px panel for every logical tile made the panel's own edges visible
/// as a grid; walking through the panel keeps adjacent floor tiles coherent.
fn draw_ground_patch(
    atlas: &SpriteAtlas,
    frame: usize,
    pos: TilePos,
    center: Vec2,
    ts: f32,
    tint: Color,
) {
    const PATCH: f32 = 128.0;
    let source = atlas.source_rect(frame);
    let patch_x = pos.x.rem_euclid(4) as f32 * PATCH;
    let patch_y = pos.y.rem_euclid(4) as f32 * PATCH;
    draw_texture_ex(
        &atlas.texture,
        center.x - ts * 0.5,
        center.y - ts * 0.5,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(ts, ts)),
            source: Some(Rect::new(
                source.x + patch_x,
                source.y + patch_y,
                PATCH,
                PATCH,
            )),
            ..Default::default()
        },
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

    draw_role_prop(sprites, creature, x, y + bob, radius, ts);
    if creature.satiation <= 0.33 {
        draw_circle_lines(x, y, radius + 2.0, 2.0, dark::NEGATIVE);
    } else if creature.satiation <= 0.66 {
        draw_circle_lines(x, y, radius + 2.0, 2.0, dark::WARNING);
    }
    if creature.equipment.is_some() {
        draw_gear_glint(x, y + bob, radius, ts);
    }
    if let Some((good, _)) = creature.carrying {
        draw_cargo(sprites, x, y + bob, radius, good, ts);
    }
}

/// A role's most oversized tool rides beside its bearer, letting a dense
/// colony read like a moving production diagram rather than a field of dots.
fn draw_role_prop(
    sprites: &WorldSprites,
    creature: &Creature,
    x: f32,
    y: f32,
    radius: f32,
    ts: f32,
) {
    if matches!(creature.species.as_str(), "beetle" | "salamander") {
        return;
    }
    let frame = match creature.job {
        Job::Miner => 0,
        Job::Carrier => 1,
        Job::Cook => 2,
        Job::Smith | Job::Smelter | Job::Engineer => 3,
        Job::Guard => 4,
        Job::Janitor => 2,
        Job::Courier => 1,
        Job::Idle => 5,
    };
    sprites.role_props.draw_frame(
        frame,
        vec2(x - radius * 0.50, y + radius * 0.08),
        vec2(ts * 0.84, ts * 0.84),
        false,
        WHITE,
    );
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

fn draw_cargo(sprites: &WorldSprites, x: f32, y: f32, radius: f32, good: Good, ts: f32) {
    let (cx, cy) = (x + radius * 0.56, y - radius * 0.48);
    let frame = match good {
        Good::Mushroom => 0,
        Good::Ore => 1,
        Good::Ingot => 2,
        Good::Wood => 3,
        Good::Charcoal => 4,
        Good::RawFood | Good::CookedFood => 0,
    };
    sprites.cargo.draw_frame(
        frame,
        vec2(cx, cy),
        vec2(ts * 0.54, ts * 0.54),
        false,
        WHITE,
    );
}
