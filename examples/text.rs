use macroquad::prelude::*;
use mqanim::{ui::draw_text_centered, Animation};

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
fn window_conf() -> Conf {
    Conf {
        window_title: "Text Example".to_owned(),
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

    loop {
        animation.set_camera();
        draw_text_centered("Hello World From Droid Sans Mono", 0., 0., 20, WHITE);

        animation.set_default_camera();
        animation.draw_frame();

        next_frame().await;
    }
}
