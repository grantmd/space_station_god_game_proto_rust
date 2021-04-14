use crate::item::*;
use crate::station::{Tile, TileType};

use ggez::graphics::{Color, DrawMode, DrawParam, Mesh};
use ggez::{graphics, Context, GameResult};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

// An Inhabitant of the Station
#[derive(Debug)]
pub struct Inhabitant {
    pub pos: Point2,
    pub dest: Option<Point2>,
    kind: InhabitantType,
    health: u8,
    hunger: u8,
    thirst: u8,
    age: u8,
    items: Vec<Box<dyn Item>>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InhabitantType {
    Pilot,
    Engineer,
    Medic,
    Soldier,
    Miner,
    Cook,
    Ghost,
}

impl Inhabitant {
    pub fn new(pos: Point2, kind: InhabitantType) -> Inhabitant {
        Inhabitant {
            pos: pos,
            dest: None,
            kind: kind,
            health: 100,
            hunger: 0,
            thirst: 0,
            age: 1,
            items: Vec::new(),
        }
    }

    // Whether we can move to a type of tile
    // Doesn't check whether we can _get_ there, but only if we can be there
    pub fn can_move_to(&mut self, tile: Option<&Tile>) -> bool {
        match self.kind {
            // Ghosts can go anywhere, lol
            InhabitantType::Ghost => true,

            // Everyone else needs to test the type of tile
            _ => match tile {
                Some(t) => match t.kind {
                    TileType::Wall(_) => false,
                    TileType::Door(_) => true, // TODO: Check if we can open it?
                    TileType::Floor => true,
                },
                None => false,
            },
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.hunger >= 100 {
            self.take_damage(1);
        }

        Ok(())
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        station_pos: Point2,
        camera: &crate::Camera,
    ) -> GameResult<()> {
        let pos = Point2::new(
            (crate::TILE_WIDTH * self.pos.x) - (crate::TILE_WIDTH / 2.0),
            (crate::TILE_WIDTH * self.pos.y) - (crate::TILE_WIDTH / 2.0),
        );
        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            pos,
            crate::TILE_WIDTH / 2.0 - 10.0,
            0.1,
            Color::WHITE,
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

    pub fn eat(&mut self, item: &Food) {
        self.hunger -= item.energy;
    }

    pub fn take_damage(&mut self, amount: u8) {
        self.health -= amount;
        if self.health <= 0 {
            self.die();
        }
    }

    pub fn die(&mut self) {
        // TODO: What if already a ghost!?
        self.kind = InhabitantType::Ghost;
    }
}
