use serde::{Deserialize, Serialize};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Camera {
    pub pos: Point2,
    pub zoom: Point2,
}
