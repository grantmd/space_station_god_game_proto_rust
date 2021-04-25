use crate::item::*;

use ggez::graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder};
use ggez::{graphics, Context, GameResult};

use oorandom::Rand32;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

type Point2 = glam::Vec2;

const FLOOR_COLOR: Color = Color::new(0.1, 0.1, 0.1, 1.0);
const WALL_COLOR: Color = Color::new(0.3, 0.3, 0.3, 1.0);
const BORDER_COLOR: Color = Color::BLACK;

// A position on a grid
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    // We make a standard helper function so that we can create a new `GridPosition` more easily.
    pub fn new(x: i32, y: i32) -> Self {
        GridPosition { x, y }
    }
}

// Convenient creation of a GridPosition from a tuple
impl From<(i32, i32)> for GridPosition {
    fn from(pos: (i32, i32)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

impl fmt::Display for GridPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// A Tile object, which the Station is made of
#[derive(Debug)]
pub struct Tile {
    pub pos: GridPosition,         // x,y position of the tile within the station
    pub kind: TileType,            // what type of square the tile is
    pub items: Vec<Box<dyn Item>>, // Items that are present on/in the tile
}

// Tiles are equal if they are in the same spot
impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for Tile {}

impl Hash for Tile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
    }
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
    fn new(pos: GridPosition, kind: TileType) -> Tile {
        Tile {
            pos: pos,
            kind: kind,
            items: Vec::new(),
        }
    }

    pub fn add_item<T: Item + 'static>(&mut self, item: T) {
        self.items.push(Box::new(item));
    }

    pub fn to_world_position(&self, station: &Station) -> Point2 {
        Point2::new(
            station.pos.x + (self.pos.x as f32 * crate::TILE_WIDTH),
            station.pos.y + (self.pos.y as f32 * crate::TILE_WIDTH),
        )
    }
}

// A type for the Station itself
pub struct Station {
    pub pos: Point2, // The position of the station (upper-left, basically), in world coordinates
    tiles: HashMap<GridPosition, Tile>, // All the Tiles that make up the station
    mesh: Option<Mesh>, // A cache of the mesh making up the station structure
}

impl Station {
    // Creates a new station from scratch.
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

