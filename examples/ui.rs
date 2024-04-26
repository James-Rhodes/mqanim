use macroquad::prelude::*;
use mqanim::Animation;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
fn window_conf() -> Conf {
    Conf {
        window_title: "UI Example".to_owned(),
        sample_count: 16,
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut animation = Animation::new(WINDOW_WIDTH, WINDOW_HEIGHT, None);
    // animation.enable_fxaa();

    let mut button_pushed = false;
    let mut circle_button_pushed = false;
    loop {
        animation.set_camera();
        mqanim::ui::Button::new(
            vec2(0., 0.),
            mqanim::ui::ButtonShape::Rectangle {
                width: 100.,
                height: 75.,
            },
        )
        .mouse_pos(animation.get_world_mouse())
        .draw(&mut button_pushed);

        mqanim::ui::Button::new(
            vec2(-100., 0.),
            mqanim::ui::ButtonShape::Circle { radius: 25. },
        )
        .button_type(mqanim::ui::ButtonType::Push)
        .mouse_pos(animation.get_world_mouse())
        .draw(&mut circle_button_pushed);

        animation.set_default_camera();
        animation.draw_frame();

        next_frame().await;
    }
}
