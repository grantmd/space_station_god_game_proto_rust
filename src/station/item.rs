use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, Context, GameResult};

use core::fmt::Debug;

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

// An item is the base of objects that live inside the station on tiles and inhabitants can interact
pub trait Item {
    fn get_name(&self) -> String;
    fn draw(&mut self, ctx: &mut Context, pos: Point2, camera: &crate::Camera) -> GameResult<()>;
    fn update(&mut self, ctx: &mut Context) -> GameResult<()>;
}

impl Debug for dyn Item {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Item: {}", self.get_name())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Food {
    pub energy: i8,
    pos: super::GridPosition,
}

impl Item for Food {
    fn get_name(&self) -> String {
        format!("Yummy yummy food. Restores {} energy", self.energy)
    }

    fn draw(
        &mut self,
        ctx: &mut Context,
        station_pos: Point2,
        camera: &crate::Camera,
    ) -> GameResult<()> {
        let pos = Point2::new(
            (crate::TILE_WIDTH * self.pos.x as f32) - (crate::TILE_WIDTH / 2.0),
            (crate::TILE_WIDTH * self.pos.y as f32) - (crate::TILE_WIDTH / 2.0),
        );
        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            pos,
            crate::TILE_WIDTH / 2.0 - 10.0,
            0.1,
            Color::new(1.0, 1.0, 0.0, 1.0),
        )?;
        graphics::draw(
            ctx,
            &mesh,
            DrawParam::default()
                .dest(station_pos)
                .offset(camera.pos)
                .scale(camera.zoom),
        )
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}
