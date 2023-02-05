use std::iter::zip;

use anyhow::Context;

use super::{colours, colours::Colour};

pub const UNRENDERED_ROWS: usize = 4;
const NUM_ROWS: usize = 28;
const NUM_COLS: usize = 12;
const MAX_SPEED: u8 = 10;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockState {
    Emp,
    Arr,
    Ell,
    Eye,
    Ess,
    Ohh,
    Tee,
    Zee,
}

impl BlockState {
    pub fn to_tetromino(&self) -> Tetromino {
        match self {
            BlockState::Arr => Tetromino::arr(),
            BlockState::Ell => Tetromino::ell(),
            BlockState::Eye => Tetromino::eye(),
            BlockState::Ess => Tetromino::ess(),
            BlockState::Ohh => Tetromino::ohh(),
            BlockState::Tee => Tetromino::tee(),
            BlockState::Zee => Tetromino::zee(),
            _ => Tetromino::eye(),
        }
    }
}

const NEXT_TETRO_BAG: [BlockState; 7] = [
    BlockState::Arr,
    BlockState::Ell,
    BlockState::Ess,
    BlockState::Eye,
    BlockState::Ohh,
    BlockState::Tee,
    BlockState::Zee,
];

pub struct CurrentTetromino {
    pub tetromino: Tetromino,
    pub x: usize,
    pub y: usize,
}

impl CurrentTetromino {
    pub fn next_one() -> CurrentTetromino {
        let next_idx: usize = random_number::random!(..NEXT_TETRO_BAG.len());
        let tetro = NEXT_TETRO_BAG[next_idx].to_tetromino();
        tetro.into()
    }
    pub fn down(&mut self) {
        self.y -= 1;
    }
}

#[derive(Debug)]
pub enum GameEvent {
    Step,
}

pub struct GameState {
    pub blocks: [[BlockState; NUM_COLS]; NUM_ROWS + UNRENDERED_ROWS],
    pub score: u128,
    pub level: u8,
    pub time_elapsed: u8,
    pub current_tetromino: CurrentTetromino,
}

impl std::default::Default for GameState {
    fn default() -> Self {
        GameState {
            blocks: [[BlockState::Emp; NUM_COLS]; UNRENDERED_ROWS + NUM_ROWS],
            score: 0,
            level: 0,
            time_elapsed: 0,
            current_tetromino: CurrentTetromino::next_one(),
        }
    }
}

impl GameState {
    pub fn step_time(
        &mut self,
        event_loop: &winit::event_loop::EventLoopProxy<GameEvent>,
    ) -> anyhow::Result<()> {
        if self.time_elapsed > self.step_to_pass() {
            self.update_blocks();
            self.time_elapsed = 0;
            event_loop
                .send_event(GameEvent::Step)
                .context("Couldn't send GameEvent::Step")?;
        }
        self.time_elapsed += 1;
        Ok(())
    }

    pub fn tetromino_down(&mut self) {
        if self.can_move(0, -1) {
            self.current_tetromino.down();
        } else {
            // Commit
        }
    }

    fn step_to_pass(&self) -> u8 {
        if self.level > 10 {
            MAX_SPEED
        } else {
            MAX_SPEED - self.level
        }
    }

    fn can_move(&self, dx: i8, dy: i8) -> bool {
        let (width, height) = {
            (
                self.current_tetromino.tetromino.shape[0].len() as i8,
                self.current_tetromino.tetromino.shape.len() as i8,
            )
        };

        let (blocks_x, blocks_y) = {
            (
                dx + self.current_tetromino.x as i8,
                dy + self.current_tetromino.y as i8,
            )
        };

        // Check board bounds
        if blocks_x + width >= NUM_COLS as i8
            || blocks_y + height >= (NUM_ROWS + UNRENDERED_ROWS) as i8
        {
            return false;
        }
        if blocks_x < 0 {
            return false;
        }

        let tetromino_last_line = self.current_tetromino.tetromino.shape.last().unwrap();
        let blocks_row = &self.blocks[blocks_y as usize];
        for col in zip(
            blocks_row[blocks_x as usize..(blocks_x + width) as usize].iter(),
            tetromino_last_line.iter(),
        ) {
            if col != (&BlockState::Emp, &BlockState::Emp) {
                return false;
            }
        }
        true
    }

