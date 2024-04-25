use macroquad::prelude::*;
use mqanim::{
    map,
    plot::{AxisStyle, Graph, GraphEndPointStyle, GraphStyle, LabelStyle, MarkerStyle, TickStyle},
    Animation,
};
use std::f32::consts::PI;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
fn window_conf() -> Conf {
    Conf {
        window_title: "Template".to_owned(),
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

    let x_sine: Vec<f32> = (-100..100)
        .map(|val| map(val as f32, -100.0, 100.0, -2. * PI, 2. * PI))
        .collect();
    let y_sine: Vec<f32> = (-100..100)
        .map(|val| {
            let t = map(val as f32, -100.0, 100.0, -2. * PI, 2. * PI);
            f32::sin(t)
        })
        .collect();

    let mut time = 0.;
    loop {
        time += 0.01;
        let sine: Vec<Vec2> = (-500..500)
            .map(|val| {
                let t = map(val as f32, -100.0, 100.0, -2. * PI, 2. * PI);
                vec2(t, -f32::sin(t + time))
            })
            .collect();
        animation.set_camera();
        let _mouse = animation.get_world_mouse();

        let graph = Graph::new(
            vec2(0., 0.),
            vec2(WINDOW_WIDTH - 100., WINDOW_HEIGHT - 100.),
            -3.5..3.5,
            -3.5..3.5,
        )
        .style(GraphStyle {
            x_style: AxisStyle {
                tick_step: 0.5,
                tick_style: TickStyle::LabelAndMarker {
                    label_style: LabelStyle {
                        pos_offset: vec2(0., 0.),
                        color: WHITE,
                        font_size: 12,
                        decimal_places: 2,
                    },
                    marker_style: MarkerStyle {
                        length: 5.,
                        thickness: 2.,
                        color: WHITE,
                    },
                },
                end_point_style: GraphEndPointStyle::Arrow { thickness: 7. },
                line_thickness: 3.,
                line_color: WHITE,
            },
            y_style: AxisStyle {
                tick_step: 0.5,
                tick_style: TickStyle::LabelAndMarker {
                    label_style: LabelStyle {
                        pos_offset: vec2(0., 0.),
                        color: WHITE,
                        font_size: 12,
                        decimal_places: 2,
                    },
                    marker_style: MarkerStyle {
                        length: 5.,
                        thickness: 2.,
                        color: WHITE,
                    },
                },
                end_point_style: GraphEndPointStyle::Arrow { thickness: 7. },
                line_thickness: 3.,
                line_color: WHITE,
            },
        });
        graph.draw_axes();
        graph.plot_line_vec(&sine, 3., PURPLE);
        graph.plot_line_xy(&x_sine, &y_sine, 3., BLUE);
        let pt = graph.graph_to_world(vec2(-0.5, -0.5));
        draw_circle(pt.x, pt.y, 10., ORANGE);
        graph.plot_pt_vec(&vec2(0.2, 0.2), 10., RED);
        graph.plot_pt_xy(0.4, 0.4, 15., BLUE);

        animation.set_default_camera();
        animation.draw_frame();

        next_frame().await;
    }
}
