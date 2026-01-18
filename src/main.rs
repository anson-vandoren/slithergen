mod args;
pub mod io;
// pub mod viewer; // Deprecated
pub mod model;
pub mod web_viewer;

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
                    web_viewer::show_map(&map);
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

        let mut displayed_first_map = false;

        for (radius, difficulty) in config.tasks {
            for i in 0..config.count_per_task {
                // TODO: Replace with actual generation logic
                let mut map = model::Map::new(radius);
                // Fill with dummy data for testing viewer/io
                let coords: Vec<model::Coord> = map.iter_coords().collect();
                for coord in coords {
                    map.cells
                        .insert(coord, model::Cell::new(model::Region::Inside, 3, true));
                }

                if config.display && !displayed_first_map {
                    web_viewer::show_map(&map);
                    displayed_first_map = true;
                    if task_count > 1 || config.count_per_task > 1 {
                        println!("(Displaying only the first generated map)");
                    }
                }

                // Save
                // Determine size folder name
                let size_str = match radius {
                    2 => "small",
                    4 => "medium",
                    8 => "large",
                    11 => "huge",
                    _ => "custom", // Fallback for custom radii
                };
                let size_dir = if size_str == "custom" {
                    format!("radius_{}", radius)
                } else {
                    size_str.to_string()
                };

                let diff_str = difficulty.to_string(); // Difficulty implements Display

                let save_dir = config.output_dir.join(&size_dir).join(&diff_str);
                if let Err(e) = std::fs::create_dir_all(&save_dir) {
                    eprintln!("Failed to create directory {:?}: {}", save_dir, e);
                    continue;
                }

                let filename = format!("{}.bin", i);
                let path = save_dir.join(filename);
                if let Err(e) = io::save_map(&map, &path) {
                    eprintln!("Failed to save map to {:?}: {}", path, e);
                }
            }
        }
    }
}