    fn remove(&mut self) {
        let mut new_blocks = [[BlockState::Emp; NUM_COLS]; UNRENDERED_ROWS + NUM_ROWS];

        let mut copy_to = UNRENDERED_ROWS + NUM_ROWS;
        for row in self.blocks.iter().rev() {
            let unfilled = row
                .iter()
                .map(|x| *x == BlockState::Emp)
                .reduce(|acc, x| acc && x)
                .unwrap_or(false);
            if unfilled {
                copy_to -= 1;
                new_blocks[copy_to][..NUM_COLS].copy_from_slice(row);
            }
        }

        self.blocks = new_blocks;
    }

    fn update_blocks(&mut self) {
        self.remove();
    }
}

pub struct Tetromino {
    /// Colour associated with blocks of this tetromino
    pub colour: Colour,
    /// A bounding box of blocks that has the shape filled in with coloured blocks
    pub shape: Vec<Vec<BlockState>>,
    /// Padding from the top of the board so that bottom of the tetromino is just
    /// above the visible part. Basically `UNRENDERED_ROWS - tetromino_height`.
    pub starting_pos_y: usize,
    /// Starting positin in the horizontal.
    /// Basically `(NUM_COLS - tetromino_width) / 2`.
    pub starting_pos_x: usize,
}

impl Into<CurrentTetromino> for Tetromino {
    fn into(self) -> CurrentTetromino {
        CurrentTetromino {
            x: self.starting_pos_x,
            y: self.starting_pos_y,
            tetromino: self,
        }
    }
}

impl Tetromino {
    ///
    /// xx
    /// x
    /// x
    ///
    pub fn arr() -> Self {
        Tetromino {
            colour: colours::RED,
            shape: vec![
                vec![BlockState::Arr, BlockState::Arr],
                vec![BlockState::Arr, BlockState::Emp],
                vec![BlockState::Arr, BlockState::Emp],
            ],
            starting_pos_y: UNRENDERED_ROWS - 3,
            starting_pos_x: (NUM_COLS - 2) / 2,
        }
    }

    ///
    /// x
    /// x
    /// xx
    ///
    pub fn ell() -> Self {
        Tetromino {
            colour: colours::BROWN,
            shape: vec![
                vec![BlockState::Ell, BlockState::Emp],
                vec![BlockState::Ell, BlockState::Emp],
                vec![BlockState::Ell, BlockState::Ell],
            ],
            starting_pos_y: UNRENDERED_ROWS - 3,
            starting_pos_x: (NUM_COLS - 2) / 2,
        }
    }

    ///
    ///  xx
    /// xx
    ///
    pub fn ess() -> Self {
        Tetromino {
            colour: colours::MAROON,
            shape: vec![
                vec![BlockState::Emp, BlockState::Ess, BlockState::Ess],
                vec![BlockState::Ess, BlockState::Ess, BlockState::Emp],
            ],
            starting_pos_y: UNRENDERED_ROWS - 2,
            starting_pos_x: (NUM_COLS - 3) / 2,
        }
    }

    ///
    /// x
    /// x
    /// x
    /// x
    ///
    pub fn eye() -> Self {
        Tetromino {
            colour: colours::LIGHT_PURPLE,
            shape: vec![
                vec![BlockState::Eye],
                vec![BlockState::Eye],
                vec![BlockState::Eye],
                vec![BlockState::Eye],
            ],
            starting_pos_y: UNRENDERED_ROWS - 4,
            starting_pos_x: (NUM_COLS - 1) / 2,
        }
    }

    ///
    /// xx
    /// xx
    ///
    pub fn ohh() -> Self {
        Tetromino {
            colour: colours::NAVY_BLUE,
            shape: vec![
                vec![BlockState::Ohh, BlockState::Ohh],
                vec![BlockState::Ohh, BlockState::Ohh],
            ],
            starting_pos_y: UNRENDERED_ROWS - 2,
            starting_pos_x: (NUM_COLS - 2) / 2,
        }
    }

    ///
    /// xxx
    ///  x
    ///
    pub fn tee() -> Self {
        Tetromino {
            colour: colours::GRAY,
            shape: vec![
                vec![BlockState::Emp, BlockState::Tee, BlockState::Emp],
                vec![BlockState::Tee, BlockState::Tee, BlockState::Tee],
            ],
            starting_pos_y: UNRENDERED_ROWS - 2,
            starting_pos_x: (NUM_COLS - 3) / 2,
        }
    }

    ///
    /// xx
    ///  xx
    ///
    pub fn zee() -> Self {
        Tetromino {
            colour: colours::GREEN,
            shape: vec![
                vec![BlockState::Zee, BlockState::Zee, BlockState::Emp],
                vec![BlockState::Emp, BlockState::Zee, BlockState::Zee],
            ],
            starting_pos_y: UNRENDERED_ROWS - 2,
            starting_pos_x: (NUM_COLS - 3) / 2,
        }
    }
}
