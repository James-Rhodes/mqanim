use std::ops::Range;

use macroquad::prelude::*;

use crate::{map, ui::draw_text_centered};

#[derive(Copy, Clone)]
pub struct LabelStyle {
    pub pos_offset: Vec2,
    pub color: Color,
    pub font_size: u16,
    pub decimal_places: usize,
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            pos_offset: vec2(0., 0.),
            color: WHITE,
            font_size: 12,
            decimal_places: 2,
        }
    }
}

#[derive(Copy, Clone)]
pub struct MarkerStyle {
    pub length: f32,
    pub thickness: f32,
    pub color: Color,
}
impl Default for MarkerStyle {
    fn default() -> Self {
        Self {
            length: 5.,
            thickness: 2.,
            color: WHITE,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub enum TickStyle {
    LabelAndMarker {
        label_style: LabelStyle,
        marker_style: MarkerStyle,
    },
    Marker {
        style: MarkerStyle,
    },
    Label {
        style: LabelStyle,
    },

    #[default]
    Nothing,
}

pub enum GraphEndPointStyle {
    Arrow { thickness: f32 },
    Nothing,
}

impl Default for GraphEndPointStyle {
    fn default() -> Self {
        GraphEndPointStyle::Arrow { thickness: 7. }
    }
}

pub struct AxisStyle {
    pub tick_step: f32,
    pub tick_style: TickStyle,
    pub end_point_style: GraphEndPointStyle,
    pub line_thickness: f32,
    pub line_color: Color,
}

impl Default for AxisStyle {
    fn default() -> Self {
        AxisStyle {
            tick_step: 0.5,
            tick_style: TickStyle::default(),
            end_point_style: GraphEndPointStyle::default(),
            line_thickness: 3.,
            line_color: WHITE,
        }
    }
}

#[derive(Default)]
pub struct GraphStyle {
    pub x_style: AxisStyle,
    pub y_style: AxisStyle,
}

pub struct Graph {
    world_center_pos: Vec2,
    world_size: Vec2,
    x_range: Range<f32>,
    y_range: Range<f32>,
    style: GraphStyle,
    world_min_coords: Vec2,
    world_max_coords: Vec2,
    axes_pos: Vec2, // The position where the x and y axis cross
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Graph {
    pub fn new(
        world_center_pos: Vec2,
        world_size: Vec2,
        x_range: Range<f32>,
        y_range: Range<f32>,
    ) -> Self {
        assert!(
            x_range.start < x_range.end,
            "The x_range must have start smaller than end"
        );
        assert!(
            y_range.start < y_range.end,
            "The y_range must have start smaller than end"
        );

        let mut graph = Self {
            world_center_pos,
            world_size,
            x_range,
            y_range,
            style: GraphStyle::default(),
            world_min_coords: vec2(
                world_center_pos.x - world_size.x / 2.,
                world_center_pos.y - world_size.y / 2.,
            ),
            world_max_coords: vec2(
                world_center_pos.x + world_size.x / 2.,
                world_center_pos.y + world_size.y / 2.,
            ),
            axes_pos: vec2(0., 0.),
        };

        graph.axes_pos = graph.graph_to_world(vec2(0.0, 0.0));
        if !graph.y_range.contains(&0.) && graph.y_range.end != 0. {
            graph.axes_pos.y = graph.world_min_coords.y;
        }
        if !graph.x_range.contains(&0.) && graph.x_range.end != 0. {
            graph.axes_pos.x = graph.world_min_coords.x;
        }

        graph
    }
    pub fn style(mut self, style: GraphStyle) -> Self {
        self.style = style;
        self
    }
    pub fn world_center_pos(&self) -> Vec2 {
        self.world_center_pos
    }

    pub fn world_size(&self) -> Vec2 {
        self.world_size
    }

    pub fn draw_axes(&self) {
        // Draw X Axis
        draw_line(
            self.world_min_coords.x,
            self.axes_pos.y,
            self.world_max_coords.x,
            self.axes_pos.y,
            self.style.y_style.line_thickness,
            self.style.y_style.line_color,
        );

        // Draw Y Axis
        draw_line(
            self.axes_pos.x,
            self.world_min_coords.y,
            self.axes_pos.x,
            self.world_max_coords.y,
            self.style.x_style.line_thickness,
            self.style.x_style.line_color,
        );

        self.draw_axes_end_pts();
        self.draw_ticks();
    }
    fn draw_ticks(&self) {
        if matches!(self.style.x_style.tick_style, TickStyle::Nothing)
            && matches!(self.style.y_style.tick_style, TickStyle::Nothing)
        {
            return;
        }

        // X axis
        let tick_start = if self.x_range.contains(&0.) {
            0.
        } else if self.x_range.start < 0. && self.x_range.end < 0. {
            self.x_range.end
        } else {
            self.x_range.start
        };

        let num_ticks = if self.x_range.start == 0. || self.x_range.end == 0. {
            ((self.x_range.end - self.x_range.start) / self.style.x_style.tick_step) + 1.
        } else {
            (self.x_range.end - self.x_range.start) / self.style.x_style.tick_step
        };
        let num_ticks = num_ticks.round() as usize;
        for tick_num in 0..num_ticks {
            let above = tick_start + tick_num as f32 * self.style.x_style.tick_step;
            let below = tick_start - tick_num as f32 * self.style.x_style.tick_step;

            if above <= self.x_range.end
                && (self.x_range.start == 0. || self.x_range.end == 0. || above != 0.)
            {
                let pt = self.graph_to_world(vec2(above, 0.));
                self.draw_tick(
                    pt,
                    above,
                    Orientation::Horizontal,
                    self.style.x_style.tick_style,
                );
            }
            if below >= self.x_range.start
                && (self.x_range.start == 0. || self.x_range.end == 0. || below != 0.)
            {
                let pt = self.graph_to_world(vec2(below, 0.));
                self.draw_tick(
                    pt,
                    below,
                    Orientation::Horizontal,
                    self.style.x_style.tick_style,
                );
            }
        }

        // Y axis
        let tick_start = if self.y_range.contains(&0.) {
            0.
        } else if self.y_range.start < 0. && self.y_range.end < 0. {
            self.y_range.end
        } else {
            self.y_range.start
        };

        let num_ticks = if self.y_range.start == 0. || self.y_range.end == 0. {
            ((self.y_range.end - self.y_range.start) / self.style.y_style.tick_step) + 1.
        } else {
            (self.y_range.end - self.y_range.start) / self.style.y_style.tick_step
        };
        let num_ticks = num_ticks.round() as usize;
        for tick_num in 0..num_ticks {
            let above = tick_start + tick_num as f32 * self.style.y_style.tick_step;
            let below = tick_start - tick_num as f32 * self.style.y_style.tick_step;

            if above <= self.y_range.end
                && (self.y_range.start == 0. || self.y_range.end == 0. || above != 0.)
            {
                let pt = self.graph_to_world(vec2(0., above));
                self.draw_tick(
                    pt,
                    above,
                    Orientation::Vertical,
                    self.style.y_style.tick_style,
                );
            }
            if below >= self.y_range.start
                && (self.y_range.start == 0. || self.y_range.end == 0. || below != 0.)
            {
                let pt = self.graph_to_world(vec2(0., below));
                self.draw_tick(
                    pt,
                    below,
                    Orientation::Vertical,
                    self.style.y_style.tick_style,
                );
            }
        }
    }

    fn draw_tick(&self, pos: Vec2, value: f32, orientation: Orientation, style: TickStyle) {
        let offset = -20.;
        let vert_offset = vec2(offset, 0.);
        let hori_offset = vec2(0., offset / 2.);
        let (line_params, label_style) = match (&orientation, style) {
            (
                Orientation::Horizontal,
                TickStyle::LabelAndMarker {
                    label_style,
                    marker_style,
                },
            ) => {
                let label_pos = pos + label_style.pos_offset + hori_offset;
                (
                    Some((
                        [
                            vec2(pos.x, pos.y + marker_style.length / 2.),
                            vec2(pos.x, pos.y - marker_style.length / 2.),
                        ],
                        marker_style.thickness,
                        marker_style.color,
                    )),
                    Some(LabelStyle {
                        pos_offset: label_pos,
                        ..label_style
                    }),
                )
            }
            (Orientation::Horizontal, TickStyle::Marker { style }) => (
                Some((
                    [
                        vec2(pos.x, pos.y + style.length / 2.),
                        vec2(pos.x, pos.y - style.length / 2.),
                    ],
                    style.thickness,
                    style.color,
                )),
                None,
            ),
            (Orientation::Horizontal, TickStyle::Label { style }) => {
                let label_pos = pos + style.pos_offset + hori_offset;
                (
                    None,
                    Some(LabelStyle {
                        pos_offset: label_pos,
                        ..style
                    }),
                )
            }
            (
                Orientation::Vertical,
                TickStyle::LabelAndMarker {
                    label_style,
                    marker_style,
                },
            ) => {
                let label_pos = pos + label_style.pos_offset + vert_offset;
                (
                    Some((
                        [
                            vec2(pos.x + marker_style.length / 2., pos.y),
                            vec2(pos.x - marker_style.length / 2., pos.y),
                        ],
                        marker_style.thickness,
                        marker_style.color,
                    )),
                    Some(LabelStyle {
                        pos_offset: label_pos,
                        ..label_style
                    }),
                )
            }
            (Orientation::Vertical, TickStyle::Marker { style }) => (
                Some((
                    [
                        vec2(pos.x + style.length / 2., pos.y),
                        vec2(pos.x - style.length / 2., pos.y),
                    ],
                    style.thickness,
                    style.color,
                )),
                None,
            ),
            (Orientation::Vertical, TickStyle::Label { style }) => {
                let label_pos = pos + style.pos_offset + vert_offset;
                (
                    None,
                    Some(LabelStyle {
                        pos_offset: label_pos,
                        ..style
                    }),
                )
            }
            (_, TickStyle::Nothing) => (None, None),
        };

        if let Some((line_pts, line_thickness, line_color)) = line_params {
            draw_line(
                line_pts[0].x,
                line_pts[0].y,
                line_pts[1].x,
                line_pts[1].y,
                line_thickness,
                line_color,
            )
        }
        if let Some(label_style) = label_style {
            let dp = label_style.decimal_places;
            draw_text_centered(
                &format!("{value:.dp$}"),
                label_style.pos_offset.x,
                label_style.pos_offset.y,
                label_style.font_size,
                label_style.color,
            )
        }
    }

    fn draw_axes_end_pts(&self) {
        let zero_position = self.graph_to_world(vec2(0.0, 0.0));
        // TODO: Make this into a function rather than the copy pasta below
        match self.style.x_style.end_point_style {
            GraphEndPointStyle::Arrow { thickness } => {
                let min_x = self.world_min_coords.x - thickness;
                let max_x = self.world_max_coords.x + thickness;
                let y = zero_position.y;
                if self.world_min_coords.x != zero_position.x {
                    draw_triangle(
                        vec2(min_x, y),
                        vec2(min_x + thickness, y + thickness),
                        vec2(min_x + thickness, y - thickness),
                        self.style.x_style.line_color,
                    );
                }
                if self.world_max_coords.x != zero_position.x {
                    draw_triangle(
                        vec2(max_x, y),
                        vec2(max_x - thickness, y + thickness),
                        vec2(max_x - thickness, y - thickness),
                        self.style.x_style.line_color,
                    );
                }
            }
            GraphEndPointStyle::Nothing => (),
        };
        match self.style.y_style.end_point_style {
            GraphEndPointStyle::Arrow { thickness } => {
                let min_y = self.world_min_coords.y - thickness;
                let max_y = self.world_max_coords.y + thickness;
                let x = zero_position.x;
                if self.world_min_coords.y != zero_position.y {
                    draw_triangle(
                        vec2(x, min_y),
                        vec2(x + thickness, min_y + thickness),
                        vec2(x - thickness, min_y + thickness),
                        self.style.x_style.line_color,
                    );
                }
                if self.world_max_coords.x != zero_position.y {
                    draw_triangle(
                        vec2(x, max_y),
                        vec2(x + thickness, max_y - thickness),
                        vec2(x - thickness, max_y - thickness),
                        self.style.x_style.line_color,
                    );
                }
            }
            GraphEndPointStyle::Nothing => (),
        };
    }
    pub fn graph_to_world(&self, pt: Vec2) -> Vec2 {
        vec2(
            map(
                pt.x,
                self.x_range.start,
                self.x_range.end,
                self.world_min_coords.x,
                self.world_max_coords.x,
            ),
            map(
                pt.y,
                self.y_range.start,
                self.y_range.end,
                self.world_min_coords.y,
                self.world_max_coords.y,
            ),
        )
    }
    pub fn plot_line_vec(&self, pts: &[Vec2], thickness: f32, color: Color) {
        pts.windows(2).for_each(|slice| {
            let pt_a = self.graph_to_world(slice[0]);
            let pt_b = self.graph_to_world(slice[1]);
            self.plot_line_world(&pt_a, &pt_b, thickness, color);
        });
    }

    pub fn plot_line_xy(&self, x: &[f32], y: &[f32], thickness: f32, color: Color) {
        x.windows(2).zip(y.windows(2)).for_each(|(xs, ys)| {
            let pt_a = self.graph_to_world(vec2(xs[0], ys[0]));
            let pt_b = self.graph_to_world(vec2(xs[1], ys[1]));

            self.plot_line_world(&pt_a, &pt_b, thickness, color);
        })
    }
    pub fn plot_pt_vec(&self, pt: &Vec2, radius: f32, color: Color) {
        let pt = self.graph_to_world(*pt);
        if !self.world_pt_in_world_bb(&pt) {
            return;
        }

        draw_circle(pt.x, pt.y, radius, color);
    }

    pub fn plot_pt_xy(&self, x: f32, y: f32, radius: f32, color: Color) {
        let pt = self.graph_to_world(vec2(x, y));
        if !self.world_pt_in_world_bb(&pt) {
            return;
        }

        draw_circle(pt.x, pt.y, radius, color);
    }
    fn plot_line_world(&self, pt_a: &Vec2, pt_b: &Vec2, thickness: f32, color: Color) {
        if !self.world_pt_in_world_bb(pt_a) && !self.world_pt_in_world_bb(pt_b) {
            // Neither point is on the graph so bail this iteration
            return;
        }
        // TODO: if a is in the graph but b isn't then clamp b at the nearest intersection point with
        // the nearest border. Vice versa for b. If both are off then don't draw anything

        draw_line(pt_a.x, pt_a.y, pt_b.x, pt_b.y, thickness, color);
    }
    fn world_pt_in_world_bb(&self, pt: &Vec2) -> bool {
        pt.x >= self.world_min_coords.x
            && pt.y >= self.world_min_coords.y
            && pt.x <= self.world_max_coords.x
            && pt.y <= self.world_max_coords.y
    }
}
