mod args;
pub mod io;
pub mod model;
pub mod viewer;

fn main() {
    let args: args::Args = argh::from_env();
    let config = args.normalize();
    // Create output directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        return;
    }

    if let Some(path) = config.load_path {
        // Load mode
        match io::load_map(&path) {
            Ok(map) => {
                println!("Loaded map with radius {}", map.radius);
                if config.display {
                    viewer::display_map(&map);
                }
            }
            Err(e) => eprintln!("Failed to load map: {}", e),
        }
    } else {
        // Generation mode
        if config.tasks.is_empty() {
            println!("No generation tasks scheduled.");
            return;
        }

        let task_count = config.tasks.len();
        println!(
            "Generating {} puzzles per task ({} tasks)...",
            config.count_per_task, task_count
        );

        for (radius, difficulty) in config.tasks {
            for i in 0..config.count_per_task {
                // STUB: Replace with actual generation logic
                let mut map = model::Map::new(radius);
                // Fill with dummy data for testing viewer/io
                let coords: Vec<model::Coord> = map.iter_coords().collect();
                for coord in coords {
                    map.cells
                        .insert(coord, model::Cell::new(model::Region::Inside, 3, true));
                }

                if config.display && config.count_per_task == 1 && task_count == 1 {
                    viewer::display_map(&map);
                }

                // Save
                let filename = format!("slither_r{}_{:?}_{}.bin", radius, difficulty, i);
                let path = config.output_dir.join(filename);
                if let Err(e) = io::save_map(&map, &path) {
                    eprintln!("Failed to save map to {:?}: {}", path, e);
                }
            }
        }
    }
}
