//! Top-level game: owns the state machine, camera, tool mode, and
//! fixed-timestep accumulator, and dispatches `UiAction` intents.

use crate::audio::{Audio, Sfx};
use crate::data::GameData;
use crate::simulation::{self, MAX_TICKS_PER_FRAME, SIM_DT};
use crate::state::creatures::Job;
use crate::state::outposts::{TransitCompletion, TransitDirection};
use crate::state::{GameSession, GameState, StateTransition};
use crate::tutorial::{self, TutorialInputs};
use crate::ui::{self, UiAction, UiMode};
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

mod capture_scenes;
#[path = "game_actions.rs"]
mod game_actions;
mod input;
mod notifications;
mod persistence;
pub(super) use notifications::format_expedition_completion;
#[cfg(test)]
mod tests;

#[cfg(test)]
use persistence::{
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
    pub(super) fn arm_touch_target_audit(&mut self) {
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
                self.notifications
                    .warning("Famine! Stockpile empty — add food.");
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

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);

        if self.touch_audit_frames > 0 {
            macroquad_toolkit::ui::begin_target_frame();
        }

        // Preserve readable text when the fixed 1280x720 layout is letterboxed
        // into a smaller browser canvas. The toolkit keeps this bounded so the
        // established candidate scale remains unchanged at the design size.
        let ui_text_scale = macroquad_toolkit::ui::set_ui_text_scale_for_screen(
            ui::LOGICAL_WIDTH,
            ui::LOGICAL_HEIGHT,
            1.25,
        );

        let actions = match &self.state {
            GameState::Menu => {
                let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
                let actions = ui::menu::draw(
                    &self.data,
                    &virtual_ui,
                    &self.menu_sprites,
                    self.save_exists,
                    self.settings_open,
                    self.confirm_new_warren,
                    self.audio.volume(),
                );
                end_virtual_ui_frame();
                actions
            }
            GameState::Warren(session) => {
                let hover = self.hover_tile(session);
                let touch_tap = self.touch_tap.and_then(|screen| {
                    let world = self.camera.screen_to_world(screen);
                    input::tile_at_world(session, &self.data, world)
                });

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
                        event_log_open: self.event_log_open,
                        event_log_page: self.event_log_page,
                        event_history: self.notifications.history(),
                        routes_open: self.routes_open,
                        confirm_load: self.confirm_load,
                        paused: self.paused,
                        save_exists: self.save_exists,
                        checkpoint_warning: self.checkpoint_warning,
                        touch_position: self.touch_tap,
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
                if !frame.pointer_over_ui && !self.camera_input_claimed {
                    if let Some(tile) = touch_tap {
                        actions.push(UiAction::WorldClick(tile));
                    } else if is_mouse_button_released(MouseButton::Left) {
                        if let Some(tile) = hover {
                            actions.push(UiAction::WorldClick(tile));
                        }
                    }
                }
                actions
            }
        };

        // Roll the HUD's visible controls into the next frame's neighbor map
        // so the shared toolkit can grow their touch hit areas safely.
        macroquad_toolkit::ui::end_frame_neighbours();

        if self.touch_audit_frames > 0 {
            self.touch_audit_frames -= 1;
            if self.touch_audit_frames == 0 {
                report_touch_target_audit();
                macroquad_toolkit::ui::end_target_audit();
            }
        }

        for action in actions {
            self.events.push(action);
        }

        // The published game page reserves its lower-right corner for the
        // Report a Bug widget and the tall Outpost inspection card both use
        // the lower-right corner. Keep the toast stack anchored there while
        // lifting it clear of the page chrome and shifting it left of the
        // card's footprint.
        // Modal guides already provide a larger surface for the same
        // messages. Keep transient toasts behind that surface so they do not
        // cover its controls or duplicate the recent-events list.
        if !self.help_open && !self.routes_open {
            self.notifications.draw_with_config_and_offset(
                &NotificationRenderConfig {
                    anchor: NotificationAnchor::BottomRight,
                    // Notifications are already positioned in screen space.
                    // Counter the logical HUD's readability multiplier so a
                    // compact canvas does not enlarge the toast text past
                    // its fixed screen-space row width.
                    font_size: 16.0 / ui_text_scale,
                    ..Default::default()
                },
                vec2(-250.0, -82.0),
            );
        }
    }

    /// World tile under the mouse cursor, if inside the map.
    fn hover_tile(&self, session: &GameSession) -> Option<TilePos> {
        input::tile_at_world(
            session,
            &self.data,
            self.camera.screen_to_world(mouse_position().into()),
        )
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

    fn reassign(&mut self, from: Job, to: Job) {
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

    fn transition(&mut self, transition: StateTransition) {
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
                self.paused = false;
                self.confirm_new_warren = false;
                self.confirm_load = false;
                self.state = GameState::Menu;
            }
        }
    }

    fn reset_camera_for(&mut self, session: &GameSession) {
        let tile = self.data.config.tile_size;
        let (sx, sy) = session.world.spawn.to_f32();
        let center = vec2((sx + 0.5) * tile, (sy + 0.5) * tile);
        self.camera = Camera2D::with_config(center, 1.0, input::camera_config(&self.data, tile));
        // Don't let the reset itself count as "the player looked around".
        self.last_camera = (self.camera.target, self.camera.zoom);
    }

    /// Center the map on a building selected through a HUD shortcut while
    /// preserving the camera's configured world bounds.
    pub(super) fn focus_camera_on_tile(&mut self, tile: TilePos) {
        if let Some(center) = tile_world_center(tile, self.data.config.tile_size) {
            self.camera.pan(center - self.camera.target);
        }
    }
}

