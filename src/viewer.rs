use crate::model::{Cell, Coord, Map, Region};

const COLOR_RESET: &str = "\x1b[0m";
const BG_YELLOW: &str = "\x1b[43m";
const BG_MAGENTA: &str = "\x1b[45m";
const FG_BLACK: &str = "\x1b[30m";
const FG_WHITE: &str = "\x1b[37m";

pub fn display_map(map: &Map) {
    let r = map.radius as i8;

    // We need to print row by row.
    // Spec order is q first then r.
    // For printing, we want to iterate y (rows) then x (cols).
    // In axial coords, rows are roughly aligned with 'r', but it's slanted.
    //
    // To print a hex grid in terminal, we can use "offset coordinates" or just map axial to screen coords.
    // A simple approach for Axial (q, r):
    // y = r
    // x = q + r/2 (this is for "pointy top" usually, but let's check assumptions)
    //
    // Actually, let's just iterate r from top to bottom (-R to R),
    // and for each row, iterate q.
    // The range of q depends on r.
    //
    // Range of r is -R to +R.
    // Range of q for a given r is:
    // max(-R, -r-R) <= q <= min(R, -r+R)

    // But wait, the spec says:
    // Outer loop: q from -R to +R.
    // Inner loop: r from max(-R, -q-R) to min(R, -q+R).
    // This defines the valid set of (q, r).

    // To print nicely, we want constant r rows.
    for r_curr in -(r as i32)..=(r as i32) {
        let r_val = r_curr as i8;

        // Calculate indentation
        // The top row (r=-R) has length R+1.
        // It needs indentation.
        // Let's compute the "start q" and "end q" for this row r.
        let q_min = (-r).max(-r_val - r);
        let q_max = r.min(-r_val + r);

        // Indentation logic:
        // As r increases from -R to 0, the row shifts left? or right?
        // Let's visualize R=2.
        // r=-2: q in [0, 2] -> (0, -2), (1, -2), (2, -2). 3 hexes.
        // r=-1: q in [-1, 2] -> (-1, -1)...(2, -1). 4 hexes.
        // r=0:  q in [-2, 2] -> (-2, 0)...(2, 0). 5 hexes.
        // r=1:  q in [-2, 1] -> 4 hexes.
        // r=2:  q in [-2, 0] -> 3 hexes.

        // To center them, we need spaces.
        // Row length = q_max - q_min + 1.
        // Max row length is 2R+1 (at r=0).
        // Difference is (2R+1) - current_len.
        // We pad half that difference.

        // However, hex grids are staggered.
        // r=-2: 0 1 2   (shifted right relative to r=0?)
        // r=0: -2 -1 0 1 2

        // Let's just use absolute spacing.
        // We can indent by `abs(r_val)`.
        let indent = r_val.abs();
        print!("{:width$}", "", width = indent as usize * 2); // 2 spaces per half-hex shift?

        // Actually, typical hex ASCII art:
        //   / \ / \
        //  |   |   |
        //   \ / \ /
        //
        // Simple "block" representation:
        // [ ] [ ] [ ]
        //  [ ] [ ] [ ] [ ]
        // [ ] [ ] [ ] [ ] [ ]

        for q_val in q_min..=q_max {
            let coord = Coord::new(q_val, r_val);
            if let Some(cell) = map.cells.get(&coord) {
                print_cell(cell);
            } else {
                print!("??  ");
            }
            print!("  "); // Spacing between hexes in a row
        }
        println!(); // End of row
    }
}

fn print_cell(cell: &Cell) {
    let (bg, fg) = match cell.region {
        Region::Inside => (BG_YELLOW, FG_BLACK),
        Region::Outside => (BG_MAGENTA, FG_WHITE),
    };

    let content = if cell.clue_visible {
        format!("{}", cell.full_neighbor_count)
    } else {
        " ".to_string()
    };

    print!("{}{}[{}]{}", bg, fg, content, COLOR_RESET);
}
