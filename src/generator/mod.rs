use crate::args::Difficulty;
use crate::model::Map;

pub mod dummy;

pub use dummy::DummyGenerator;

pub trait Generator {
    fn generate(&self, radius: u8, difficulty: Difficulty) -> Map;
}
