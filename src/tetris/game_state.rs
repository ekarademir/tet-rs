const NUM_ROWS: usize = 28;
const NUM_COLS: usize = 12;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockState {
    Empty,
    Filled,
}

pub struct GameState {
    pub blocks: [[BlockState; NUM_COLS]; NUM_ROWS],
}

impl std::default::Default for GameState {
    fn default() -> Self {
        GameState {
            blocks: [[BlockState::Filled; NUM_COLS]; NUM_ROWS],
        }
    }
}
