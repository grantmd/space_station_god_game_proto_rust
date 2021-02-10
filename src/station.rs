use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{Color, DrawParam, Text};
use ggez::{conf, graphics, timer, Context, ContextBuilder, GameResult};

use std::collections::HashMap;

type Point2 = glam::Vec2;

// A Tile object, which the Station is made of
#[derive(Debug)]
pub struct Tile {
    pos: Point2,        // x,y position of the tile within the station
    pub kind: TileType, // what type of square the tile is
}
#[derive(Debug)]
pub enum TileType {
    Floor,
    Wall,
    Door,
}

impl Tile {
    fn new(pos: Point2, kind: TileType) -> Tile {
        Tile {
            pos: pos,
            kind: kind,
        }
    }
}

// A type for the Station itself
#[derive(Debug)]
pub struct Station {
    pub pos: Point2, // The position of the station (upper-left, basically), in world coordinates
    tiles: HashMap<(i32, i32), Tile>, // All the Tiles that make up the station
}

impl Station {
    // Creates a new station from scratch.
    // Will eventually be randomly-generated
    pub fn new(pos: Point2, width: u32, height: u32) -> Station {
        let mut s = Station {
            pos: pos,
            tiles: HashMap::new(),
        };

        for x in 0..width {
            for y in 0..height {
                // Figure out what type of tile
                let mut tile_type = TileType::Floor;
                if x == 0 || y == 0 {
                    tile_type = TileType::Wall;
                }
                if x == width - 1 || y == height - 1 {
                    tile_type = TileType::Wall;
                }

                // Place the tile
                let tile = Tile::new(Point2::new(x as f32, y as f32), tile_type);
                s.add_tile(tile);
            }
        }

        s
    }

    // Adds a tile to the station. Trusts the tile's position
    pub fn add_tile(&mut self, tile: Tile) {
        self.tiles
            .insert((tile.pos.x as i32, tile.pos.y as i32), tile);
    }

    // How many tiles do we have?
    pub fn num_tiles(&self) -> usize {
        self.tiles.len()
    }

    // Do we have a tile at a spot?
    pub fn has_tile(&self, pos: (i32, i32)) -> bool {
        self.tiles.contains_key(&pos)
    }

    // Get tile at a spot, if any
    pub fn get_tile(&self, pos: (i32, i32)) -> Option<&Tile> {
        self.tiles.get(&pos)
    }

    // Removes a tile
    pub fn remove_tile(&mut self, pos: (i32, i32)) {
        self.tiles.remove(&pos);
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for (index, tile) in &self.tiles {
            let rect = graphics::Rect::new(
                self.pos.x + (crate::TILE_WIDTH * index.0 as f32) - (crate::TILE_WIDTH / 2.0),
                self.pos.y + (crate::TILE_WIDTH * index.1 as f32) - (crate::TILE_WIDTH / 2.0),
                crate::TILE_WIDTH,
                crate::TILE_WIDTH,
            );

            let mesh = match tile.kind {
                TileType::Floor => graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(1.0),
                    rect,
                    Color::new(0.3, 0.3, 0.3, 1.0),
                )?,
                TileType::Wall => graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    Color::new(0.3, 0.3, 0.3, 1.0),
                )?,
                TileType::Door => graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    Color::WHITE,
                )?,
            };
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }

        Ok(())
    }
}
