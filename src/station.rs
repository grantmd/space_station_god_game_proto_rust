use ggez::graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder};
use ggez::{graphics, Context, GameResult};

use oorandom::Rand32;
use std::collections::HashMap;

type Point2 = glam::Vec2;

const FLOOR_COLOR: Color = Color::new(0.1, 0.1, 0.1, 1.0);
const WALL_COLOR: Color = Color::new(0.3, 0.3, 0.3, 1.0);
const BORDER_COLOR: Color = Color::BLACK;

// A Tile object, which the Station is made of
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tile {
    pos: (i32, i32),    // x,y position of the tile within the station
    pub kind: TileType, // what type of square the tile is
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TileType {
    Floor,
    Wall(WallDirection),
    Door(WallDirection),
}

// Walls have lots of different possible directions, which indicate how they are drawn
// Directions like "top-left" indicate that in a square walled room, this is the top-left corner
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum WallDirection {
    InteriorVertical,
    InteriorHorizontal,
    InteriorCross,
    InteriorCornerTopLeft,
    InteriorCornerTopRight,
    InteriorCornerBottomLeft,
    InteriorCornerBottomRight,
    ExteriorTop,
    ExteriorBottom,
    ExteriorLeft,
    ExteriorRight,
    ExteriorCornerTopLeft,
    ExteriorCornerTopRight,
    ExteriorCornerBottomLeft,
    ExteriorCornerBottomRight,
    Full,
}

impl Tile {
    fn new(pos: (i32, i32), kind: TileType) -> Tile {
        Tile {
            pos: pos,
            kind: kind,
        }
    }
}

// A type for the Station itself
pub struct Station {
    pub pos: Point2, // The position of the station (upper-left, basically), in world coordinates
    tiles: HashMap<(i32, i32), Tile>, // All the Tiles that make up the station
    mesh: Option<Mesh>,
}

impl Station {
    // Creates a new station from scratch.
    // Will eventually be randomly-generated
    pub fn new(
        ctx: &mut Context,
        pos: Point2,
        width: usize,
        height: usize,
        rng: &mut Rand32,
    ) -> Station {
        let mut s = Station {
            pos: pos,
            tiles: HashMap::with_capacity(width * height),
            mesh: None,
        };

        s.generate(width, height, rng);
        s.build_mesh(ctx).unwrap();

        s
    }

    fn generate(&mut self, width: usize, height: usize, rng: &mut Rand32) {
        // Randomly place floor tiles to give us a base
        for x in 0..width as i32 {
            for y in 0..height as i32 {
                if rng.rand_float() < 0.45 {
                    let tile = Tile::new((x, y), TileType::Floor);
                    self.add_tile(tile);
                }
            }
        }

        // Loop over the floor tiles we placed and expand into bigger spaces
        // Do this a couple times
        for _ in 0..2 {
            for x in 0..width as i32 {
                for y in 0..height as i32 {
                    let pos = (x, y);
                    let neighbor_count = self.get_neighbors(pos).len();
                    if self.has_tile(pos) {
                        if neighbor_count < 2 {
                            self.remove_tile(pos);
                        }
                    } else {
                        if neighbor_count == 3 {
                            let tile = Tile::new(pos, TileType::Floor);
                            self.add_tile(tile);
                        }
                    }
                }
            }
        }

        // Loop over the floor tiles and place walls around the edges
        let tiles = self.tiles.clone();
        for (pos, tile) in tiles {
            if tile.kind == TileType::Floor {
                for x in -1..2 {
                    for y in -1..2 {
                        // Don't consider ourselves
                        if x == 0 && y == 0 {
                            continue;
                        }

                        // If the neighbor doesn't have a floor, make it a wall
                        if !self.has_tile((pos.0 + x, pos.1 + y)) {
                            // Decide on the type of wall
                            let wall_direction = WallDirection::Full;

                            // Add it
                            let tile =
                                Tile::new((pos.0 + x, pos.1 + y), TileType::Wall(wall_direction));
                            self.add_tile(tile);
                        }
                    }
                }
            }
        }
    }

    // Adds a tile to the station. Trusts the tile's position
    pub fn add_tile(&mut self, tile: Tile) {
        self.tiles.insert((tile.pos.0, tile.pos.1), tile);
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

    // Get the neighbors of a tile
    pub fn get_neighbors(&self, pos: (i32, i32)) -> Vec<&Tile> {
        let mut neighbors = Vec::new();

        for x in -1..2 {
            for y in -1..2 {
                // Don't consider ourselves
                if x == 0 && y == 0 {
                    continue;
                }

                // Check if there is a tile there, and add it if so
                if let Some(tile) = self.get_tile((pos.0 + x, pos.1 + y)) {
                    neighbors.push(tile);
                }
            }
        }

        neighbors
    }

    // Removes a tile
    pub fn remove_tile(&mut self, pos: (i32, i32)) {
        self.tiles.remove(&pos);
    }

    pub fn draw(&mut self, ctx: &mut Context, camera: &crate::Camera) -> GameResult<()> {
        match &self.mesh {
            Some(mesh) => graphics::draw(
                ctx,
                mesh,
                DrawParam::default()
                    .dest(self.pos)
                    .offset(camera.pos)
                    .scale(camera.zoom),
            ),
            None => Ok(()),
        }
    }

    fn build_mesh(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mb = &mut MeshBuilder::new();
        for (index, tile) in &self.tiles {
            let tile_rect = graphics::Rect::new(
                (crate::TILE_WIDTH * index.0 as f32) - (crate::TILE_WIDTH / 2.0),
                (crate::TILE_WIDTH * index.1 as f32) - (crate::TILE_WIDTH / 2.0),
                crate::TILE_WIDTH,
                crate::TILE_WIDTH,
            );
            let center = tile_rect.center();

            match &tile.kind {
                TileType::Floor => {
                    // Fill the floor
                    mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?
                }
                TileType::Wall(wall_direction) => match wall_direction {
                    WallDirection::ExteriorCornerTopLeft => {
                        // Fill the bottom-right quarter of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            center.x,
                            center.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections on the outside of the fill. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::ExteriorCornerTopRight => {
                        // Fill the bottom-left quarter of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections on the outside of the fill. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            center.x + 1.0,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::ExteriorCornerBottomLeft => {
                        // Fill the top-right quarter of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            center.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections on the outside of the fill. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y + 1.0,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::ExteriorCornerBottomRight => {
                        // Fill the top-left quarter of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections on the outside of the fill. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y + 1.0,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            center.x + 1.0,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::ExteriorTop => {
                        // Fill the bottom-half of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw the top-half of the tile as wall
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?
                    }
                    WallDirection::ExteriorBottom => {
                        // Fill the top-half of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw the bottom-half of the tile as wall
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y + 1.0,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?
                    }
                    WallDirection::ExteriorLeft => {
                        // Fill the right-half of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            center.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw the left-half of the tile as wall
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?
                    }
                    WallDirection::ExteriorRight => {
                        // Fill the left-half of the tile as a floor
                        let floor_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), floor_rect, FLOOR_COLOR)?;

                        // Draw the right-half of the tile as wall
                        let wall_rect = graphics::Rect::new(
                            center.x + 1.0,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?
                    }
                    WallDirection::InteriorVertical => {
                        // Fill the floor
                        mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;

                        // Create a vertical wall centered to the tile
                        let wall_rect = graphics::Rect::new(
                            center.x - crate::TILE_WIDTH / 4.0,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?
                    }
                    WallDirection::InteriorHorizontal => {
                        // Fill the floor
                        mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;

                        // Create a horizontal wall centered to the tile
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y - crate::TILE_WIDTH / 4.0,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?
                    }
                    WallDirection::InteriorCross => {
                        // Fill the floor
                        mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;

                        // Create a horizontal wall centered to the tile
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y - crate::TILE_WIDTH / 4.0,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;

                        // Create a vertical wall centered to the tile
                        let wall_rect = graphics::Rect::new(
                            center.x - crate::TILE_WIDTH / 4.0,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?
                    }
                    WallDirection::InteriorCornerTopLeft => {
                        // Fill the floor
                        mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::InteriorCornerTopRight => {
                        // Fill the floor
                        mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            center.x + 1.0,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::InteriorCornerBottomLeft => {
                        // Fill the floor
                        mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y + 1.0,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::InteriorCornerBottomRight => {
                        // Fill the floor
                        mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;

                        // Draw two "wall" sections. One vertical, one horizontal.
                        let wall_rect = graphics::Rect::new(
                            tile_rect.x,
                            center.y + 1.0,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            center.x + 1.0,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0 - 1.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?
                    }
                    WallDirection::Full => mb.rectangle(DrawMode::fill(), tile_rect, WALL_COLOR)?,
                },
                TileType::Door(_) => mb.rectangle(DrawMode::fill(), tile_rect, Color::WHITE)?,
            };

            // Draw a line around it to make it a tile
            mb.rectangle(DrawMode::stroke(1.0), tile_rect, BORDER_COLOR)?;
        }

        self.mesh = mb.build(ctx).ok();

        Ok(())
    }
}
