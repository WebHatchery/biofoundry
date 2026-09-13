//! Public library surface for Biofoundry's deterministic game logic.
//!
//! The executable is a thin Macroquad runtime shell. Keeping the game modules
//! here gives integration tests a stable seam while the implementation keeps
//! ownership of its internal helpers and rendering details.

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod audio;
pub mod data;
pub mod game;
pub mod simulation;
pub mod state;
pub mod tutorial;
pub mod ui;

/// Window configuration shared by the desktop and capture runtime.
pub fn window_conf() -> Conf {
    capture::capture_window_conf(
        "BIOFOUNDRY",
        "Biofoundry",
        ui::LOGICAL_WIDTH as i32,
        ui::LOGICAL_HEIGHT as i32,
    )
}

/// Run the game loop and the optional screenshot/capture harness.
pub async fn run() {
    let mut game = game::Game::new().await;

    // Screenshot harness: when BIOFOUNDRY_CAPTURE_PATH is set, seed the named
    // scene ("menu", "warren", or a named verification scene), render
    // deterministic frames, write a PNG, and exit. Stubbed out on wasm32.
    if let Some(configs) = capture::CaptureConfig::all_from_env("BIOFOUNDRY") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |dt| {
                game.update(dt);
                game.draw();
            })
            .await;
        }
        return;
    }

    loop {
        let dt = get_frame_time().min(0.1);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