    // Randomly generate a new station
    fn generate(&mut self, width: usize, height: usize, rng: &mut Rand32) {
        // Randomly place floor tiles to give us a base
        for x in 0..width as i32 {
            for y in 0..height as i32 {
                if rng.rand_float() < 0.70 {
                    let tile = Tile::new(GridPosition::new(x, y), TileType::Floor);
                    self.add_tile(tile);
                }
            }
        }

        // Loop over the floor tiles we placed and expand into bigger spaces
        // Do this a couple times
        for _ in 0..2 {
            for x in 0..width as i32 {
                for y in 0..height as i32 {
                    let pos = GridPosition::new(x, y);
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
        // This is done in two loops because I am not good at Rust and don't
        // know how to solve the memory access issues of modifying while looping
        let mut to_place = HashMap::new();
        for (pos, tile) in self.tiles.iter() {
            if tile.kind == TileType::Floor {
                for x in -1..2 {
                    for y in -1..2 {
                        // Don't consider ourselves
                        if x == 0 && y == 0 {
                            continue;
                        }

                        // If the neighbor doesn't have a floor, make it a wall
                        let neighbor_pos = GridPosition::new(pos.x + x, pos.y + y);
                        if !self.has_tile(neighbor_pos) {
                            // Decide on the type of wall
                            if let Some(wall_direction) = self.get_wall_direction(*pos) {
                                // Add it
                                to_place.insert(neighbor_pos, TileType::Wall(wall_direction));
                            }
                        }
                    }
                }
            }
        }

        for (&pos, &tile_type) in to_place.iter() {
            let new_tile = Tile::new(pos, tile_type);
            self.add_tile(new_tile);
        }

        // Place some items on the tiles
        for (_pos, tile) in self.tiles.iter_mut() {
            if tile.kind == TileType::Floor {
                println!("Placing fridge at {:#?}", tile);
                tile.add_item(Fridge::new(tile.pos));
                break;
            }
        }
    }

    // For a given position, get the best wall direction based on neighbors
    // Used for station generation
    fn get_wall_direction(&self, pos: GridPosition) -> Option<WallDirection> {
        let neighbors = self.get_neighbors(pos);

        let mut direction = WallDirection::Full;

        // Place the exterior corners first
        if !neighbors.contains_key(&(-1, 1)) && !neighbors.contains_key(&(0, 0)) {
            direction = WallDirection::ExteriorCornerTopLeft;
        } else if !neighbors.contains_key(&(1, 1)) && !neighbors.contains_key(&(0, 0)) {
            direction = WallDirection::ExteriorCornerTopRight;
        } else if !neighbors.contains_key(&(-1, 1)) && !neighbors.contains_key(&(0, 1)) {
            direction = WallDirection::ExteriorCornerBottomLeft;
        } else if !neighbors.contains_key(&(1, 1)) && !neighbors.contains_key(&(0, 1)) {
            direction = WallDirection::ExteriorCornerBottomRight;
        }
        // Exterior walls are fairly easy
        else if !neighbors.contains_key(&(-1, 1)) {
            direction = WallDirection::ExteriorLeft;
        } else if !neighbors.contains_key(&(1, 1)) {
            direction = WallDirection::ExteriorRight;
        } else if !neighbors.contains_key(&(0, 0)) {
            direction = WallDirection::ExteriorTop;
        } else if !neighbors.contains_key(&(0, 1)) {
            direction = WallDirection::ExteriorBottom;
        }

        direction = WallDirection::Full; // Temporary override while I figure this function out
        Some(direction)
    }

    // Adds a tile to the station. Trusts the tile's position
    pub fn add_tile(&mut self, tile: Tile) {
        self.tiles.insert(tile.pos, tile);
    }

    // How many tiles do we have?
    pub fn num_tiles(&self) -> usize {
        self.tiles.len()
    }

    // Do we have a tile at a grid position?
    pub fn has_tile(&self, pos: GridPosition) -> bool {
        self.tiles.contains_key(&pos)
    }

    // Get tile at a grid position, if any
    pub fn get_tile(&self, pos: GridPosition) -> Option<&Tile> {
        self.tiles.get(&pos)
    }

    // Get a random tile within the station
    pub fn get_random_tile(&self, kind: TileType, rng: &mut Rand32) -> Option<&Tile> {
        let mut options = Vec::with_capacity(self.num_tiles());
        for tile in self.tiles.values() {
            if tile.kind == kind {
                options.push(tile);
            }
        }

        if options.len() == 0 {
            return None;
        }

        let index = rng.rand_range(0..options.len() as u32) as usize;
        Some(options[index])
    }

    // Get a tile at a screen position, if any
    // TODO: position should be a Point2 once ggez updates it
    pub fn get_tile_from_screen(&self, pos: Point2, camera: &crate::Camera) -> Option<&Tile> {
        // This is just world coordinates with camera translation
        return self.get_tile_from_world(pos);
    }

    pub fn get_tile_from_world(&self, pos: Point2) -> Option<&Tile> {
        // Translate the screen position into a grid position
        let screen_pos = pos - (Point2::one() * crate::TILE_WIDTH / 2.0); // Move up and to the left by half a tile on screen
        let mut translated = (screen_pos / crate::TILE_WIDTH) - (self.pos / crate::TILE_WIDTH); // Move from screen to grid by dividing by tile width
        translated = translated.ceil(); // Snap to grid
        let grid_pos = GridPosition::new(translated.x as i32, translated.y as i32); // Convert types

        // Return the tile, if any
        self.get_tile(grid_pos)
    }

    // Get the neighbors of a tile
    pub fn get_neighbors(&self, pos: GridPosition) -> HashMap<(i32, i32), &Tile> {
        let mut neighbors = HashMap::with_capacity(8);

        for x in -1..2 {
            for y in -1..2 {
                // Don't consider ourselves
                if x == 0 && y == 0 {
                    continue;
                }

                // Check if there is a tile there, and add it if so
                if let Some(tile) = self.get_tile(GridPosition::new(pos.x + x, pos.y + y)) {
                    neighbors.insert((x, y), tile);
                }
            }
        }

        neighbors
    }

    // Removes a tile
    pub fn remove_tile(&mut self, pos: GridPosition) {
        self.tiles.remove(&pos);
    }

    // From a tile in the station, generate a list of reachable non-wall tiles via breadth-first search
    // Keys are reached tiles, values are where we came from to get there
    fn search<'a>(
        &'a self,
        start: &'a Tile,
        target: Option<&Tile>,
    ) -> HashMap<&'a Tile, Option<&Tile>> {
        let mut frontier = Vec::new();
        frontier.push(start);

        let mut came_from = HashMap::new();
        came_from.insert(start, None);

        while !frontier.is_empty() {
            let current = frontier.pop().unwrap();

            if Some(current) == target {
                break;
            }

            for (_pos, next) in self.get_neighbors(current.pos) {
                if !came_from.contains_key(next) {
                    match next.kind {
                        TileType::Wall(_) => {}
                        _ => {
                            // TODO: Locked doors
                            frontier.push(next);
                            came_from.insert(next, Some(current));
                        }
                    }
                }
            }
        }

        came_from
    }

    // Given a start and an end, generate a path that doesn't include walls
    // TODO: This infinite loops if there's no path
    pub fn path_to<'a>(&'a self, start: &Tile, target: &'a Tile) -> Vec<&Tile> {
        // Start at the end and work backwards
        let mut current = target;
        let mut path = Vec::new();

        let reachable = self.search(target, Some(start));
        let mut count = 0;
        while current != start {
            path.push(current);
            if let Some(next) = reachable[current] {
                current = next;
            }

            count += 1;
            if count > 10000 {
                // TODO: Return an error instead
                return Vec::new();
            }
        }

        path.reverse();
        path
    }

    // Update callback on the station
    pub fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update all items
        for (_pos, tile) in self.tiles.iter_mut() {
            for item in tile.items.iter_mut() {
                item.update(ctx)?;
            }
        }

        Ok(())
    }

    // Draw callback
    pub fn draw(&mut self, ctx: &mut Context, camera: &crate::Camera) -> GameResult<()> {
        // Draw the pre-calculated station mesh
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
        }?;

        // Draw items on tiles
        for (_pos, tile) in self.tiles.iter() {
            for item in tile.items.iter() {
                item.draw(ctx, self.pos, camera)?;
            }
        }

        Ok(())
    }

    // Create a mesh from our state
    fn build_mesh(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mb = &mut MeshBuilder::new();
        for (index, tile) in &self.tiles {
            let tile_rect = graphics::Rect::new(
                (crate::TILE_WIDTH * index.x as f32) - (crate::TILE_WIDTH / 2.0),
                (crate::TILE_WIDTH * index.y as f32) - (crate::TILE_WIDTH / 2.0),
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
