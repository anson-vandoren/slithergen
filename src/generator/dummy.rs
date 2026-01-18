use crate::args::Difficulty;
use crate::model::{Cell, Coord, Map, Region};

use super::Generator;

pub struct DummyGenerator;

impl Generator for DummyGenerator {
    fn generate(&self, radius: u8, _difficulty: Difficulty) -> Map {
        let mut map = Map::new(radius);
        // Fill with dummy data for testing viewer/io
        let coords: Vec<Coord> = map.iter_coords().collect();
        for coord in coords {
            map.cells.insert(coord, Cell::new(Region::Inside, 3, true));
        }
        map
    }
}
