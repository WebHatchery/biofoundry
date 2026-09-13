//! Frame rendering for menu, world, HUD, and notifications.

use super::*;

impl Game {
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
                        hud_panel: self.hud_panel,
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
}
