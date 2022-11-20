use thallium::math::Vector2;

pub struct Circle {
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub mass: f64,
    pub radius: f64,
}
