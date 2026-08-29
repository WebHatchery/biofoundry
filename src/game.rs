//! Top-level game: owns the state machine, camera, tool mode, and
//! fixed-timestep accumulator, and dispatches `UiAction` intents.

use crate::audio::{Audio, Sfx};
use crate::data::GameData;
use crate::simulation::{self, MAX_TICKS_PER_FRAME, SIM_DT};
use crate::state::creatures::Job;
use crate::state::{GameSession, GameState, StateTransition};
use crate::tutorial::{self, TutorialInputs};
use crate::ui::{self, UiAction, UiMode};
use macroquad::prelude::*;
use macroquad_toolkit::camera::{Camera2D, Camera2DConfig, CameraBounds};
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::input::TouchGesture;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, quarantine_slot, restore_slot_backup,
    save_to_slot_with_version_and_backup, slot_backup_exists, slot_exists,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState};

const CAMERA_DRAG_THRESHOLD: f32 = 6.0;

mod capture_scenes;
#[path = "game_actions.rs"]
mod game_actions;

pub struct Game {
    data: GameData,
    state: GameState,
    camera: Camera2D,
    mode: UiMode,
    events: EventBus<UiAction>,
    notifications: NotificationManager,
    audio: Audio,
    /// Real time not yet consumed by fixed-step sim ticks.
    accumulator: f32,
    /// Edge detector for the famine warning toast.
    famine_announced: bool,
    /// Last frame's camera pose, for the tutorial's "look around" step.
    last_camera: (Vec2, f32),
    /// The title menu's settings panel is showing.
    settings_open: bool,
    /// The warren field guide is showing.
    help_open: bool,
    /// Whether the fixed-timestep simulation is paused by the player.
    paused: bool,
    /// A save slot exists, so the menu can offer Continue.
    save_exists: bool,
    /// Where the right button went down, to tell a click from a camera drag.
    right_press: Vec2,
    /// Where the primary pointer went down for direct map panning.
    mouse_pan_start: Option<Vec2>,
    /// Whether this frame's primary-pointer gesture claimed the map.
    camera_input_claimed: bool,
    /// Touch recognizer for one-finger map drags and two-finger pinch zoom.
    touch_gesture: TouchGesture,
    /// Keeps a claimed touch drag from becoming a synthetic mouse tap.
    touch_camera_claimed: bool,
    /// Building tile the player clicked to inspect (first-pass legibility).
    selected_building: Option<TilePos>,
    /// Embedded storybook creature atlas used by the warren renderer.
    world_sprites: ui::warren::WorldSprites,
    /// Hand-painted cavern tableau used by the title menu.
    menu_sprites: ui::menu::MenuSprites,
    /// Illustrated job markers used by the text-forward HUD.
    hud_sprites: ui::hud::HudSprites,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().unwrap_or_else(|err| {
            panic!("Biofoundry embedded data failed to load: {}", err);
        });

        let camera = Camera2D::with_config(vec2(0.0, 0.0), 1.0, camera_config(&data, 1.0));
        let mut audio = Audio::load().await;
        audio.load_settings(&data.config.game_name);
        let save_exists = slot_exists(&data.config.game_name, &data.config.save_slot);

