use std::str::FromStr;

use super::colours::Colour;

const NUM_ROWS: usize = 28;
const NUM_COLS: usize = 12;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockState {
    Empty,
    Filled,
}

pub struct GameState {
    pub blocks: [[BlockState; NUM_COLS]; NUM_ROWS],
    pub score: String,
    pub level: String,
}

impl std::default::Default for GameState {
    fn default() -> Self {
        GameState {
            blocks: [[BlockState::Filled; NUM_COLS]; NUM_ROWS],
            score: "0000".to_string(),
            level: "000".to_string(),
        }
    }
}

pub struct Tetromino<const R: usize, const C: usize> {
    pub colour: Colour,
    pub shape: [[BlockState; R]; C],
}
