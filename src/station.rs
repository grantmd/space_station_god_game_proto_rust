use ggez::graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder};
use ggez::{graphics, Context, GameResult};

use std::collections::HashMap;

type Point2 = glam::Vec2;

const FLOOR_COLOR: Color = Color::new(0.1, 0.1, 0.1, 1.0);
const WALL_COLOR: Color = Color::new(0.3, 0.3, 0.3, 1.0);

// A Tile object, which the Station is made of
#[derive(Debug)]
pub struct Tile {
    pos: Point2,        // x,y position of the tile within the station
    pub kind: TileType, // what type of square the tile is
}
#[derive(Debug)]
pub enum TileType {
    Floor,
    Wall(WallDirection),
    Door(WallDirection),
}

// Walls have lots of different possible directions, which indicate how they are drawn
#[derive(Debug)]
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
pub struct Station {
    pub pos: Point2, // The position of the station (upper-left, basically), in world coordinates
    tiles: HashMap<(i32, i32), Tile>, // All the Tiles that make up the station
    mesh: Option<Mesh>,
}

impl Station {
    // Creates a new station from scratch.
    // Will eventually be randomly-generated
    pub fn new(ctx: &mut Context, pos: Point2, width: u32, height: u32) -> Station {
        let mut s = Station {
            pos: pos,
            tiles: HashMap::new(),
            mesh: None,
        };

        s.generate(width, height);
        s.build_mesh(ctx).unwrap();

        s
    }

    fn generate(&mut self, width: u32, height: u32) {
        for x in 0..width {
            for y in 0..height {
                // Figure out what type of tile
                let mut tile_type = TileType::Floor;
                if x == 0 && y == 0 {
                    tile_type = TileType::Wall(WallDirection::ExteriorCornerTopLeft);
                } else if x == width - 1 && y == 0 {
                    tile_type = TileType::Wall(WallDirection::ExteriorCornerTopRight);
                } else if x == 0 && y == height - 1 {
                    tile_type = TileType::Wall(WallDirection::ExteriorCornerBottomLeft);
                } else if x == width - 1 && y == height - 1 {
                    tile_type = TileType::Wall(WallDirection::ExteriorCornerBottomRight);
                } else if x == 0 && y != 0 {
                    tile_type = TileType::Wall(WallDirection::ExteriorLeft);
                } else if x != 0 && y == 0 {
                    tile_type = TileType::Wall(WallDirection::ExteriorTop);
                } else if x == width - 1 && y != height - 1 {
                    tile_type = TileType::Wall(WallDirection::ExteriorRight);
                } else if x != width - 1 && y == height - 1 {
                    tile_type = TileType::Wall(WallDirection::ExteriorBottom);
                }

                // Place the tile
                let tile = Tile::new(Point2::new(x as f32, y as f32), tile_type);
                self.add_tile(tile);
            }
        }
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
                    mb.rectangle(DrawMode::fill(), tile_rect, FLOOR_COLOR)?;
                    // Draw a line around it to make it a tile
                    mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            center.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            center.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            tile_rect.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            center.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;
                        let wall_rect2 = graphics::Rect::new(
                            center.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect2, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            center.y,
                            crate::TILE_WIDTH,
                            crate::TILE_WIDTH / 2.0,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
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
                            center.x,
                            tile_rect.y,
                            crate::TILE_WIDTH / 2.0,
                            crate::TILE_WIDTH,
                        );
                        mb.rectangle(DrawMode::fill(), wall_rect, WALL_COLOR)?;

                        // Draw a line around it to make it a tile
                        mb.rectangle(DrawMode::stroke(1.0), tile_rect, WALL_COLOR)?
                    }
                    _ => mb.rectangle(DrawMode::fill(), tile_rect, WALL_COLOR)?,
                },
                TileType::Door(_) => mb.rectangle(DrawMode::fill(), tile_rect, Color::WHITE)?,
            };
        }

        self.mesh = mb.build(ctx).ok();

        Ok(())
    }
}
