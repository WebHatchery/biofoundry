//! Top-level game: owns the state machine, camera, tool mode, and
//! fixed-timestep accumulator, and dispatches `UiAction` intents.

use crate::audio::{Audio, Sfx};
use crate::data::GameData;
use crate::simulation::{self, MAX_TICKS_PER_FRAME, SIM_DT};
use crate::state::creatures::Job;
use crate::state::outposts::{TransitCompletion, TransitDirection};
use crate::state::{GameSession, GameState, StateTransition};
use crate::tutorial::{self, TutorialInputs};
use crate::ui::{self, HudPanel, UiAction, UiMode};
use macroquad::prelude::*;
use macroquad_toolkit::camera::Camera2D;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::input::TouchGesture;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{slot_backup_exists, slot_exists};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState};
mod messages;
pub use messages::{
    advanced_building_hidden, auto_load_notice, auto_resupply_notice, auto_return_notice,
    clear_replacement_confirmations, emit_touch_target_audit_report, progression_reaches_safe_beat,
    report_touch_target_audit, simulation_blocked_by_modal, tile_world_center,
    transit_completion_notice, transit_failure_notice, unlock_notice, warren_secured_notice,
};

mod capture_scenes;
#[path = "game_actions.rs"]
pub mod game_actions;
pub mod input;
pub mod notifications;
pub mod persistence;
mod render;