fn report_touch_target_audit() {
    let mut report = String::new();
    if !macroquad_toolkit::ui::neighbours_warm() {
        report.push_str("touch targets: no settled controls were recorded\n");
        emit_touch_target_audit_report(&report);
        return;
    }

    if let Some((width, worst)) = macroquad_toolkit::ui::smallest_touchable_width(ui::LOGICAL_WIDTH)
    {
        report.push_str(&format!(
            "touch targets: need a {:.0}px-wide window; worst is {}\n",
            width, worst
        ));
    }
    for (side, label) in macroquad_toolkit::ui::undersized_targets() {
        report.push_str(&format!("touch targets: drawn {:.0}px — {}\n", side, label));
    }
    for (a, b, area) in macroquad_toolkit::ui::overlapping_targets() {
        report.push_str(&format!(
            "touch targets: {} and {} overlap by {:.0}px² once grown\n",
            a, b, area
        ));
    }
    emit_touch_target_audit_report(&report);
}

fn emit_touch_target_audit_report(report: &str) {
    print!("{report}");
    if let Ok(path) = std::env::var("BIOFOUNDRY_TOUCH_AUDIT_REPORT") {
        if let Err(error) = std::fs::write(&path, report) {
            eprintln!("touch targets: could not write {path}: {error}");
        }
    }
}

fn tile_world_center(tile: TilePos, tile_size: f32) -> Option<Vec2> {
    if !tile_size.is_finite() || tile_size <= 0.0 {
        return None;
    }
    let (x, y) = tile.to_f32();
    Some(vec2((x + 0.5) * tile_size, (y + 0.5) * tile_size))
}

fn simulation_blocked_by_modal(
    session: &GameSession,
    data: &GameData,
    help_open: bool,
    routes_open: bool,
    confirm_load: bool,
) -> bool {
    help_open
        || routes_open
        || confirm_load
        || (session.won && !session.victory_shown)
        || (session.factory_complete && !session.factory_shown)
        || (session.worm_awake && !session.worm_shown)
        || (!session.worm_awake && session.is_non_viable(data))
}

fn clear_replacement_confirmations(confirm_new_warren: &mut bool, confirm_load: &mut bool) {
    *confirm_new_warren = false;
    *confirm_load = false;
}

fn progression_reaches_safe_beat(report: &simulation::TickReport) -> bool {
    report.wild.raid_survived
        || report.wild.captured > 0
        || !report.wild.unlocked.is_empty()
        || report.wild.bred_beetle
        || report.outpost_relay_awarded
        || report.outpost_convoy_awarded > 0
        || report.outpost_muster_awarded > 0
        || report.outpost_concord_awarded
        || report.outpost_circuit_awarded
        || report.outpost_encore_awarded > 0
        || report.outpost_chorus_awarded
        || report.auto_load_started.is_some()
}

