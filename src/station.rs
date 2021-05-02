use crate::item::*;

use ggez::graphics::{Color, DrawMode, DrawParam, Mesh, MeshBuilder};
use ggez::{graphics, Context, GameResult};

use oorandom::Rand32;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
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

    // Manhattan distance on a square grid
    pub fn distance(&self, other: GridPosition) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl Ord for GridPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        other.x.cmp(&self.x).then_with(|| self.y.cmp(&other.y))
    }
}

impl PartialOrd for GridPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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

// A struct used to construct pathfinding movements
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Movement {
    cost: usize,
    pos: GridPosition,
}

// Compare movements by lowest-cost first, then positions as tie-breakers
impl Ord for Movement {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Movement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    pub fn new(pos: GridPosition, kind: TileType) -> Tile {
        Tile {
            pos,
            kind,
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
            pos,
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
                    } else if neighbor_count == 3 {
                        let tile = Tile::new(pos, TileType::Floor);
                        self.add_tile(tile);
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
                            // Decide on the type of wall and add it
                            to_place.insert(
                                neighbor_pos,
                                TileType::Wall(self.get_wall_direction(*pos)),
                            );
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
    fn get_wall_direction(&self, pos: GridPosition) -> WallDirection {
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
        direction
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

    // Removes a tile
    pub fn remove_tile(&mut self, pos: GridPosition) {
        self.tiles.remove(&pos);
    }

    // Get a random tile within the station
    pub fn get_random_tile(&self, kind: TileType, rng: &mut Rand32) -> Option<&Tile> {
        let mut options = Vec::with_capacity(self.num_tiles());
        for tile in self.tiles.values() {
            if tile.kind == kind {
                options.push(tile);
            }
        }

        if options.is_empty() {
            return None;
        }

        let index = rng.rand_range(0..options.len() as u32) as usize;
        Some(options[index])
    }

    // Get a tile at a screen position, if any
    // TODO: position should be a Point2 once ggez updates it
    pub fn get_tile_from_screen(&self, pos: Point2, camera: &crate::Camera) -> Option<&Tile> {
        // This is just world coordinates with camera translation
        self.get_tile_from_world(pos)
    }

    pub fn get_tile_from_world(&self, pos: Point2) -> Option<&Tile> {
        // Translate the world position into a grid position
        let screen_pos = pos - (Point2::one() * crate::TILE_WIDTH / 2.0); // Move up and to the left by half a tile
        let mut translated = (screen_pos / crate::TILE_WIDTH) - (self.pos / crate::TILE_WIDTH); // Move from world to grid by dividing by tile width
        translated = translated.ceil(); // Snap to grid
        let grid_pos = GridPosition::new(translated.x as i32, translated.y as i32); // Convert types

        // Return the tile, if any
        self.get_tile(grid_pos)
    }

    // Get the neighbors of a tile, ignoring diagonal directions, because we don't move that way
    pub fn get_neighbors(&self, pos: GridPosition) -> HashMap<(i32, i32), &Tile> {
        let mut neighbors = HashMap::with_capacity(4);

        let x = pos.x;
        let y = pos.y;

        // E W N S
        let mut dirs = vec![
            GridPosition::new(x + 1, y),
            GridPosition::new(x - 1, y),
            GridPosition::new(x, y - 1),
            GridPosition::new(x, y + 1),
        ];
        // see "Ugly paths" section for an explanation: https://www.redblobgames.com/pathfinding/a-star/implementation.html#troubleshooting-ugly-path
        if (x + y) % 2 == 0 {
            dirs.reverse(); // S N W E
        }

        for dir in dirs {
            // Check if there is a tile there, and add it if so
            if let Some(tile) = self.get_tile(dir) {
                neighbors.insert((dir.x, dir.y), tile);
            }
        }

        neighbors
    }

    // From a tile in the station, generate a list of reachable non-wall tile positions to the target
    // Keys are reached tile positions, values are where we came from to get there
    // Costs are taken into account and in the future could route around tough doors or whatever
    // This is A*
    fn search(
        &self,
        start: GridPosition,
        target: GridPosition,
    ) -> HashMap<GridPosition, Option<GridPosition>> {
        let mut frontier = BinaryHeap::new();
        frontier.push(Movement {
            cost: 0,
            pos: start,
        });

        let mut came_from = HashMap::new();
        came_from.insert(start, None);
        let mut cost_so_far = HashMap::new();
        cost_so_far.insert(start, 0);

        while !frontier.is_empty() {
            let current = frontier.pop().unwrap();

            if current.pos == target {
                break;
            }

            for (_pos, next) in self.get_neighbors(current.pos) {
                let new_cost = cost_so_far.get(&current.pos).unwrap_or(&0)
                    + self.movement_cost(&current.pos, next);
                if new_cost < *cost_so_far.get(&next.pos).unwrap_or(&usize::MAX) {
                    cost_so_far.insert(next.pos, new_cost);
                    match next.kind {
                        TileType::Wall(_) => {}
                        _ => {
                            frontier.push(Movement {
                                cost: new_cost + self.movement_heuristic(next.pos, target) as usize,
                                pos: next.pos,
                            });
                            came_from.insert(next.pos, Some(current.pos));
                        }
                    }
                }
            }
        }

        came_from
    }

    // Compute the cost of moving from a position to a tile. Lower is better
    // I'd like to use floating point values here, but that's problematic for
    // sorting in the binary heap. So instead we'll just multiply everything by 1,000
    fn movement_cost(&self, current: &GridPosition, next: &Tile) -> usize {
        // TODO: Locked doors
        // Cost is distance between the grid positions
        (current.distance(next.pos) * 1000) as usize
    }

    // Calculate the heuristic value between two grid positions, to be used for pathfinding
    fn movement_heuristic(&self, a: GridPosition, b: GridPosition) -> u32 {
        ((a.x - b.x).abs() + (a.y - b.y).abs()) as u32
    }

    // Given a start and an end, generate a path that doesn't include walls
    // TODO: This infinite loops if there's no path
    // TODO: This needs to be able to path outside of the station somehow
    pub fn path_to(&self, start: GridPosition, target: GridPosition) -> Vec<GridPosition> {
        // Start at the end and work backwards
        let mut current = target;
        let mut path = Vec::new();

        let reachable = self.search(start, target);
        let mut count = 0;
        while current != start {
            path.push(current);
            if let Some(next) = reachable.get(&current).unwrap_or(&None) {
                current = *next;
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

#[cfg(test)]
mod tests {
    use super::{GridPosition, Point2, Station, Tile, TileType, WallDirection};
    use oorandom::Rand32;
    use std::collections::HashMap;

    // Function to make an empty station, used in tests
    fn test_station() -> Station {
        Station {
            pos: Point2::new(1.0, 1.0),
            tiles: HashMap::new(),
            mesh: None,
        }
    }

    // Function to make a 4x4 floor station surrounded by walls, used in tests
    fn test_station_full() -> Station {
        let mut s = test_station();

        // Make top wall
        s.add_tile(Tile::new(
            GridPosition::new(0, 0),
            TileType::Wall(WallDirection::ExteriorCornerTopLeft),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(1, 0),
            TileType::Wall(WallDirection::ExteriorTop),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(2, 0),
            TileType::Wall(WallDirection::ExteriorTop),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(3, 0),
            TileType::Wall(WallDirection::ExteriorCornerTopRight),
        ));

        // Make bottom wall
        s.add_tile(Tile::new(
            GridPosition::new(0, 3),
            TileType::Wall(WallDirection::ExteriorCornerBottomLeft),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(1, 3),
            TileType::Wall(WallDirection::ExteriorBottom),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(2, 3),
            TileType::Wall(WallDirection::ExteriorBottom),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(3, 3),
            TileType::Wall(WallDirection::ExteriorCornerBottomRight),
        ));

        // Left wall
        s.add_tile(Tile::new(
            GridPosition::new(0, 1),
            TileType::Wall(WallDirection::ExteriorLeft),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(0, 2),
            TileType::Wall(WallDirection::ExteriorLeft),
        ));

        // Right wall
        s.add_tile(Tile::new(
            GridPosition::new(3, 1),
            TileType::Wall(WallDirection::ExteriorRight),
        ));
        s.add_tile(Tile::new(
            GridPosition::new(3, 2),
            TileType::Wall(WallDirection::ExteriorRight),
        ));

        // Floors
        s.add_tile(Tile::new(GridPosition::new(1, 1), TileType::Floor));
        s.add_tile(Tile::new(GridPosition::new(1, 2), TileType::Floor));
        s.add_tile(Tile::new(GridPosition::new(2, 1), TileType::Floor));
        s.add_tile(Tile::new(GridPosition::new(2, 2), TileType::Floor));

        s
    }

    #[test]
    fn tile_num_tiles() {
        let s = test_station();
        assert_eq!(0, s.num_tiles());
    }

    #[test]
    fn add_tile() {
        let mut s = test_station();
        let tile = Tile::new(GridPosition::new(1, 1), TileType::Floor);
        s.add_tile(tile);
        assert_eq!(1, s.num_tiles());
    }

    #[test]
    fn has_tile() {
        let mut s = test_station();
        let pos = GridPosition::new(1, 1);
        let tile = Tile::new(pos, TileType::Floor);
        s.add_tile(tile);
        assert!(s.has_tile(pos));
    }

    #[test]
    fn get_tile() {
        let mut s = test_station();
        let pos = GridPosition::new(1, 1);
        let tile = Tile::new(pos, TileType::Floor);
        s.add_tile(tile);

        let tile2 = s.get_tile(pos).unwrap();
        assert_eq!(pos, tile2.pos);
        assert_eq!(TileType::Floor, tile2.kind);
    }

    #[test]
    fn get_tile_empty() {
        let mut s = test_station();
        let pos = GridPosition::new(1, 1);
        let tile = Tile::new(pos, TileType::Floor);
        s.add_tile(tile);

        let tile2 = s.get_tile(GridPosition::new(1, 2));
        assert_eq!(None, tile2);
    }

    #[test]
    fn remove_tile() {
        let mut s = test_station();
        let pos = GridPosition::new(1, 1);
        let tile = Tile::new(pos, TileType::Floor);
        s.add_tile(tile);
        assert!(s.has_tile(pos));
        s.remove_tile(pos);
        assert!(!s.has_tile(pos));
    }

    #[test]
    fn get_random_tile() {
        let s = test_station_full();

        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));

        let floor_tile = s.get_random_tile(TileType::Floor, &mut rng).unwrap();
        assert_eq!(floor_tile.kind, TileType::Floor, "Finds a floor tile");

        let wall_tile1 = s
            .get_random_tile(TileType::Wall(WallDirection::ExteriorTop), &mut rng)
            .unwrap();
        assert_eq!(
            wall_tile1.kind,
            TileType::Wall(WallDirection::ExteriorTop),
            "Finds an exterior top wall tile"
        );

        let wall_tile2 = s
            .get_random_tile(
                TileType::Wall(WallDirection::ExteriorCornerBottomRight),
                &mut rng,
            )
            .unwrap();
        assert_eq!(
            wall_tile2.kind,
            TileType::Wall(WallDirection::ExteriorCornerBottomRight),
            "Finds the bottom-right exterior wall"
        );

        let door_tile =
            s.get_random_tile(TileType::Door(WallDirection::InteriorVertical), &mut rng);
        assert_eq!(door_tile, None, "Does not find any doors");
    }

    #[test]
    fn get_neighbors() {
        let s = test_station_full();

        // Test neighbors of the top-left corner
        let neighbors1 = s.get_neighbors(GridPosition::new(0, 0));
        assert_eq!(neighbors1.len(), 2, "Has two neighbors");
        assert_eq!(
            neighbors1[&(0, 1)].kind,
            TileType::Wall(WallDirection::ExteriorLeft),
            "Neighbor below is an exterior left wall"
        );
        assert_eq!(
            neighbors1[&(1, 0)].kind,
            TileType::Wall(WallDirection::ExteriorTop),
            "Neighbor to the right is an exterior top wall"
        );

        // Test neighbors in the middle-ish
        let neighbors2 = s.get_neighbors(GridPosition::new(1, 1));
        assert_eq!(neighbors2.len(), 4, "Has four neighbors");
        assert_eq!(
            neighbors2[&(1, 0)].kind,
            TileType::Wall(WallDirection::ExteriorTop),
            "Upper neighbor is an exterior top wall"
        );
        assert_eq!(
            neighbors2[&(2, 1)].kind,
            TileType::Floor,
            "Right neighbor is a floor"
        );
        assert_eq!(
            neighbors2[&(1, 2)].kind,
            TileType::Floor,
            "Below neighbor is a floor"
        );
        assert_eq!(
            neighbors2[&(0, 1)].kind,
            TileType::Wall(WallDirection::ExteriorLeft),
            "Left neighbor is an exterior left wall"
        );
    }
    #[test]
    fn search() {
        let s = test_station_full();
        let start = GridPosition::new(1, 1);
        let target = GridPosition::new(2, 2);
        let search = s.search(start, target);

        assert_eq!(search.len(), 4, "We can reach 3 tiles plus ourselves");
        assert_ne!(
            search[&target],
            Some(start),
            "Source of target tile is not start (because that would be a diagonal move)"
        );
    }

    #[test]
    fn path_to() {
        let mut s = test_station_full();
        let start = GridPosition::new(1, 1);
        let target = GridPosition::new(2, 2);

        let path = s.path_to(start, target);
        assert_eq!(path.len(), 2, "Can path to the target in 2 moves");

        s.add_tile(Tile::new(target, TileType::Wall(WallDirection::Full)));
        let path = s.path_to(start, target);
        assert_eq!(path.len(), 0, "Cannnot path to a wall");
    }
}
