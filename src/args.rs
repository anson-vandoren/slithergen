use std::str::FromStr;

use argh::FromArgs;

/// Standard grid sizes
///
/// Radius in this case assumes the center hex is r=0
/// Therefore the number of tiles is 3r^2 + 3r + 1
#[derive(Clone, Copy, Debug, Default)]
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
#[derive(Clone, Copy, Debug, Default)]
pub enum Difficulty {
    /// roughly 15% of hints removed
    Easy,
    /// Roughly 30% of hints removed
    Medium,
    #[default]
    /// As many hints as possible removed
    Hard,
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
#[derive(FromArgs)]
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
}

impl Args {
    pub fn new() -> Self {
        let mut args: Args = argh::from_env();
        let mut errs = Vec::new();
        if args.radius.is_none() && args.size.is_none() {
            errs.push("No radius or size given, using default size 'small'");
            args.size = Some(GridSize::Small);
        }
        if args.radius.is_some() && args.size.is_some() {
            errs.push("Both radius and size given, going with radius");
            args.size = None;
            args.radius = Some(args.radius.unwrap());
        }

        if args.difficulty.is_none() {
            errs.push("No difficulty given, using default difficulty 'hard'");
            args.difficulty = Some(Difficulty::Hard);
        }

        // print in yellow using ansi escape codes
        if errs.len() > 0 {
            println!("\x1b[33m{}\x1b[0m", errs.join("\n"));
        }

        args
    }
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let radius = self.radius.unwrap();
        let size = self
            .size
            .map(|s| s.to_string())
            .unwrap_or("unknown".to_owned());
        let difficulty = self.difficulty.unwrap();
        write!(
            f,
            "Generate a map with radius: {} ({}), difficulty: {}",
            radius, size, difficulty
        )
    }
}
