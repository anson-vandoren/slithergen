use crate::model::{Cell, Coord, Map, Region};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

/// Save a map to a binary file
/// Format: [Flags: u8] [Radius: u8] [HexBytes...]
/// Legacy Format: [Radius: u8] [HexBytes...]
pub fn save_map<P: AsRef<Path>>(map: &Map, path: P, legacy: bool) -> io::Result<()> {
    let mut file = File::create(path)?;

    if !legacy {
        // Byte 0: Flags (Reserved 0)
        file.write_all(&[0u8])?;
    }

    // Byte 1 (or 0 if legacy): Radius
    file.write_all(&[map.radius])?;

    // Hexagon Data
    // We must iterate in the specific order defined by iter_coords
    for coord in map.iter_coords() {
        let cell = map.cells.get(&coord).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Map incomplete or missing cells",
            )
        })?;

        let byte = pack_cell(cell);
        file.write_all(&[byte])?;
    }

    Ok(())
}

/// Load a map from a binary file
pub fn load_map<P: AsRef<Path>>(path: P) -> io::Result<Map> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.is_empty() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "File empty"));
    }

    // Auto-detect legacy format
    // Legacy: [Radius] [Data...]
    // Modern: [Flags] [Radius] [Data...]

    let candidate_legacy_radius = buffer[0];
    let hex_count_legacy =
        3 * (candidate_legacy_radius as u32) * (candidate_legacy_radius as u32 + 1) + 1;
    let expected_size_legacy = 1 + hex_count_legacy as usize;

    let (radius, start_offset) = if buffer.len() == expected_size_legacy {
        // Detected Legacy
        (candidate_legacy_radius, 1)
    } else {
        // Assume Modern
        if buffer.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "File too short",
            ));
        }
        let _flags = buffer[0];
        let radius = buffer[1];
        (radius, 2)
    };

    let mut map = Map::new(radius);

    // Expected hex count check for modern path (legacy implicitly checked by detection logic, but good to double check or simplify)
    let expected_hexes = 3 * (radius as u32) * (radius as u32 + 1) + 1;
    if buffer.len() - start_offset != expected_hexes as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File size does not match radius",
        ));
    }

    let mut iter = buffer.iter().skip(start_offset);

    // Reconstruct utilizing the determinstic iteration order
    // iter_coords is stateless based on radius, so we can use it to rebuild keys.
    let coords: Vec<Coord> = map.iter_coords().collect();

    for coord in coords {
        let byte = *iter
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Missing cell data"))?;
        let cell = unpack_cell(byte);
        map.cells.insert(coord, cell);
    }

    Ok(map)
}

fn pack_cell(cell: &Cell) -> u8 {
    let mut byte = 0u8;

    // Bit 0: Region (1=Inside, 0=Outside)
    if cell.region == Region::Inside {
        byte |= 0x1;
    }

    // Bits 1-3: Target Count
    byte |= (cell.full_neighbor_count & 0x7) << 1;

    // Bit 4: Show Number
    if cell.clue_visible {
        byte |= 0x10;
    }

    byte
}

fn unpack_cell(byte: u8) -> Cell {
    let region = if (byte & 0x1) != 0 {
        Region::Inside
    } else {
        Region::Outside
    };

    let count = (byte >> 1) & 0x7;
    let visible = (byte & 0x10) != 0;

    Cell::new(region, count, visible)
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use tempfile::tempdir;

    #[googletest::test]
    fn test_pack_unpack() {
        let c1 = Cell::new(Region::Inside, 3, true);
        let b1 = pack_cell(&c1);
        let u1 = unpack_cell(b1);
        expect_that!(u1, eq(&c1));

        // Manual bit check
        // Inside=1, Count=3 (011) << 1 = 6 (00110), Visible=1 << 4 = 16 (10000)
        // Total = 1 + 6 + 16 = 23 (0x17)
        expect_that!(b1, eq(23));

        let c2 = Cell::new(Region::Outside, 0, false);
        let b2 = pack_cell(&c2);
        let u2 = unpack_cell(b2);
        expect_that!(u2, eq(&c2));
        expect_that!(b2, eq(0));
    }

    #[googletest::test]
    fn test_save_load_roundtrip() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_map.bin");

        let mut map = Map::new(1); // Radius 1 = 7 hexes
        // Populate map cells manually for test
        let coords: Vec<Coord> = map.iter_coords().collect();
        for coord in coords {
            map.cells.insert(coord, Cell::new(Region::Inside, 1, true));
        }

        // Save
        save_map(&map, &path, false).unwrap();

        // Load
        let loaded = load_map(&path).unwrap();

        expect_that!(loaded.radius, eq(1));
        expect_that!(loaded.cells.len(), eq(map.cells.len()));

        for (coord, cell) in map.cells.iter() {
            expect_that!(loaded.cells.get(coord), some(eq(cell)));
        }

        Ok(())
    }

    #[googletest::test]
    fn test_save_load_legacy_roundtrip() -> Result<()> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_map_legacy.bin");

        let mut map = Map::new(1);
        let coords: Vec<Coord> = map.iter_coords().collect();
        for coord in coords {
            map.cells.insert(coord, Cell::new(Region::Inside, 1, true));
        }

        // Save legacy
        save_map(&map, &path, true).unwrap();

        // Check file size (should be 1 + 7 = 8 bytes, vs 9 bytes for modern)
        let metadata = std::fs::metadata(&path)?;
        expect_that!(metadata.len(), eq(8));

        // Load (auto-detect)
        let loaded = load_map(&path).unwrap();

        expect_that!(loaded.radius, eq(1));
        expect_that!(loaded.cells.len(), eq(map.cells.len()));

        Ok(())
    }
}
