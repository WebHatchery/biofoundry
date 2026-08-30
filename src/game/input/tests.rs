use super::*;

#[test]
fn claimed_mouse_drag_blocks_the_release_action() {
    assert!(camera_claim_after_mouse_release(false, true));
    assert!(camera_claim_after_mouse_release(true, false));
    assert!(!camera_claim_after_mouse_release(false, false));
}

#[test]
fn touch_taps_use_the_same_map_coordinate_conversion_as_mouse_hover() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let spawn = session.spawn_tile();
    let tile_size = data.config.tile_size;
    let screen = vec2(
        (spawn.x as f32 + 0.5) * tile_size,
        (spawn.y as f32 + 0.5) * tile_size,
    );

    assert_eq!(tile_at_screen(&session, &data, screen), Some(spawn));
    assert_eq!(tile_at_screen(&session, &data, vec2(-1.0, -1.0)), None);
}
