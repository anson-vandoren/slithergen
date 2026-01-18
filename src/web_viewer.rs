use crate::model::{Cell, Coord, Map};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Custom serialization structure for the Map.
/// We convert the HashMap<Coord, Cell> to a flat list of cells to ensure
/// consistent JSON serialization and easier consumption in JavaScript,
/// avoiding potential issues with non-string keys in JSON objects.
#[derive(Serialize)]
struct WebMap<'a> {
    radius: u8,
    cells: Vec<WebCell<'a>>,
}

#[derive(Serialize)]
struct WebCell<'a> {
    coords: Coord,
    cell: &'a Cell,
}

impl<'a> From<&'a Map> for WebMap<'a> {
    fn from(map: &'a Map) -> Self {
        let cells = map
            .cells
            .iter()
            .map(|(&coords, cell)| WebCell { coords, cell })
            .collect();
        WebMap {
            radius: map.radius,
            cells,
        }
    }
}

pub fn show_map(map: &Map) {
    let web_map = WebMap::from(map);
    let json_data = serde_json::to_string(&web_map).expect("Failed to serialize map");

    // Read template (embedded at compile time)
    let template = include_str!("web_viewer/template.html");

    // Inject data
    let html_content = template.replace("/* DATA_PLACEHOLDER */ null", &json_data);

    // Write to a temporary HTML file in the current directory.
    let output_path = Path::new("slithergen_view.html");

    let mut file = File::create(output_path).expect("Failed to create viewer file");
    file.write_all(html_content.as_bytes())
        .expect("Failed to write viewer HTML");

    println!("Generated viewer at {:?}", output_path);

    // Open in browser
    if let Err(e) = open::that(output_path) {
        eprintln!("Failed to open browser: {}", e);
    }
}