pub use notifications::format_expedition_completion;
pub use persistence::{
    migrate_tutorial_progress, missing_save_notice, no_saved_slot_available,
    non_viable_save_notice, save_failure_banner, save_failure_notice, save_recovery_failure_banner,
    save_slot_available, should_restore_missing_primary,
};

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
    /// The title menu is confirming replacement of an existing save.
    confirm_new_warren: bool,
    /// The active Warren is confirming replacement with its last checkpoint.
    confirm_load: bool,
    /// The warren field guide is showing.
    help_open: bool,
    /// The recent event history is showing inside the field guide shell.
    event_log_open: bool,
    /// The single compact management drawer open above the world map.
    hud_panel: Option<HudPanel>,
    /// Zero is the newest page; larger values reveal older event history.
    event_log_page: usize,
    /// Whether the fixed-timestep simulation is paused by the player.
    paused: bool,
    /// A save slot exists, so the menu can offer Continue.
    save_exists: bool,
    /// Short-lived shell state that keeps a rejected save visible until the
    /// player successfully checkpoints again or loads a safe session.
    checkpoint_warning: Option<&'static str>,
    /// Where the right button went down, to tell a click from a camera drag.
    right_press: Vec2,
    /// Where the primary pointer went down for direct map panning.
    mouse_pan_start: Option<Vec2>,
    /// Whether the primary-pointer gesture crossed the drag threshold.
    mouse_camera_claimed: bool,
    /// Whether this frame's primary-pointer gesture claimed the map.
    camera_input_claimed: bool,
    /// Touch recognizer for one-finger map drags and two-finger pinch zoom.
    touch_gesture: TouchGesture,
    /// Keeps a claimed touch drag from becoming a synthetic mouse tap.
    touch_camera_claimed: bool,
    /// A touch tap that was not consumed by a HUD control this frame.
    touch_tap: Option<Vec2>,
    /// Building tile the player clicked to inspect (first-pass legibility).
    selected_building: Option<TilePos>,
    /// Whether the post-awakening route ledger is showing.
    routes_open: bool,
    /// Remaining frames for an opt-in capture-time touch-target audit.
    touch_audit_frames: u8,
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

        let camera = Camera2D::with_config(vec2(0.0, 0.0), 1.0, input::camera_config(&data, 1.0));
        let mut audio = Audio::load().await;
        audio.load_settings(&data.config.game_name);
        let save_exists = persistence::save_slot_available(
            slot_exists(&data.config.game_name, &data.config.save_slot),
            slot_backup_exists(&data.config.game_name, &data.config.save_slot),
        );

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
            confirm_new_warren: false,
            confirm_load: false,
            help_open: false,
            event_log_open: false,
            hud_panel: None,
            event_log_page: 0,
            paused: false,
            save_exists,
            checkpoint_warning: None,
            right_press: vec2(0.0, 0.0),
            mouse_pan_start: None,
            mouse_camera_claimed: false,
            camera_input_claimed: false,
            touch_gesture: TouchGesture::new(),
            touch_camera_claimed: false,
            touch_tap: None,
            selected_building: None,
            routes_open: false,
            touch_audit_frames: 0,
            world_sprites: ui::warren::WorldSprites::load(),
            menu_sprites: ui::menu::MenuSprites::load(),
            hud_sprites: ui::hud::HudSprites::load(),
        }
    }

    /// Seed a named scene for the headless screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        capture_scenes::begin(self, scene);
    }

    /// Arm the shared touch-target audit for a capture scene. Two frames are
    /// needed because safe hit-area growth uses the previous frame's control
    /// neighbours; reporting the first frame would measure a state the player
    /// never actually receives after the UI has settled.
    pub fn arm_touch_target_audit(&mut self) {
        self.touch_audit_frames = 2;
        macroquad_toolkit::ui::begin_target_audit();
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
            if !self.paused
                && !simulation_blocked_by_modal(
                    session,
                    &self.data,
                    self.help_open,
                    self.routes_open,
                    self.confirm_load,
                )
            {
                self.accumulator += dt;
                let mut ticks = 0;
                while self.accumulator >= SIM_DT && ticks < MAX_TICKS_PER_FRAME {
                    let report = simulation::tick(session, &self.data);
                    safe_beat_reached |= progression_reaches_safe_beat(&report);
                    for deserter in &report.deserters {
                        self.notifications.danger(format!(
                            "A starving {} deserted the warren!",
                            deserter.job.label().to_lowercase()
                        ));
                        self.audio.play(Sfx::Deny);
                    }
                    if report.won_this_tick {
                        safe_beat_reached = true;
                        self.notifications.success(warren_secured_notice(session));
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
                    for completion in &report.expedition_completed {
                        safe_beat_reached = true;
                        self.notifications
                            .info(format_expedition_completion(*completion));
                        self.audio.play(Sfx::Complete);
                    }
                    safe_beat_reached |= notifications::announce_outpost_milestones(
                        &report,
                        &self.data,
                        &mut self.notifications,
                        &mut self.audio,
                    );
                    if report.auto_return_started.is_some() {
                        safe_beat_reached = true;
                        self.notifications.info(auto_return_notice());
                        self.audio.play(Sfx::Select);
                    }
                    if report.auto_resupply_started.is_some() {
                        safe_beat_reached = true;
                        self.notifications.info(auto_resupply_notice());
                        self.audio.play(Sfx::Select);
                    }
                    if report.auto_load_started.is_some() {
                        safe_beat_reached = true;
                        self.notifications.info(auto_load_notice());
                        self.audio.play(Sfx::Select);
                    }
                    if report.transit_failed.is_some() {
                        safe_beat_reached = true;
                        self.notifications.danger(transit_failure_notice());
                        self.audio.play(Sfx::Deny);
                    }
                    if let Some(completion) = report.transit_completed {
                        safe_beat_reached = true;
                        self.notifications
                            .success(transit_completion_notice(completion));
                        self.audio.play(Sfx::Complete);
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
                        self.notifications
                            .success(unlock_notice(&self.data, session, name));
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
                self.notifications.warning(notifications::famine_notice());
                self.audio.play(Sfx::Alarm);
            } else if session.economy.food > 5.0 {
                self.famine_announced = false;
            }

            // Tutorial: advance any steps the player just satisfied. The
            let camera_moved = (self.camera.target - self.last_camera.0).length() > 4.0
                || (self.camera.zoom - self.last_camera.1).abs() > 0.01;
            if tutorial::advance(session, &self.data, TutorialInputs { camera_moved }) {
                safe_beat_reached = true;
                self.audio.play(Sfx::Select);
            }
            self.last_camera = (self.camera.target, self.camera.zoom);

            // Keyboard shortcuts remain secondary to the visible controls.
            if is_key_pressed(KeyCode::F5) {
                self.events.push(UiAction::Save);
            }
            if is_key_pressed(KeyCode::F9) {
                self.events.push(UiAction::RequestLoad);
            }
            if input.escape_pressed {
                // Escape backs out of a tool first, then to the menu.
                if self.help_open {
                    self.events.push(UiAction::ToggleHelp);
                } else if self.confirm_load {
                    self.events.push(UiAction::CancelLoad);
                } else if self.mode != UiMode::Inspect {
                    self.mode = self.mode.clone().after_successful_placement();
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
        } else if input.escape_pressed {
            if self.settings_open {
                self.settings_open = false;
            } else if self.confirm_new_warren {
                self.confirm_new_warren = false;
            }
        }

        if safe_beat_reached {
            self.autosave_game();
        }

        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }

    /// World tile under the mouse cursor, if inside the map.
    pub fn hover_tile(&self, session: &GameSession) -> Option<TilePos> {
        input::tile_at_world(
            session,
            &self.data,
            self.camera.screen_to_world(mouse_position().into()),
        )
    }

    pub fn world_click(&mut self, tile: TilePos) {
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
            self.hud_panel = None;
            return;
        }
        let GameState::Warren(session) = &mut self.state else {
            return;
        };
        let mut map_changed = false;
        match mode {
            UiMode::Build(kind) => {
                if simulation::try_place_build_site(session, &self.data, &kind, tile) {
                    map_changed = true;
                    // Building is a one-shot map action. Return to Inspect so
                    // the next map tap can select a building instead of
                    // silently placing another copy of the same site.
                    self.mode = UiMode::Inspect;
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
                    map_changed = true;
                    self.audio.play(Sfx::Select);
                }
            }
            UiMode::Inspect => {}
        }
        if map_changed {
            self.autosave_game();
        }
    }

    pub fn reassign(&mut self, from: Job, to: Job) {
        let changed = if let GameState::Warren(session) = &mut self.state {
            let species = &self.data.species;
            session.reassign(from, to, |s| {
                species.get(s).map(|d| d.reassignable).unwrap_or(false)
            })
        } else {
            false
        };
        if changed {
            self.audio.play(Sfx::Select);
            self.autosave_game();
        }
    }

    pub fn transition(&mut self, transition: StateTransition) {
        match transition {
            StateTransition::StartWarren => {
                let session = GameSession::new(&self.data, self.data.config.world_seed);
                self.notifications.clear();
                self.notifications.clear_history();
                self.reset_camera_for(&session);
                self.reset_session_view_state();
                self.state = GameState::Warren(Box::new(session));
                self.autosave_game();
            }
            StateTransition::BackToMenu => {
                self.mode = UiMode::Inspect;
                self.help_open = false;
                self.event_log_open = false;
                self.event_log_page = 0;
                self.routes_open = false;
                self.hud_panel = None;
                self.paused = false;
                self.confirm_new_warren = false;
                self.confirm_load = false;
                self.state = GameState::Menu;
            }
        }
    }

    pub fn reset_camera_for(&mut self, session: &GameSession) {
        let tile = self.data.config.tile_size;
        let (sx, sy) = session.world.spawn.to_f32();
        let center = vec2((sx + 0.5) * tile, (sy + 0.5) * tile);
        self.camera = Camera2D::with_config(center, 1.0, input::camera_config(&self.data, tile));
        // Don't let the reset itself count as "the player looked around".
        self.last_camera = (self.camera.target, self.camera.zoom);
    }

    /// Center the map on a building selected through a HUD shortcut while
    /// preserving the camera's configured world bounds.
    pub fn focus_camera_on_tile(&mut self, tile: TilePos) {
        if let Some(center) = tile_world_center(tile, self.data.config.tile_size) {
            self.camera.pan(center - self.camera.target);
        }
    }
}
