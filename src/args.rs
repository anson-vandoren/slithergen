use std::path::PathBuf;
use std::str::FromStr;

use argh::FromArgs;

/// Standard grid sizes
///
/// Radius in this case assumes the center hex is r=0
/// Therefore the number of tiles is 3r^2 + 3r + 1
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum GridSize {
    #[default]
    /// Radius 2, 19 hexes
    Small = 2,
    /// Radius 4, 61 hexes
    Medium = 4,
    /// Radius 8, 217 hexes
    Large = 8,
    /// Radius 11, 397 hexes
    Huge = 11,
}

impl GridSize {
    pub fn all() -> &'static [GridSize] {
        &[
            GridSize::Small,
            GridSize::Medium,
            GridSize::Large,
            GridSize::Huge,
        ]
    }
}

impl std::fmt::Display for GridSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridSize::Small => f.write_str("small"),
            GridSize::Medium => f.write_str("medium"),
            GridSize::Large => f.write_str("large"),
            GridSize::Huge => f.write_str("huge"),
        }
    }
}

impl FromStr for GridSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "small" => Ok(GridSize::Small),
            "sm" => Ok(GridSize::Small),
            "medium" => Ok(GridSize::Medium),
            "med" => Ok(GridSize::Medium),
            "large" => Ok(GridSize::Large),
            "lg" => Ok(GridSize::Large),
            "huge" => Ok(GridSize::Huge),
            "yuge" => Ok(GridSize::Huge),
            _ => Err(format!("Invalid grid size: {}", s)),
        }
    }
}

/// Relative difficulty of the puzzle
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Difficulty {
    /// roughly 15% of hints removed
    Easy,
    /// Roughly 30% of hints removed
    Medium,
    #[default]
    /// As many hints as possible removed
    Hard,
}

impl Difficulty {
    pub fn all() -> &'static [Difficulty] {
        &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Easy => f.write_str("easy"),
            Difficulty::Medium => f.write_str("medium"),
            Difficulty::Hard => f.write_str("hard"),
        }
    }
}

impl FromStr for Difficulty {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            _ => Err(format!("Invalid difficulty: {}", s)),
        }
    }
}

/// Slitherlink map generator
#[derive(Debug, FromArgs)]
pub struct Args {
    /// hexagonal grid radius - exclusive with and takes precedence over size
    #[argh(option)]
    pub radius: Option<u8>,

    /// hexagonal grid radius expressed as a relative size - exclusive with and overwritten by radius
    #[argh(option)]
    pub size: Option<GridSize>,

    /// difficulty of the puzzle to be generated
    #[argh(option)]
    pub difficulty: Option<Difficulty>,

    /// output directory for generated maps. defaults to "./maps"
    #[argh(option, default = "String::from(\"./maps\")")]
    pub output: String,

    /// generate one of every size and difficulty. implied if no other filtering args are given
    #[argh(switch)]
    pub all: bool,

    /// number of puzzles to generate per permutation
    #[argh(option)]
    pub count: Option<u32>,

    /// positional count argument. if provided, behaves like --count and --all
    #[argh(positional)]
    pub count_pos: Option<u32>,

    /// input file to load and display (skips generation)
    #[argh(option)]
    pub load: Option<String>,

    /// display the generated or loaded puzzle in terminal
    #[argh(switch)]
    pub display: bool,

    /// output format (currently only 'binary-full' is supported)
    #[argh(
        option,
        from_str_fn(output_format_from_str),
        default = "OutputFormat::BinaryFull"
    )]
    pub format: OutputFormat,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputFormat {
    BinaryFull,
}

fn output_format_from_str(s: &str) -> Result<OutputFormat, String> {
    match s {
        "binary-full" => Ok(OutputFormat::BinaryFull),
        _ => Err(format!("Unknown format: {}", s)),
    }
}

// Rewriting Config to handle custom radius vs GridSize clearer.
// Actually, I will simplify: we generate a list of (Radius, Difficulty).
// GridSize is just a helper for Radius.
#[derive(Debug, PartialEq)]
pub struct ResolvedConfig {
    pub output_dir: PathBuf,
    pub count_per_task: u32,
    pub tasks: Vec<(u8, Difficulty)>, // radius, difficulty
    pub load_path: Option<PathBuf>,
    pub display: bool,
    pub format: OutputFormat,
}

