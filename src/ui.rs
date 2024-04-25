use super::map;
use macroquad::prelude::*;
use std::ops::Range;

pub fn draw_text_centered(text: &str, x: f32, y: f32, font_size: u16, color: Color) {
    let text_center = get_text_center(text, None, font_size, 1., 0.);
    draw_text_ex(
        text,
        x - text_center.x,
        y + text_center.y,
        TextParams {
            font_size,
            font_scale: -1.,
            font_scale_aspect: -1.,
            color,
            ..Default::default()
        },
    );
}

#[derive(Copy, Clone)]
pub struct SliderStyle {
    pub bar_height: f32,
    pub bar_color: Color,
    pub marker_color: Color,
}
impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            bar_height: 5.,
            bar_color: WHITE,
            marker_color: GRAY,
        }
    }
}

pub struct Slider {
    center_pos: Vec2,
    size: Vec2,
    style: SliderStyle,
    mouse_pos: Option<Vec2>,
    range: Range<f32>,
    min_coords: Vec2,
    max_coords: Vec2,
}
impl Slider {
    pub fn new(center_pos: Vec2, size: Vec2, range: Range<f32>) -> Self {
        let min_coords = vec2(center_pos.x - size.x / 2., center_pos.y - size.y / 2.);
        let max_coords = vec2(center_pos.x + size.x / 2., center_pos.y + size.y / 2.);
        Self {
            center_pos,
            size,
            style: SliderStyle::default(),
            mouse_pos: None,
            range,
            min_coords,
            max_coords,
        }
    }
    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = style;
        self
    }
    pub fn mouse_pos(mut self, mouse_pos: Vec2) -> Self {
        self.mouse_pos = Some(mouse_pos);
        self
    }
    pub fn draw(&self, data: &mut f32) {
        let mouse_pos = if let Some(mouse_pos) = self.mouse_pos {
            mouse_pos
        } else {
            mouse_position().into()
        };

        let draw_x = self.center_pos.x - self.size.x / 2.;
        let draw_y = self.center_pos.y - self.style.bar_height / 2.;
        draw_rectangle(
            draw_x,
            draw_y,
            self.size.x,
            self.style.bar_height,
            self.style.bar_color,
        );

        let marker_x = map(
            *data,
            self.range.start,
            self.range.end,
            -self.size.x / 2. + self.center_pos.x,
            self.size.x / 2. + self.center_pos.x,
        );
        let marker_pos = vec2(marker_x, self.center_pos.y);
        let mouse_intersects_bb = mouse_pos.x >= (self.min_coords.x - self.size.y / 2.)
            && mouse_pos.x <= (self.max_coords.x + self.size.y / 2.)
            && mouse_pos.y <= self.max_coords.y
            && mouse_pos.y >= self.min_coords.y;
        if is_mouse_button_down(MouseButton::Left) && mouse_intersects_bb {
            *data = map(
                mouse_pos.x,
                self.min_coords.x,
                self.max_coords.x,
                self.range.start,
                self.range.end,
            );
            *data = data.clamp(self.range.start, self.range.end);
        }
        draw_circle(
            marker_pos.x,
            marker_pos.y,
            self.size.y / 2.,
            self.style.marker_color,
        );
    }
}
