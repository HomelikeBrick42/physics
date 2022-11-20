use std::collections::HashSet;

use lerp::Lerp;
use physics::Circle;
use thallium::{
    math::Vector2,
    platform::{Surface, SurfaceEvent},
    renderer::{PrimitiveType, RendererAPI},
    scene::{Camera, CameraProjectionType, Transform},
};

fn main() {
    let mut renderer =
        Surface::new((640, 480).into(), "Physics").into_renderer(RendererAPI::OpenGL);

    let shader = renderer
        .create_shader(
            include_str!("./basic.vert.glsl"),
            include_str!("./basic.frag.glsl"),
        )
        .unwrap();

    let vertex_buffer = renderer.create_vertex_buffer(&[], &[]);
    let index_buffer = renderer.create_index_buffer(&[0, 1, 2, 3]);

    let mut camera = Camera::default();

    let mut circles = std::iter::repeat_with(|| {
        let radius = 1.0;
        Circle {
            position: (
                (rand::random::<f64>() * 2.0 - 1.0) * BOUNDS.x - radius,
                (rand::random::<f64>() * 2.0 - 1.0) * BOUNDS.y - radius,
            )
                .into(),
            velocity: (
                (rand::random::<f64>() * 2.0 - 1.0) * 6.0,
                (rand::random::<f64>() * 2.0 - 1.0) * 6.0,
            )
                .into(),
            mass: 1.0,
            radius,
        }
    })
    .take(60)
    .collect::<Vec<_>>();

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
            update(&mut circles, FIXED_UPDATE_RATE);
            let energy: f64 = circles
                .iter()
                .map(|circle| {
                    let speed = circle.velocity.length();
                    0.5 * circle.mass * speed * speed
                })
                .sum();
            _ = energy;
            fixed_update_time -= FIXED_UPDATE_RATE;
        }

        renderer.clear((0.1, 0.1, 0.1).into());
        {
            let mut draw_context = renderer.drawing_context(camera, false);
            for circle in &circles {
                draw_context.draw_indexed(
                    PrimitiveType::TriangleStrip,
                    shader,
                    vertex_buffer,
                    index_buffer,
                    None,
                    Transform {
                        position: (circle.position.x as _, circle.position.y as _, 0.0).into(),
                        scale: (circle.radius as _, circle.radius as _, 1.0).into(),
                        ..Default::default()
                    }
                    .into(),
                    {
                        let speed = circle.velocity.length();
                        let energy = 0.5 * circle.mass * speed * speed;

                        let slow_r = 50.0 / 255.0;
                        let slow_g = 100.0 / 255.0;
                        let slow_b = 120.0 / 255.0;

                        let fast_r = 255.0 / 255.0;
                        let fast_g = 100.0 / 255.0;
                        let fast_b = 70.0 / 255.0;

                        const DIVISOR: f64 = 2.0;
                        (
                            slow_r.lerp(fast_r, energy / DIVISOR) as f32,
                            slow_g.lerp(fast_g, energy / DIVISOR) as f32,
                            slow_b.lerp(fast_b, energy / DIVISOR) as f32,
                        )
                            .into()
                    },
                );
            }
        }
        renderer.present();
    }
    renderer.get_surface_mut().hide();
}

const SCALE: f64 = 20.0;
const FIXED_UPDATE_RATE: f64 = 1.0 / 60.0;
const BOUNDS: Vector2<f64> = Vector2 { x: SCALE, y: SCALE };

fn update(circles: &mut [Circle], ts: f64) {
    for circle in circles.iter_mut() {
        circle.position += circle.velocity * ts.into();
    }
    loop {
        let mut collisions = HashSet::new();
        for i in 0..circles.len() {
            let a = &mut circles[i];
            if a.velocity.y > 0.0 && a.position.y + a.radius > BOUNDS.y {
                a.velocity.y *= -1.0;
            }
            if a.velocity.y < 0.0 && a.position.y - a.radius < -BOUNDS.y {
                a.velocity.y *= -1.0;
            }
            if a.velocity.x > 0.0 && a.position.x + a.radius > BOUNDS.x {
                a.velocity.x *= -1.0;
            }
            if a.velocity.x < 0.0 && a.position.x - a.radius < -BOUNDS.x {
                a.velocity.x *= -1.0;
            }
            for j in i + 1..circles.len() {
                let (a, b) = {
                    // random stuff to get 2 mutable references to the same array, the compiler cant guarantee that `i != j`
                    let (start, end) = circles.split_at_mut(i + 1);
                    (&mut start[i], &mut end[j - i - 1])
                };
                let a_to_b = b.position - a.position;
                let intersecting =
                    a_to_b.sqr_length() <= (a.radius + b.radius) * (a.radius + b.radius);
                let moving_towards_eachother = a_to_b.dot(a.velocity - b.velocity) > 0.0;
                if intersecting && moving_towards_eachother && !collisions.contains(&j) {
                    let a_velocity = a.velocity
                        - (a.position - b.position)
                            * (((2.0 * b.mass) / (a.mass + b.mass))
                                * ((a.velocity - b.velocity).dot(a.position - b.position)
                                    / (a.position - b.position).sqr_length()))
                            .into();
                    let b_velocity = b.velocity
                        - (b.position - a.position)
                            * (((2.0 * a.mass) / (a.mass + b.mass))
                                * ((b.velocity - a.velocity).dot(b.position - a.position)
                                    / (b.position - a.position).sqr_length()))
                            .into();
                    a.velocity = a_velocity;
                    b.velocity = b_velocity;
                    collisions.insert(j);
                    break;
                }
            }
        }
        if collisions.is_empty() {
            break;
        }
    }
}