fn transit_completion_notice(completion: TransitCompletion) -> &'static str {
    let payload = match (completion.cargo_units > 0, completion.passenger_count > 0) {
        (true, true) => "cargo and crew delivered",
        (true, false) => "cargo delivered",
        (false, true) => "crew delivered",
        (false, false) => "route complete",
    };
    match completion.direction {
        TransitDirection::ToOutpost => match payload {
            "cargo and crew delivered" => {
                "The worm reaches the outpost — cargo and crew delivered."
            }
            "cargo delivered" => "The worm reaches the outpost — cargo delivered.",
            "crew delivered" => "The worm reaches the outpost — crew delivered.",
            _ => "The worm reaches the outpost — route complete.",
        },
        TransitDirection::ToShrine => match payload {
            "cargo and crew delivered" => {
                "The worm returns to the shrine — cargo and crew delivered."
            }
            "cargo delivered" => "The worm returns to the shrine — cargo delivered.",
            "crew delivered" => "The worm returns to the shrine — crew delivered.",
            _ => "The worm returns to the shrine — route complete.",
        },
    }
}

fn auto_return_notice() -> &'static str {
    "Outpost hold full — cargo returning while scouts remain remote."
}

fn auto_resupply_notice() -> &'static str {
    "Outpost scouts need food — a food-only resupply is on its way."
}

fn auto_load_notice() -> &'static str {
    "Auto-load departed — cargo and available scouts are on the worm road."
}

fn transit_failure_notice() -> &'static str {
    "The worm route failed — tap the outpost, then reactivate the route before trying again."
}

fn warren_secured_notice(session: &GameSession) -> &'static str {
    if session.job_count(Job::Guard) > 0 {
        "The warren is secure — onboarding complete."
    } else {
        "The reserve gate is secure — assign a Guard to finish onboarding."
    }
}

fn unlock_notice(data: &GameData, session: &GameSession, name: &str) -> String {
    let Some(unlock) = data.unlocks.iter().find(|unlock| unlock.name == name) else {
        return format!("Unlocked: {name}");
    };

    let detail = match unlock.effect.as_str() {
        "unlock_building" => unlock
            .building
            .as_deref()
            .and_then(|id| data.buildings.get(id).map(|building| (id, building)))
            .map(|(id, building)| {
                if advanced_building_hidden(session, id) {
                    "available in Build & Dig after onboarding".to_owned()
                } else {
                    format!("build {} from Build & Dig", building.name)
                }
            })
            .unwrap_or_else(|| "available in Build & Dig".to_owned()),
        "unlock_creature" => {
            let route = match unlock.id.as_str() {
                "slime_janitor" | "bat_courier" => "recruit from Jobs",
                _ => "breed at the Breeding Pit",
            };
            if crate::ui::legibility::advanced_systems_unlocked(session) {
                route.to_owned()
            } else {
                format!("{route} after onboarding")
            }
        }
        "guard_dps_mult" => format!("Guards deal +{:.0}% damage", (unlock.value - 1.0) * 100.0),
        "farm_cap_mult" => format!("Farms hold +{:.0}% food", (unlock.value - 1.0) * 100.0),
        "beetle_carry_mult" => {
            format!("Beetle Haulers carry +{:.0}%", (unlock.value - 1.0) * 100.0)
        }
        "breed_interval_mult" => format!(
            "Breeding Pits hatch {:.0}% sooner",
            (1.0 - unlock.value) * 100.0
        ),
        "unlock_equipment" => "queue it at the Blacksmith".to_owned(),
        _ => unlock.description.trim_end_matches('.').to_owned(),
    };

    format!("Unlocked: {} — {detail}.", unlock.name)
}

fn advanced_building_hidden(session: &GameSession, building_id: &str) -> bool {
    !matches!(
        building_id,
        "blacksmith" | "cook_pot" | "farm" | "mine" | "worm_shrine"
    ) && !crate::ui::legibility::advanced_systems_unlocked(session)
}
