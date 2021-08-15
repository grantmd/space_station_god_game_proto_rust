use super::gridposition::*;
use super::station::*;
use crate::item::*;

use serde::{Deserialize, Serialize};

use std::hash::{Hash, Hasher};

type Point2 = glam::Vec2;

// A Tile object, which the Station is made of
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile {
    pub pos: GridPosition, // x,y position of the tile within the station
    pub kind: TileType,    // what type of square the tile is
    pub items: Vec<Item>,  // Items that are present on/in the tile
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

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Wall(WallDirection),
    Door(WallDirection),
}

// Walls have lots of different possible directions, which indicate how they are drawn
// Directions like "top-left" indicate that in a square walled room, this is the top-left corner
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
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

    // Add an item to the tile
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    // Do we have an item of this type on us?
    pub fn has_item(&self, item_types: Vec<ItemType>) -> bool {
        if self.get_item(item_types).is_some() {
            return true;
        }

        false
    }

    pub fn get_item(&self, item_types: Vec<ItemType>) -> Option<&Item> {
        for item in self.items.iter() {
            if item_types.contains(&item.get_type()) {
                return Some(item);
            }

            // If this is a container, we need to iterate inside
            if let ItemType::Container(_) = item.get_type() {
                for subitem in item.get_items().iter() {
                    // Is this what we're looking for?
                    if item_types.contains(&subitem.get_type()) {
                        return Some(item);
                    }
                }
            }
        }

        None
    }

    // Given an item uuid, removes it from the tile
    pub fn remove_item(&mut self, id: uuid::Uuid) {
        self.items.retain(|item| item.get_id() != id)
    }

    // Convert a tile's grid position to a "world" position, based on where the station is
    pub fn to_world_position(&self, station: &Station) -> Point2 {
        Point2::new(
            station.pos.x + (self.pos.x as f32 * crate::TILE_WIDTH),
            station.pos.y + (self.pos.y as f32 * crate::TILE_WIDTH),
        )
    }
}
