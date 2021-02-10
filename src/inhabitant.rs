use crate::station::{Tile, TileType};

use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{Color, DrawParam, Text};
use ggez::{conf, graphics, timer, Context, ContextBuilder, GameResult};

use keyframe::{ease, functions::EaseInOut};

// Alias some types to making reading/writing code easier and also in case math libraries change again
type Point2 = glam::Vec2;

// An Inhabitant of the Station
#[derive(Debug)]
pub struct Inhabitant {
    pub pos: Point2,
    pub dest: Option<Point2>,
    kind: InhabitantType,
    health: i8,
    hunger: i8,
    thirst: i8,
}

#[derive(Debug)]
pub enum InhabitantType {
    Pilot,
    Engineer,
    Medic,
    Soldier,
    Miner,
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
                    TileType::Wall => false,
                    TileType::Door => true, // TODO: Check if we can open it?
                    TileType::Floor => true,
                },
                None => false,
            },
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, station_pos: Point2) -> GameResult<()> {
        let pos = Point2::new(
            station_pos.x + (crate::TILE_WIDTH * self.pos.x) - (crate::TILE_WIDTH / 2.0),
            station_pos.y + (crate::TILE_WIDTH * self.pos.y) - (crate::TILE_WIDTH / 2.0),
        );
        let mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            pos,
            crate::TILE_WIDTH / 2.0 - 5.0,
            0.1,
            Color::WHITE,
        )?;
        graphics::draw(ctx, &mesh, DrawParam::default())
    }
}
