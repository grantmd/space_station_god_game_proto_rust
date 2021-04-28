use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, Context, GameError, GameResult};

use core::fmt::Debug;

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

// An item is the base of objects that live inside the station on tiles and inhabitants can interact
pub trait Item {
    fn get_name(&self) -> String;
    fn draw(&self, ctx: &mut Context, pos: Point2, camera: &crate::Camera) -> GameResult<()>;
    fn update(&mut self, ctx: &mut Context) -> GameResult<()>;
}

impl Debug for dyn Item {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Item: {}", self.get_name())
    }
}

#[derive(Debug)]
pub struct Food {
    pub energy: u8,
    pos: super::GridPosition,
}

impl Item for Food {
    fn get_name(&self) -> String {
        format!("Yummy yummy food. Restores {} energy", self.energy)
    }

    fn draw(
        &self,
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

impl Food {
    pub fn new(pos: super::GridPosition) -> Food {
        Food {
            pos: pos,
            energy: 10,
        }
    }
}

#[derive(Debug)]
pub struct Fridge {
    items: Vec<Food>,
    capacity: usize,
    pos: super::GridPosition,
}

impl Item for Fridge {
    fn get_name(&self) -> String {
        format!("Food storage. Has {} items.", self.items.len())
    }

    fn draw(
        &self,
        ctx: &mut Context,
        station_pos: Point2,
        camera: &crate::Camera,
    ) -> GameResult<()> {
        let pos = Point2::new(
            (crate::TILE_WIDTH * self.pos.x as f32) - (crate::TILE_WIDTH / 2.0),
            (crate::TILE_WIDTH * self.pos.y as f32) - (crate::TILE_WIDTH / 2.0),
        );
        let mesh = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            graphics::Rect::new(pos.x + 10.0, pos.y + 10.0, 10.0, 10.0),
            Color::new(0.5, 0.5, 0.5, 1.0),
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

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update all the contents
        for item in self.items.iter_mut() {
            item.update(ctx)?;
        }

        Ok(())
    }
}

impl Fridge {
    pub fn new(pos: super::GridPosition) -> Fridge {
        let mut fridge = Fridge {
            pos: pos,
            capacity: 10,
            items: Vec::with_capacity(10),
        };

        fridge.add_item(Food::new(pos)).unwrap();

        fridge
    }

    pub fn add_item(&mut self, item: Food) -> GameResult<()> {
        if self.items.len() >= self.capacity {
            return Err(GameError::CustomError(format!("Fridge is at capacity")));
        }

        self.items.push(item);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Food, Fridge};
    use crate::station::GridPosition;

    #[test]
    fn new_fridge_contains_items() {
        let fridge = Fridge::new(GridPosition::new(1, 1));
        assert_eq!(1, fridge.items.len());
    }

    #[test]
    fn fridge_add_item() {
        let mut fridge = Fridge::new(GridPosition::new(1, 1));
        assert!(fridge.add_item(Food::new(fridge.pos)).is_ok());
        assert_eq!(2, fridge.items.len());
    }

    #[test]
    fn fridge_max_items() {
        let mut fridge = Fridge::new(GridPosition::new(1, 1));
        while fridge.items.len() < fridge.capacity {
            assert!(fridge.add_item(Food::new(fridge.pos)).is_ok());
        }

        let result = fridge.add_item(Food::new(fridge.pos));
        assert!(result.is_err());
    }
}
