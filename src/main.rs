//! Biofoundry executable shell.

use biofoundry::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    biofoundry::run().await;
}
