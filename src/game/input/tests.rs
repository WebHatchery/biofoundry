use super::*;

#[test]
fn claimed_mouse_drag_blocks_the_release_action() {
    assert!(camera_claim_after_mouse_release(false, true));
    assert!(camera_claim_after_mouse_release(true, false));
    assert!(!camera_claim_after_mouse_release(false, false));
}

#[test]
fn map_tile_lookup_accepts_world_coordinates() {
    let data = GameData::load().expect("embedded game data");
    let session = GameSession::new(&data, 42);
    let spawn = session.spawn_tile();
    let tile_size = data.config.tile_size;
    let world = vec2(
        (spawn.x as f32 + 0.5) * tile_size,
        (spawn.y as f32 + 0.5) * tile_size,
    );

    assert_eq!(tile_at_world(&session, &data, world), Some(spawn));
    assert_eq!(tile_at_world(&session, &data, vec2(-1.0, -1.0)), None);
}
