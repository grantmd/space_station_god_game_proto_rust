use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::fmt;

// A position on a grid
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
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
