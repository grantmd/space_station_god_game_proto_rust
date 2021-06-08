use super::gridposition::*;

use std::cmp::Ordering;

// A struct used to construct pathfinding movements
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Movement {
    pub cost: usize,
    pub pos: GridPosition,
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
