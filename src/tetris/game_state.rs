const NUM_ROWS: usize = 28;
const NUM_COLS: usize = 12;

#[derive(Clone, Copy)]
pub(super) enum BlockState {
    Empty,
    Filled,
}

pub(super) struct GameState {
    blocks: [[BlockState; NUM_COLS]; NUM_ROWS],
}

impl std::default::Default for GameState {
    fn default() -> Self {
        GameState {
            blocks: [[BlockState::Empty; NUM_COLS]; NUM_ROWS],
        }
    }
}
