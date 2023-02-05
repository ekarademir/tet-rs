use anyhow::Context;

use super::{colours, colours::Colour};

pub const NUM_ROWS: usize = 28;
pub const NUM_COLS: usize = 12;
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

#[derive(Clone)]
pub struct CurrentTetromino {
    pub tetromino: Tetromino,
    pub x: usize,
    pub y: i8,
}

impl CurrentTetromino {
    pub fn next_one() -> CurrentTetromino {
        let next_idx: usize = random_number::random!(..NEXT_TETRO_BAG.len());
        let tetro = NEXT_TETRO_BAG[next_idx].to_tetromino();
        tetro.into()
    }

    pub fn down(&mut self) {
        self.y += 1;
    }

    pub fn right(&mut self) {
        self.x += 1;
    }

    pub fn left(&mut self) {
        let newx = self.x as i8 - 1;
        self.x = if newx >= 0 { newx as usize } else { 0 };
    }
}

#[derive(Debug)]
pub enum GameEvent {
    Step,
}

pub struct GameState {
    pub blocks: [[BlockState; NUM_COLS]; NUM_ROWS],
    pub score: u128,
    pub level: u8,
    pub current_tetromino: CurrentTetromino,
    pub next_tetromino: CurrentTetromino,
    pub time_elapsed: u8,
    pub steps_elapsed: u128,
}

impl std::default::Default for GameState {
    fn default() -> Self {
        GameState {
            blocks: [[BlockState::Emp; NUM_COLS]; NUM_ROWS],
            score: 0,
            level: 0,
            time_elapsed: 0,
            steps_elapsed: 0,
            current_tetromino: CurrentTetromino::next_one(),
            next_tetromino: CurrentTetromino::next_one(),
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
        self.steps_elapsed += 1;
        Ok(())
    }

    pub fn tetromino_down(&mut self) {
        if self.can_move(0, 1) {
            self.current_tetromino.down();
        } else {
            // Commit
        }
    }

    pub fn tetromino_right(&mut self) {
        if self.can_move(1, 0) {
            self.current_tetromino.right();
        }
    }

    pub fn tetromino_left(&mut self) {
        if self.can_move(-1, 0) {
            self.current_tetromino.left();
        }
    }

    pub fn tetromino_rotate(&mut self) {
        //
    }

    fn step_to_pass(&self) -> u8 {
        if self.level > 10 {
            MAX_SPEED
        } else {
            MAX_SPEED - self.level
        }
    }

    fn can_move(&self, dx: i8, dy: i8) -> bool {
        let mut tetro = self.current_tetromino.clone();
        let x = tetro.x as i8 + dx;
        tetro.x = if x >= 0 { x as usize } else { 0 };
        tetro.y += dy;
        self.can_do(&tetro)
    }

    fn can_do(&self, ctetro: &CurrentTetromino) -> bool {
        let (tetro_width, tetro_height) = {
            (
                ctetro.tetromino.shape[0].len() as i8,
                ctetro.tetromino.shape.len() as i8,
            )
        };

        // with the movement
        let (tetro_blocks_x, tetro_blocks_y) = (ctetro.x as i8, ctetro.y);

        let (board_width, board_height) = (NUM_COLS as i8, NUM_ROWS as i8);
        let (board_x, board_y) = (0, 0);

        // Check board bounds
        if tetro_blocks_x < board_x
            || tetro_blocks_x + tetro_width > board_x + board_width
            || tetro_blocks_y + tetro_height < board_y // Rebate the off screen starting
            || tetro_blocks_y + tetro_height > board_y + board_height
        {
            return false;
        }

        // Check if it intersects with the board
        // Visible piece of the tetromino
        let (tetro_row_start, tetro_row_end) = {
            let in_view = tetro_blocks_y + tetro_height;
            if in_view < tetro_height {
                ((tetro_height - in_view) as usize, tetro_height as usize)
            } else {
                (0, tetro_height as usize)
            }
        };

        // Intersected part of the board
        let (board_row_start, board_row_end) = (
            (tetro_blocks_y + tetro_height) as usize,
            (tetro_blocks_y + tetro_height) as usize + tetro_row_end - tetro_row_start,
        );

        let board_row_end = if board_row_end < NUM_ROWS {
            (tetro_blocks_y + tetro_height) as usize + tetro_row_end - tetro_row_start
        } else {
            NUM_ROWS
        };

        let (board_col_start, board_col_end) = (
            tetro_blocks_x as usize,
            (tetro_blocks_x + tetro_width) as usize,
        );

        for (drow, row) in self.blocks[board_row_start..board_row_end]
            .iter()
            .enumerate()
        {
            for (dcol, block) in row[board_col_start..board_col_end].iter().enumerate() {
                //
                let tetro_block = &ctetro.tetromino.shape[drow][dcol];
                if *tetro_block != BlockState::Emp && *block != BlockState::Emp {
                    return false;
                }
            }
        }
        true
    }

    fn remove(&mut self) {
        let mut new_blocks = [[BlockState::Emp; NUM_COLS]; NUM_ROWS];

        let mut copy_to = NUM_ROWS;
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
        // self.remove();
        self.tetromino_down();
    }
}

#[derive(Clone)]
pub struct Tetromino {
    /// Colour associated with blocks of this tetromino
    pub colour: Colour,
    /// A bounding box of blocks that has the shape filled in with coloured blocks
    pub shape: Vec<Vec<BlockState>>,
    /// Padding from the top of the board so that bottom of the tetromino is just
    /// above the visible part. Basically `- tetromino_height`.
    pub starting_pos_y: i8,
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
            starting_pos_y: -3,
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
            starting_pos_y: -3,
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
            starting_pos_y: -2,
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
            starting_pos_y: -4,
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
            starting_pos_y: -2,
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
            starting_pos_y: -2,
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
            starting_pos_y: -2,
            starting_pos_x: (NUM_COLS - 3) / 2,
        }
    }
}
