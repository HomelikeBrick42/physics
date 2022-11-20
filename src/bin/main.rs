use std::f64::consts::PI;

use physics::{render_circles, update_circles, Circle};
use rand::Rng;
use thallium::{
    math::{Vector2, Vector3, Zero},
    platform::{Surface, SurfaceEvent},
    renderer::{PrimitiveType, RendererAPI},
    scene::{Camera, CameraProjectionType, Transform},
};

const SCALE: f64 = 10.0;
const FIXED_UPDATE_RATE: f64 = 1.0 / 200.0;
const BOUNDS: Vector2<f64> = Vector2 {
    x: SCALE * 0.95,
    y: SCALE * 0.95,
};
const CIRCLE_COUNT: usize = 5;
const GRAVITY: f64 = 9.81;

fn main() {
    let mut renderer =
        Surface::new((640, 480).into(), "Physics").into_renderer(RendererAPI::OpenGL);

    let circles_shader = renderer
        .create_shader(
            include_str!("./gen_quad.vert.glsl"),
            include_str!("./circle.frag.glsl"),
        )
        .unwrap();
    let quad_shader = renderer
        .create_shader(
            include_str!("./gen_quad.vert.glsl"),
            include_str!("./basic.frag.glsl"),
        )
        .unwrap();

    let vertex_buffer = renderer.create_vertex_buffer(&[], &[]);
    let index_buffer = renderer.create_index_buffer(&[0, 1, 2, 3]);

    let mut camera = Camera::default();

    let mut circles = vec![Circle {
        position: Vector2::zero(),
        velocity: Vector2::zero(),
        acceleration: Vector2::zero(),
        mass: 1.0,
        radius: 1.0,
    }];
    // let mut circles = std::iter::repeat_with({
    //     let mut rng = rand::thread_rng();
    //     move || {
    //         let radius = rng.gen_range(0.5..1.0);
    //         let position_range = -BOUNDS.x + radius..BOUNDS.x - radius;
    //         let velocity_range = -10.0..10.0;
    //         Circle {
    //             position: (
    //                 rng.gen_range(position_range.clone()),
    //                 rng.gen_range(position_range),
    //             )
    //                 .into(),
    //             velocity: (
    //                 rng.gen_range(velocity_range.clone()),
    //                 rng.gen_range(velocity_range),
    //             )
    //                 .into(),
    //             acceleration: Vector2::zero(),
    //             mass: PI * radius * radius,
    //             radius,
    //         }
    //     }
    // })
    // .take(CIRCLE_COUNT)
    // .collect::<Vec<_>>();

    let mut last_time = std::time::Instant::now();

    let mut fixed_update_time = 0.0;

    renderer.get_surface_mut().show();
    'game_loop: loop {
        let time = std::time::Instant::now();
        let ts = time.duration_since(last_time).as_secs_f64();
        last_time = time;

        for event in renderer.get_surface_mut().events() {
            match event {
                SurfaceEvent::Close => break 'game_loop,
                SurfaceEvent::Resize(size) => {
                    renderer.resize(size);
                    let aspect = size.x as f32 / size.y as f32;
                    camera.projection_type = CameraProjectionType::Orthographic {
                        left: -aspect * SCALE as f32,
                        right: aspect * SCALE as f32,
                        top: SCALE as f32,
                        bottom: -SCALE as f32,
                        near: -1.0,
                        far: 1.0,
                    };
                }
                _ => {}
            }
        }

        fixed_update_time += ts;
        while fixed_update_time >= FIXED_UPDATE_RATE {
            #[allow(unused_variables)]
            let start = std::time::Instant::now();
            update_circles(&mut circles, BOUNDS, GRAVITY, FIXED_UPDATE_RATE);
            // println!("{}", start.elapsed().as_secs_f64());
            fixed_update_time -= FIXED_UPDATE_RATE;
        }

        renderer.clear((0.1, 0.1, 0.1).into());
        {
            let mut draw_context = renderer.drawing_context(camera, false);
            draw_context.draw_indexed(
                PrimitiveType::TriangleStrip,
                quad_shader,
                vertex_buffer,
                index_buffer,
                None,
                Transform {
                    scale: (BOUNDS.x as f32, BOUNDS.y as f32, 1.0).into(),
                    ..Default::default()
                }
                .into(),
                Vector3::zero(),
            );
            render_circles(
                draw_context.as_mut(),
                circles_shader,
                vertex_buffer,
                index_buffer,
                &circles,
                BOUNDS,
                GRAVITY,
            );
        }
        renderer.present();
    }
    renderer.get_surface_mut().hide();
}