        Self {
            data,
            state: GameState::Menu,
            camera,
            mode: UiMode::Inspect,
            events: EventBus::new(),
            notifications: NotificationManager::new(),
            audio,
            accumulator: 0.0,
            famine_announced: false,
            last_camera: (vec2(0.0, 0.0), 1.0),
            settings_open: false,
            help_open: false,
            paused: false,
            save_exists,
            right_press: vec2(0.0, 0.0),
            mouse_pan_start: None,
            camera_input_claimed: false,
            touch_gesture: TouchGesture::new(),
            touch_camera_claimed: false,
            selected_building: None,
            world_sprites: ui::warren::WorldSprites::load(),
            menu_sprites: ui::menu::MenuSprites::load(),
            hud_sprites: ui::hud::HudSprites::load(),
        }
    }

    /// Seed a named scene for the headless screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        capture_scenes::begin(self, scene);
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.update(dt);
        let input = InputState::capture();

        // The camera owns the primary pointer and touch gestures before the
        // HUD turns a release into a world click. A claimed drag is carried
        // through the release frame so it cannot also select a tile.
        if matches!(&self.state, GameState::Warren(_)) {
            self.update_camera_input(dt);
        }

        let mut safe_beat_reached = false;
        if let GameState::Warren(session) = &mut self.state {
            if !self.paused {
                self.accumulator += dt;
                let mut ticks = 0;
                while self.accumulator >= SIM_DT && ticks < MAX_TICKS_PER_FRAME {
                    let report = simulation::tick(session, &self.data);
                    for deserter in &report.deserters {
                        self.notifications.danger(format!(
                            "A starving {} deserted the warren!",
                            deserter.job.label().to_lowercase()
                        ));
                        self.audio.play(Sfx::Deny);
                    }
                    if report.won_this_tick {
                        safe_beat_reached = true;
                        self.notifications.success("The warren thrives — victory!");
                        self.audio.play(Sfx::Complete);
                    }
                    if report.factory_this_tick {
                        safe_beat_reached = true;
                        self.notifications
                            .success("The Biofoundry roars — factory complete!");
                        self.audio.play(Sfx::Complete);
                    }
                    if report.worm_this_tick {
                        safe_beat_reached = true;
                        self.notifications
                            .success("The ground heaves — the Colossal Worm awakens!");
                        self.audio.play(Sfx::Worm);
                    }
                    if report.wild.raid_started {
                        self.notifications
                            .danger("Raid! Gnarls are coming for the larder.");
                        self.audio.play(Sfx::Alarm);
                    }
                    if report.wild.raid_survived {
                        self.notifications.success("The raid is over — we held.");
                        self.audio.play(Sfx::Complete);
                    }
                    for _ in 0..report.wild.captured {
                        self.notifications
                            .success("A wild beetle was snared — specimen housed.");
                        self.audio.play(Sfx::Capture);
                    }
                    for _ in 0..report.wild.guards_killed {
                        self.notifications
                            .danger("A guard fell defending the warren.");
                        self.audio.play(Sfx::Deny);
                    }
                    for name in &report.wild.unlocked {
                        self.notifications.success(format!("Unlocked: {name}"));
                        self.audio.play(Sfx::Complete);
                    }
                    if report.wild.bred_beetle {
                        self.notifications
                            .info("The breeding pit hatched a new beetle hauler.");
                        self.audio.play(Sfx::Capture);
                    }
                    self.accumulator -= SIM_DT;
                    ticks += 1;
                }
                // Drop backlog beyond the cap instead of spiraling.
                if self.accumulator >= SIM_DT {
                    self.accumulator = 0.0;
                }
            }

            if session.economy.food <= 0.0 && !self.famine_announced {
                self.famine_announced = true;
                self.notifications
                    .warning("Famine! The stockpile is empty — workers are slowing.");
                self.audio.play(Sfx::Alarm);
            } else if session.economy.food > 5.0 {
                self.famine_announced = false;
            }

            // Tutorial: advance any steps the player just satisfied. The
            let camera_moved = (self.camera.target - self.last_camera.0).length() > 4.0
                || (self.camera.zoom - self.last_camera.1).abs() > 0.01;
            if tutorial::advance(session, &self.data, TutorialInputs { camera_moved }) {
                self.audio.play(Sfx::Select);
            }
            self.last_camera = (self.camera.target, self.camera.zoom);

            // Keyboard shortcuts remain secondary to the visible controls.
            if is_key_pressed(KeyCode::F5) {
                self.events.push(UiAction::Save);
            }
            if is_key_pressed(KeyCode::F9) {
                self.events.push(UiAction::Load);
            }
            if input.escape_pressed {
                // Escape backs out of a tool first, then to the menu.
                if self.help_open {
                    self.events.push(UiAction::ToggleHelp);
                } else if self.mode != UiMode::Inspect {
                    self.mode = UiMode::Inspect;
                } else {
                    self.events.push(UiAction::BackToMenu);
                }
            }
            // A right-click (not a camera drag) also cancels the tool.
            if is_mouse_button_pressed(MouseButton::Right) {
                self.right_press = mouse_position().into();
            }
            if is_mouse_button_released(MouseButton::Right)
                && self.mode != UiMode::Inspect
                && Vec2::from(mouse_position()).distance(self.right_press) < 8.0
            {
                self.mode = UiMode::Inspect;
            }
        } else if input.escape_pressed && self.settings_open {
            self.settings_open = false;
        }

        if safe_beat_reached {
            self.autosave_game();
        }

        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);

        let actions = match &self.state {
            GameState::Menu => {
                let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
                let actions = ui::menu::draw(
                    &self.data,
                    &virtual_ui,
                    &self.menu_sprites,
                    self.save_exists,
                    self.settings_open,
                    self.audio.volume(),
                );
                end_virtual_ui_frame();
                actions
            }
            GameState::Warren(session) => {
                let hover = self.hover_tile(session);

                self.camera.begin();
                ui::warren::draw_world(
                    session,
                    &self.data,
                    &self.world_sprites,
                    self.data.config.tile_size,
                    &self.mode,
                    hover,
                );
                set_default_camera();

                let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
                let frame = ui::hud::draw(
                    session,
                    &self.data,
                    &virtual_ui,
                    &self.hud_sprites,
                    &self.mode,
                    self.selected_building,
                    ui::hud::HudOptions {
                        help_open: self.help_open,
                        paused: self.paused,
                        save_exists: self.save_exists,
                    },
                );
                end_virtual_ui_frame();

                let mut actions = frame.actions;
                // A claimed drag must not activate a HUD button when the
                // browser delivers the touch/mouse release over that button.
                if self.camera_input_claimed {
                    actions.clear();
                }
                // Left-click routes to the world in every mode: tools act,
                // Inspect selects the building under the cursor.
                if !frame.pointer_over_ui
                    && !self.camera_input_claimed
                    && is_mouse_button_released(MouseButton::Left)
                {
                    if let Some(tile) = hover {
                        actions.push(UiAction::WorldClick(tile));
                    }
                }
                actions
            }
        };

        for action in actions {
            self.events.push(action);
        }

        self.notifications
            .draw_with_config(&NotificationRenderConfig {
                anchor: NotificationAnchor::BottomRight,
                ..Default::default()
            });
    }

    /// World tile under the mouse cursor, if inside the map.
    fn hover_tile(&self, session: &GameSession) -> Option<TilePos> {
        let world = self.camera.screen_to_world(mouse_position().into());
        let ts = self.data.config.tile_size;
        let tile = TilePos::new((world.x / ts).floor() as i32, (world.y / ts).floor() as i32);
        session.world.tiles.is_valid(tile).then_some(tile)
    }

    fn world_click(&mut self, tile: TilePos) {
        let mode = self.mode.clone();
        if mode == UiMode::Inspect {
            // Toggle-select the building under the cursor for inspection.
            let on_building =
                matches!(&self.state, GameState::Warren(s) if s.building_at(tile).is_some());
            self.selected_building = if on_building && self.selected_building != Some(tile) {
                Some(tile)
            } else {
                None
            };
            return;
        }
        let GameState::Warren(session) = &mut self.state else {
            return;
        };
        match mode {
            UiMode::Build(kind) => {
                if simulation::try_place_build_site(session, &self.data, &kind, tile) {
                    let cost = self
                        .data
                        .buildings
                        .get(&kind)
                        .map(|d| d.cost_ore)
                        .unwrap_or(0);
                    self.notifications
                        .info(format!("Site placed — carriers will deliver {cost} ore."));
                    self.audio.play(Sfx::Build);
                } else {
                    self.notifications.warning("Can't build there.");
                    self.audio.play(Sfx::Deny);
                }
            }
            UiMode::Dig => {
                if session.toggle_dig_mark(tile) {
                    self.audio.play(Sfx::Select);
                }
            }
            UiMode::Inspect => {}
        }
    }

    fn save_game(&mut self) {
        if matches!(&self.state, GameState::Warren(session) if session.creatures.is_empty()) {
            self.notifications.warning(
                "This warren has fallen silent. Load a safe save or start a new warren instead.",
            );
            return;
        }
        match self.persist_current_session() {
            Ok(()) => {
                self.save_exists = true;
                self.notifications.success("Warren saved.");
            }
            Err(err) => self.notifications.danger(format!("Save failed: {err}")),
        }
    }

    /// Persist a campaign milestone without interrupting the player's flow.
    fn autosave_game(&mut self) {
        match self.persist_current_session() {
            Ok(()) => {
                self.save_exists = true;
            }
            Err(err) => self
                .notifications
                .warning(format!("Autosave failed — use Save manually: {err}")),
        }
    }

    fn persist_current_session(&self) -> Result<(), String> {
        let GameState::Warren(session) = &self.state else {
            return Err("no active Warren".to_owned());
        };
        let config = &self.data.config;
        save_to_slot_with_version_and_backup(
            &config.game_name,
            &config.save_slot,
            session.as_ref(),
            &config.version,
        )
    }

    fn load_game(&mut self) {
        let slot = self.data.config.save_slot.clone();
        match self.load_session_from_slot(&slot) {
            Ok(session) => {
                self.install_loaded_session(session);
                self.notifications.success("Warren loaded.");
            }
            Err(err) => self.recover_failed_load(&slot, err),
        }
    }

    fn load_session_from_slot(&self, slot: &str) -> Result<GameSession, String> {
        let config = &self.data.config;
        load_from_slot_with_migration(
            &config.game_name,
            slot,
            &config.version,
            |version, value| {
                let payload = value.get("data").cloned().unwrap_or(value);
                serde_json::from_value(payload)
                    .map_err(|err| format!("Unsupported save {version:?}: {err}"))
            },
        )
    }

    fn install_loaded_session(&mut self, session: GameSession) {
        self.reset_camera_for(&session);
        self.accumulator = 0.0;
        self.mode = UiMode::Inspect;
        self.state = GameState::Warren(Box::new(session));
    }

    fn recover_failed_load(&mut self, slot: &str, error: String) {
        let config = &self.data.config;
        if !slot_exists(&config.game_name, slot) {
            self.notifications.warning(format!("Load failed: {error}"));
            return;
        }

        let quarantine = match quarantine_slot(&config.game_name, slot) {
            Ok(name) => name,
            Err(quarantine_error) => {
                self.notifications.danger(format!(
                    "Save could not load ({error}). The original was left untouched; repair it or use Menu → New Warren. ({quarantine_error})"
                ));
                return;
            }
        };

        if !slot_backup_exists(&config.game_name, slot) {
            self.save_exists = false;
            self.notifications.danger(format!(
                "Save is damaged; preserved it as {quarantine}. Use Menu → New Warren or repair it before loading again."
            ));
            return;
        }

        let backup_slot = format!("{slot}_backup");
        match self.load_session_from_slot(&backup_slot) {
            Ok(session) => match restore_slot_backup(&config.game_name, slot) {
                Ok(_) => {
                    self.install_loaded_session(session);
                    self.save_exists = true;
                    self.notifications.warning(format!(
                        "Primary save was damaged; preserved it as {quarantine} and restored the previous safe save."
                    ));
                }
                Err(restore_error) => {
                    self.install_loaded_session(session);
                    self.save_exists = false;
                    self.notifications.warning(format!(
                        "Primary save was damaged; loaded the safe backup, but could not restore it ({restore_error}). Use Save now."
                    ));
                }
            },
            Err(backup_error) => {
                self.save_exists = false;
                self.notifications.danger(format!(
                    "Save is damaged; preserved it as {quarantine}. The safe backup also failed ({backup_error}). Use Menu → New Warren or repair the preserved saves."
                ));
            }
        }
    }

    fn reassign(&mut self, from: Job, to: Job) {
        let GameState::Warren(session) = &mut self.state else {
            return;
        };
        let species = &self.data.species;
        if session.reassign(from, to, |s| {
            species.get(s).map(|d| d.reassignable).unwrap_or(false)
        }) {
            self.audio.play(Sfx::Select);
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        match transition {
            StateTransition::StartWarren => {
                let session = GameSession::new(&self.data, self.data.config.world_seed);
                self.reset_camera_for(&session);
                self.accumulator = 0.0;
                self.famine_announced = false;
                self.mode = UiMode::Inspect;
                self.help_open = false;
                self.paused = false;
                self.state = GameState::Warren(Box::new(session));
                self.autosave_game();
            }
            StateTransition::BackToMenu => {
                self.mode = UiMode::Inspect;
                self.help_open = false;
                self.paused = false;
                self.state = GameState::Menu;
            }
        }
    }

    fn reset_camera_for(&mut self, session: &GameSession) {
        let tile = self.data.config.tile_size;
        let (sx, sy) = session.world.spawn.to_f32();
        let center = vec2((sx + 0.5) * tile, (sy + 0.5) * tile);
        self.camera = Camera2D::with_config(center, 1.0, camera_config(&self.data, tile));
        // Don't let the reset itself count as "the player looked around".
        self.last_camera = (self.camera.target, self.camera.zoom);
    }

    /// Apply touch gestures and primary-pointer dragging to the warren camera.
    ///
    /// Touches are handled explicitly because browsers may synthesize a left
    /// mouse click for the first finger. A drag must pan the map while a short
    /// contact must remain available to the HUD and world tools as a tap.
    fn update_camera_input(&mut self, dt: f32) {
        self.camera_input_claimed = false;
        let touch = self.touch_gesture.update();
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
        } else {
            let mouse: Vec2 = mouse_position().into();
            if is_mouse_button_pressed(MouseButton::Left) {
                self.mouse_pan_start = Some(mouse);
                self.camera_input_claimed = false;
            }
            if let Some(start) = self.mouse_pan_start {
                if is_mouse_button_down(MouseButton::Left)
                    && mouse.distance(start) > CAMERA_DRAG_THRESHOLD
                {
                    self.camera_input_claimed = true;
                    self.camera.pan(-(mouse - start) / self.camera.zoom);
                    // Continue from the current pointer position so the
                    // camera follows the drag without accumulating rounding.
                    self.mouse_pan_start = Some(mouse);
                }
                if is_mouse_button_released(MouseButton::Left) {
                    self.mouse_pan_start = None;
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

fn camera_config(data: &GameData, tile_size: f32) -> Camera2DConfig {
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