impl Args {
    pub fn normalize(&self) -> ResolvedConfig {
        let count = self.count_pos.or(self.count).unwrap_or(1);
        let output_dir = PathBuf::from(&self.output);

        let specific_size_or_radius = self.size.is_some() || self.radius.is_some();
        let specific_difficulty = self.difficulty.is_some();
        let force_all = self.all || self.count_pos.is_some();

        // Logic for Radii
        let final_radii: Vec<u8> = if specific_size_or_radius {
            if let Some(r) = self.radius {
                vec![r]
            } else {
                vec![self.size.unwrap() as u8]
            }
        } else if force_all || !specific_difficulty {
            GridSize::all().iter().map(|s| *s as u8).collect()
        } else {
            // Default fallthrough: If we are here, specific_difficulty is true, but size is not specified.
            // "If a single numeric arg is given with no flag... assume --all flag" -> handled by force_all
            // "If no args are given... default to one of each size/type" -> handled by !specific_difficulty check above? NO.
            // If no args given: specific_size=false, specific_diff=false, force_all=false.
            // !specific_diff is true. So we go to GridSize::all(). Correct.

            // Case: `slithergen --difficulty hard`.
            // specific_size=false, specific_diff=true, force_all=false.
            // Falls through to here.
            // We should default to ALL sizes if not specified?
            // "If no args are given, default output folder and default to one of each size/type"
            // Implies default is ALL. So if I only constrain difficulty, size remains ALL.
            GridSize::all().iter().map(|s| *s as u8).collect()
        };

        // Logic for Difficulties
        let final_difficulties: Vec<Difficulty> = if specific_difficulty {
            vec![self.difficulty.unwrap()]
        } else {
            Difficulty::all().to_vec()
        };

        let mut tasks = Vec::new();
        for &r in &final_radii {
            for &d in &final_difficulties {
                tasks.push((r, d));
            }
        }

        ResolvedConfig {
            output_dir,
            count_per_task: count,
            tasks,
            load_path: self.load.as_ref().map(PathBuf::from),
            display: self.display,
            format: self.format,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[googletest::test]
    fn no_args_uses_all() -> Result<()> {
        let args = Args::from_args(&[], &[]).unwrap();
        let config = args.normalize();

        expect_that!(config.count_per_task, eq(1));
        // Use matches_pattern or eq reference for PathBuf
        expect_that!(config.output_dir, eq(&PathBuf::from("./maps")));
        // Should be all 4 sizes * 3 difficulties = 12 tasks
        expect_that!(config.tasks, len(eq(12)));
        Ok(())
    }

    #[googletest::test]
    fn single_numeric_arg_implies_count_and_all() -> Result<()> {
        let args = Args::from_args(&[], &["5"]).unwrap();
        let config = args.normalize();

        expect_that!(config.count_per_task, eq(5));
        expect_that!(config.tasks, len(eq(12))); // All implied
        Ok(())
    }

    #[googletest::test]
    fn explicit_all_flag_sets_count_one() -> Result<()> {
        let args = Args::from_args(&[], &["--all"]).unwrap();
        let config = args.normalize();

        expect_that!(config.count_per_task, eq(1));
        expect_that!(config.tasks, len(eq(12)));
        Ok(())
    }

    #[googletest::test]
    fn specific_size_filters_tasks() -> Result<()> {
        let args = Args::from_args(&[], &["--size", "small"]).unwrap();
        let config = args.normalize();

        // Specific size, default difficulties (all 3)
        expect_that!(config.tasks, len(eq(3)));
        let radii: Vec<u8> = config.tasks.iter().map(|(r, _)| *r).collect();
        expect_that!(radii, each(eq(&(GridSize::Small as u8))));
        Ok(())
    }

    #[googletest::test]
    fn specific_difficulty_filters_tasks() -> Result<()> {
        let args = Args::from_args(&[], &["--difficulty", "hard"]).unwrap();
        let config = args.normalize();

        // Default sizes (all 4), specific difficulty
        expect_that!(config.tasks, len(eq(4)));
        let difficulties: Vec<Difficulty> = config.tasks.iter().map(|(_, d)| *d).collect();
        expect_that!(difficulties, each(eq(&Difficulty::Hard)));
        Ok(())
    }

    #[googletest::test]
    fn specific_size_and_diff_singles_task() -> Result<()> {
        let args = Args::from_args(&[], &["--size", "huge", "--difficulty", "easy"]).unwrap();
        let config = args.normalize();

        expect_that!(config.tasks, len(eq(1)));
        expect_that!(
            config.tasks,
            elements_are![eq(&(GridSize::Huge as u8, Difficulty::Easy))]
        );
        Ok(())
    }

    #[googletest::test]
    fn numeric_arg_with_filter_combines() -> Result<()> {
        // "5 --size small"
        let args = Args::from_args(&[], &["5", "--size", "small"]).unwrap();
        let config = args.normalize();

        expect_that!(config.count_per_task, eq(5));
        // Small size, all difficulties (3)
        expect_that!(config.tasks, len(eq(3)));
        Ok(())
    }

    #[googletest::test]
    fn custom_output_dir() -> Result<()> {
        let args = Args::from_args(&[], &["--output", "foo/bar"]).unwrap();
        let config = args.normalize();
        expect_that!(config.output_dir, eq(&PathBuf::from("foo/bar")));
        Ok(())
    }
}
