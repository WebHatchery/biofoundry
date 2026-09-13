//! Camera gestures and map-coordinate input for the game shell.

use super::Game;
use crate::data::GameData;
use crate::state::GameSession;
use macroquad::prelude::*;
use macroquad_toolkit::camera::{Camera2DConfig, CameraBounds};
use macroquad_toolkit::grid::TilePos;

const CAMERA_DRAG_THRESHOLD: f32 = 6.0;

impl Game {
    /// Apply touch gestures and primary-pointer dragging to the warren camera.
    ///
    /// Touches are handled explicitly because browsers may synthesize a left
    /// mouse click for the first finger. A drag must pan the map while a short
    /// contact must remain available to the HUD and world tools as a tap.
    pub(super) fn update_camera_input(&mut self, dt: f32) {
        self.camera_input_claimed = false;
        self.touch_tap = None;
        let touch = self.touch_gesture.update();
        self.touch_tap = touch.tap;
        let previous_touch_claim = self.touch_camera_claimed;
        if touch.active && touch.claimed {
            self.touch_camera_claimed = true;
        }
        let touch_claimed = touch.claimed || previous_touch_claim;

        if touch.pan.length_squared() > 0.0 {
            self.camera.pan(-touch.pan / self.camera.zoom);
        }
        if (touch.scale - 1.0).abs() > f32::EPSILON {
            self.camera.zoom_at(touch.scale, touch.center);
        }

        // Ignore synthetic mouse events while a finger is on the canvas or
        // while a claimed touch is being released.
        if touch.active || self.touch_camera_claimed {
            self.mouse_pan_start = None;
            self.mouse_camera_claimed = false;
        } else {
            let mouse: Vec2 = mouse_position().into();
            if is_mouse_button_pressed(MouseButton::Left) {
                self.mouse_pan_start = Some(mouse);
                self.mouse_camera_claimed = false;
                self.camera_input_claimed = false;
            }
            if let Some(start) = self.mouse_pan_start {
                if is_mouse_button_down(MouseButton::Left)
                    && mouse.distance(start) > CAMERA_DRAG_THRESHOLD
                {
                    self.mouse_camera_claimed = true;
                    self.camera_input_claimed = true;
                    self.camera.pan(-(mouse - start) / self.camera.zoom);
                    // Continue from the current pointer position so the
                    // camera follows the drag without accumulating rounding.
                    self.mouse_pan_start = Some(mouse);
                }
                if is_mouse_button_released(MouseButton::Left) {
                    self.camera_input_claimed = camera_claim_after_mouse_release(
                        self.camera_input_claimed,
                        self.mouse_camera_claimed,
                    );
                    self.mouse_pan_start = None;
                    self.mouse_camera_claimed = false;
                }
            }
        }

        if !touch.active {
            self.touch_camera_claimed = false;
        }
        self.camera_input_claimed |= touch_claimed;

        // Keep optional keyboard and wheel shortcuts working while direct
        // primary-pointer and touch gestures remain the required path.
        self.camera.update(dt, false);
    }
}

/// Resolve a world-space point to a valid map tile.
pub fn tile_at_world(session: &GameSession, data: &GameData, world: Vec2) -> Option<TilePos> {
    let tile_size = data.config.tile_size;
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return None;
    }
    let tile = TilePos::new(
        (world.x / tile_size).floor() as i32,
        (world.y / tile_size).floor() as i32,
    );
    session.world.tiles.is_valid(tile).then_some(tile)
}

pub fn camera_config(data: &GameData, tile_size: f32) -> Camera2DConfig {
    let world_w = data.config.world_width as f32 * tile_size;
    let world_h = data.config.world_height as f32 * tile_size;
    Camera2DConfig {
        // Game handles direct mouse and touch gestures so a primary-pointer
        // drag can pan without turning the release into a world click.
        drag_button: None,
        min_zoom: 0.5,
        max_zoom: 3.0,
        bounds: Some(CameraBounds::new(vec2(0.0, 0.0), vec2(world_w, world_h))),
        ..Default::default()
    }
}

pub fn camera_claim_after_mouse_release(frame_claimed: bool, gesture_claimed: bool) -> bool {
    frame_claimed || gesture_claimed
}
