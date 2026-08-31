//! World-space feedback for the awakened Worm Shrine route network.

use crate::data::GameData;
use crate::state::outposts::TransitDirection;
use crate::state::GameSession;
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;

/// Draw the shrine-to-Outpost links beneath buildings and creatures. The
/// links make the post-campaign network visible in the world, while the
/// ledger remains the place for detailed route decisions.
pub(super) fn draw_route_links(session: &GameSession, data: &GameData, tile_size: f32) {
    if !session.worm_awake || !tile_size.is_finite() || tile_size <= 0.0 {
        return;
    }
    let Some(shrine) = session
        .buildings
        .iter()
        .find(|building| building.kind == "worm_shrine")
    else {
        return;
    };
    let shrine_center = tile_center(shrine.pos, tile_size);

    for outpost in &session.outposts {
        if !session
            .building_at(outpost.pos)
            .is_some_and(|building| building.kind == "outpost")
        {
            continue;
        }
        let outpost_center = tile_center(outpost.pos, tile_size);
        draw_route_line(shrine_center, outpost_center, tile_size, outpost.active);
        if outpost.waypoint_upgraded {
            draw_waypoint_marker(shrine_center, outpost_center, tile_size);
        }
        if let Some(transit) = session
            .worm_transit
            .as_ref()
            .filter(|transit| transit.outpost == outpost.pos)
        {
            let fraction = transit_route_fraction(
                transit.remaining,
                transit_total_sec(outpost, data),
                transit.direction,
            );
            let pulse = route_point(shrine_center, outpost_center, fraction);
            draw_circle(
                pulse.x,
                pulse.y,
                tile_size * 0.20,
                Color::new(0.04, 0.06, 0.10, 0.90),
            );
            draw_circle_lines(
                pulse.x,
                pulse.y,
                tile_size * 0.16,
                2.0,
                Color::new(0.72, 0.90, 1.0, 0.98),
            );
        }
    }
}

fn transit_total_sec(outpost: &crate::state::outposts::Outpost, data: &GameData) -> f32 {
    crate::simulation::outposts::transit_time_sec(outpost, data)
}

fn tile_center(pos: TilePos, tile_size: f32) -> Vec2 {
    vec2(
        (pos.x as f32 + 0.5) * tile_size,
        (pos.y as f32 + 0.5) * tile_size,
    )
}

fn draw_route_line(from: Vec2, to: Vec2, tile_size: f32, active: bool) {
    let shadow = Color::new(0.02, 0.03, 0.05, if active { 0.54 } else { 0.38 });
    draw_line(from.x, from.y, to.x, to.y, tile_size * 0.16, shadow);

    if active {
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            tile_size * 0.065,
            Color::new(0.42, 0.75, 0.95, 0.58),
        );
        draw_circle_lines(
            to.x,
            to.y,
            tile_size * 0.47,
            1.5,
            Color::new(0.42, 0.75, 0.95, 0.58),
        );
    } else {
        draw_dashed_line(
            from,
            to,
            tile_size * 0.55,
            tile_size * 0.42,
            tile_size * 0.05,
            Color::new(0.58, 0.61, 0.70, 0.36),
        );
        draw_circle_lines(
            to.x,
            to.y,
            tile_size * 0.42,
            1.5,
            Color::new(0.58, 0.61, 0.70, 0.34),
        );
    }
}

fn draw_waypoint_marker(from: Vec2, to: Vec2, tile_size: f32) {
    let marker = route_point(from, to, 0.5);
    let radius = tile_size * 0.24;
    draw_circle(
        marker.x,
        marker.y,
        radius * 1.45,
        Color::new(0.02, 0.03, 0.05, 0.88),
    );
    draw_circle(
        marker.x,
        marker.y,
        radius,
        Color::new(0.96, 0.72, 0.25, 0.96),
    );
    draw_line(
        marker.x - radius * 0.7,
        marker.y,
        marker.x + radius * 0.7,
        marker.y,
        tile_size * 0.045,
        Color::new(1.0, 0.92, 0.62, 0.98),
    );
    draw_line(
        marker.x,
        marker.y - radius * 0.7,
        marker.x,
        marker.y + radius * 0.7,
        tile_size * 0.045,
        Color::new(1.0, 0.92, 0.62, 0.98),
    );
}

fn draw_dashed_line(from: Vec2, to: Vec2, dash: f32, gap: f32, width: f32, color: Color) {
    let delta = to - from;
    let length = delta.length();
    if !length.is_finite() || length <= 0.0 || dash <= 0.0 || gap < 0.0 {
        return;
    }
    let direction = delta / length;
    let mut start = 0.0;
    while start < length {
        let end = (start + dash).min(length);
        let segment_start = from + direction * start;
        let segment_end = from + direction * end;
        draw_line(
            segment_start.x,
            segment_start.y,
            segment_end.x,
            segment_end.y,
            width,
            color,
        );
        start += dash + gap;
    }
}

fn transit_route_fraction(remaining: f32, total: f32, direction: TransitDirection) -> f32 {
    let progress = if total.is_finite() && total > 0.0 {
        (remaining / total).clamp(0.0, 1.0)
    } else {
        0.0
    };
    match direction {
        TransitDirection::ToOutpost => 1.0 - progress,
        TransitDirection::ToShrine => progress,
    }
}

fn route_point(from: Vec2, to: Vec2, fraction: f32) -> Vec2 {
    from + (to - from) * fraction.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests;
