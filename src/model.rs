use serde::Serialize;
use std::collections::HashMap;

/// Axial coordinates (q, r)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Coord {
    pub q: i8,
    pub r: i8,
}

impl Coord {
    pub fn new(q: i8, r: i8) -> Self {
        Self { q, r }
    }
}

/// Region type for a cell (Inside or Outside loop)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Region {
    Inside,
    Outside,
    // Unknown // Potentially useful for solving later, but spec says Map has ground truth
}

/// A single hexagonal cell on the grid
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cell {
    /// True region of the cell (Answer Key)
    pub region: Region,
    /// Number of edges around this cell that are part of the loop (0-6)
    pub full_neighbor_count: u8,
    /// Whether the number clue is shown to the player
    pub clue_visible: bool,
}

impl Cell {
    pub fn new(region: Region, count: u8, visible: bool) -> Self {
        Self {
            region,
            full_neighbor_count: count,
            clue_visible: visible,
        }
    }
}

/// The game map
#[derive(Debug, Clone, Serialize)]
pub struct Map {
    pub radius: u8,
    pub cells: HashMap<Coord, Cell>,
}

impl Map {
    pub fn new(radius: u8) -> Self {
        Self {
            radius,
            cells: HashMap::new(),
        }
    }

    /// Iterator over all coordinates in the map, following the spec order:
    /// Outer loop: q from -R to +R
    /// Inner loop: r from max(-R, -q-R) to min(R, -q+R)
    pub fn iter_coords(&self) -> impl Iterator<Item = Coord> {
        let r = self.radius as i8;
        (-r..=r).flat_map(move |q| {
            let r_min = (-r).max(-q - r);
            let r_max = r.min(-q + r);
            (r_min..=r_max).map(move |r| Coord::new(q, r))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[googletest::test]
    fn test_iter_coords_count() {
        // Formula: 3R(R+1) + 1
        // R=2 -> 3*2*3 + 1 = 19
        let map = Map::new(2);
        let count = map.iter_coords().count();
        expect_that!(count, eq(19));

        // R=0 -> 1
        let map = Map::new(0);
        let count = map.iter_coords().count();
        expect_that!(count, eq(1));
    }
}
