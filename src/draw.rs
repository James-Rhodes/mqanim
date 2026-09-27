use macroquad::prelude::*;

/// Draw a polyline with round joins and round caps.
///
/// `draw_line` draws each segment as an independent quad with flat (butt)
/// ends, so a polyline built from `draw_line` calls leaves wedge shaped gaps
/// at its corners. This fills every vertex with a small polygon, turning the
/// butt joins into round joins (and the ends into round caps).
///
/// Unlike drawing a circle at both ends of every segment, a join is drawn
/// once per vertex, and the number of sides used for the join scales with the
/// line thickness so that thin lines stay cheap.
pub fn draw_path(points: &[Vec2], thickness: f32, color: Color) {
    if points.len() < 2 || thickness <= 0. {
        return;
    }

    points.windows(2).for_each(|segment| {
        draw_line(
            segment[0].x,
            segment[0].y,
            segment[1].x,
            segment[1].y,
            thickness,
            color,
        );
    });

    let radius = thickness / 2.;
    let sides = join_sides(radius);
    points
        .iter()
        .for_each(|point| draw_poly(point.x, point.y, sides, radius, 0., color));
}

/// Sides to use for a join polygon of the given radius. `draw_circle` always
/// uses 20 sides; for the thin lines used in animations and plots a handful
/// is already enough to look round, especially with MSAA.
fn join_sides(radius: f32) -> u8 {
    (radius * 8.).clamp(8., 20.) as u8
}
