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

pub enum ButtonShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}
impl ButtonShape {
    pub fn intersects(&self, center_pos: Vec2, pt: Vec2) -> bool {
        match self {
            ButtonShape::Circle { radius } => {
                if center_pos.distance_squared(pt) < radius * radius {
                    return true;
                }

                false
            }
            ButtonShape::Rectangle { width, height } => {
                if pt.x >= center_pos.x - width / 2.
                    && pt.x <= center_pos.x + width / 2.
                    && pt.y >= center_pos.y - height / 2.
                    && pt.y <= center_pos.y + height / 2.
                {
                    return true;
                }

                false
            }
        }
    }
}
pub struct ButtonStyle {
    pub color: Color,
    pub inset_color: Color,
    pub hover_color: Color,
    pub hover_inset_color: Color,
    pub pushed_color: Color,
    pub pushed_inset_color: Color,
    pub inset_offset: f32,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            color: GRAY,
            inset_color: DARKGRAY,
            hover_color: DARKGRAY,
            hover_inset_color: BLACK,
            pushed_color: GRAY,
            pushed_inset_color: BLUE,
            inset_offset: 4.,
        }
    }
}

#[derive(Default)]
pub enum ButtonType {
    Push,
    #[default]
    Toggle,
}

pub struct Button {
    pub style: ButtonStyle,
    pub shape: ButtonShape,
    pub center_pos: Vec2,
    pub button_type: ButtonType,
    mouse_pos: Option<Vec2>,
}

impl Button {
    pub fn new(center_pos: Vec2, button_shape: ButtonShape) -> Self {
        Self {
            center_pos,
            shape: button_shape,
            style: ButtonStyle::default(),
            button_type: ButtonType::default(),
            mouse_pos: None,
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn button_type(mut self, button_type: ButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    pub fn mouse_pos(mut self, mouse_pos: Vec2) -> Self {
        self.mouse_pos = Some(mouse_pos);
        self
    }

    pub fn draw(&mut self, pushed: &mut bool) {
        let mouse_pos = self.mouse_pos.unwrap_or_else(|| mouse_position().into());
        let mut is_hovered = self.shape.intersects(self.center_pos, mouse_pos);

        if is_hovered
            && (is_mouse_button_pressed(MouseButton::Left)
                || (matches!(self.button_type, ButtonType::Push)
                    && is_mouse_button_down(MouseButton::Left)))
        {
            is_hovered = false;
            *pushed = match self.button_type {
                ButtonType::Push => true,
                ButtonType::Toggle => !*pushed,
            };
        } else {
            *pushed = match self.button_type {
                ButtonType::Push => false,
                ButtonType::Toggle => *pushed,
            };
        }

        let (inset_color, color) = match (*pushed, is_hovered) {
            (true, _) => (self.style.pushed_inset_color, self.style.pushed_color),
            (false, true) => (self.style.hover_inset_color, self.style.hover_color),
            (false, false) => (self.style.inset_color, self.style.color),
        };

        match self.shape {
            ButtonShape::Circle { radius } => {
                draw_circle(self.center_pos.x, self.center_pos.y, radius, color);
                draw_circle(
                    self.center_pos.x,
                    self.center_pos.y,
                    radius - self.style.inset_offset,
                    inset_color,
                );
            }
            ButtonShape::Rectangle { width, height } => {
                let draw_pos = vec2(
                    self.center_pos.x - width / 2.,
                    self.center_pos.y - height / 2.,
                );
                let inset_offset = self.style.inset_offset / 2.;
                draw_rectangle(draw_pos.x, draw_pos.y, width, height, color);
                draw_rectangle(
                    draw_pos.x + inset_offset,
                    draw_pos.y + inset_offset,
                    width - inset_offset * 2.,
                    height - inset_offset * 2.,
                    inset_color,
                );
            }
        }
    }
}
