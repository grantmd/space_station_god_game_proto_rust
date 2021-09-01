use serde::{Deserialize, Serialize};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

const MOVE_AMOUNT: f32 = 10.0;
const ZOOM_AMOUNT: f32 = 2.0; // TODO: Tweak this multiple

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Camera {
    pub pos: Point2,
    pub zoom: Point2,
}

impl Camera {
    // Creates a new camera, centered and zoomed to the middle of the screen
    pub fn new() -> Camera {
        Camera {
            pos: Point2::zero(),
            zoom: Point2::one(),
        }
    }

    // Moves the camera "up"
    pub fn move_up(&mut self) {
        self.pos -= Point2::unit_y() * MOVE_AMOUNT;
    }

    // Moves the camera "down"
    pub fn move_down(&mut self) {
        self.pos += Point2::unit_y() * MOVE_AMOUNT;
    }

    // Moves the camera to the "left"
    pub fn move_left(&mut self) {
        self.pos -= Point2::unit_x() * MOVE_AMOUNT;
    }

    // Moves the camera to the "right"
    pub fn move_right(&mut self) {
        self.pos += Point2::unit_x() * MOVE_AMOUNT;
    }

    pub fn zoom(&mut self, amount: f32) {
        self.zoom += Point2::one() * amount * ZOOM_AMOUNT;

        // Don't let us zoom too far out
        if self.zoom < Point2::one() {
            self.zoom = Point2::one();
        }
    }

    // Resets the camera to the default center and zoom positions
    pub fn reset(&mut self) {
        self.pos = Point2::zero();
        self.zoom = Point2::one();
    }
}
