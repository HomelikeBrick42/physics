use std::collections::HashSet;

use lerp::Lerp;
use thallium::{
    math::{Vector2, Zero},
    renderer::{IndexBufferID, PrimitiveType, RendererDrawContext, ShaderID, VertexBufferID},
    scene::Transform,
};

pub struct Circle {
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub acceleration: Vector2<f64>,
    pub mass: f64,
    pub radius: f64,
}

impl Circle {
    pub fn add_force(&mut self, force: Vector2<f64>) {
        self.acceleration += force / self.mass.into();
    }

    pub fn get_energy(&self, bounds: Vector2<f64>, gravity: f64) -> f64 {
        let kinetic_energy = 0.5 * self.mass * self.velocity.sqr_length();
        let gravitational_potental_energy = self.mass * gravity * (self.position.y - -bounds.y);
        kinetic_energy + gravitational_potental_energy
    }
}

pub fn update_circles(circles: &mut [Circle], bounds: Vector2<f64>, gravity: f64, ts: f64) {
    for circle in circles.iter_mut() {
        circle.acceleration.y -= gravity;
        circle.velocity += circle.acceleration * ts.into();
        circle.position += circle.velocity * ts.into();
    }
    let mut collisions = HashSet::with_capacity(circles.len());
    loop {
        for i in 0..circles.len() {
            let a = &mut circles[i];
            if a.velocity.y > 0.0 && a.position.y + a.radius > bounds.y {
                //a.position.y -= (a.position.y + a.radius - bounds.y) * 2.0;
                a.velocity.y *= -1.0;
                collisions.insert(i);
                continue;
            }
            if a.velocity.y < 0.0 && a.position.y - a.radius < -bounds.y {
                //a.position.y += (a.position.y - a.radius - -bounds.y) * 2.0;
                a.velocity.y *= -1.0;
                collisions.insert(i);
                continue;
            }
            if a.velocity.x > 0.0 && a.position.x + a.radius > bounds.x {
                //a.position.x -= (a.position.x + a.radius - bounds.x) * 2.0;
                a.velocity.x *= -1.0;
                collisions.insert(i);
                continue;
            }
            if a.velocity.x < 0.0 && a.position.x - a.radius < -bounds.x {
                //a.position.x += (a.position.x - a.radius - -bounds.x) * 2.0;
                a.velocity.x *= -1.0;
                collisions.insert(i);
                continue;
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
        collisions.clear();
    }
    for circle in circles.iter_mut() {
        circle.acceleration = Vector2::zero();
    }
}

pub fn render_circles(
    draw_context: &mut dyn RendererDrawContext,
    shader: ShaderID,
    vertex_buffer: VertexBufferID,
    index_buffer: IndexBufferID,
    circles: &[Circle],
    bounds: Vector2<f64>,
    gravity: f64,
) {
    let total_energy: f64 = circles
        .iter()
        .map(|circle| circle.get_energy(bounds, gravity))
        .sum();
    println!("{total_energy}");
    for circle in circles {
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
                let energy = circle.get_energy(bounds, gravity);

                let slow_r = 50.0 / 255.0;
                let slow_g = 100.0 / 255.0;
                let slow_b = 120.0 / 255.0;

                let fast_r = 255.0 / 255.0;
                let fast_g = 100.0 / 255.0;
                let fast_b = 70.0 / 255.0;

                (
                    slow_r.lerp(fast_r, energy * circles.len() as f64 / total_energy) as f32,
                    slow_g.lerp(fast_g, energy * circles.len() as f64 / total_energy) as f32,
                    slow_b.lerp(fast_b, energy * circles.len() as f64 / total_energy) as f32,
                )
                    .into()
            },
        );
    }
}
