use super::game_state::NUM_COLS;
use super::{colours, colours::Colour};

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

impl Into<Tetromino> for BlockState {
    fn into(self) -> Tetromino {
        match self {
            BlockState::Arr => Tetromino::arr(),
            BlockState::Ell => Tetromino::ell(),
            BlockState::Eye => Tetromino::eye(),
            BlockState::Ess => Tetromino::ess(),
            BlockState::Ohh => Tetromino::ohh(),
            BlockState::Tee => Tetromino::tee(),
            BlockState::Zee => Tetromino::zee(),
            BlockState::Emp => Tetromino::emp(),
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
        let tetro: Tetromino = NEXT_TETRO_BAG[next_idx].into();
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

    pub fn rotate(&mut self) {
        self.tetromino.rotate();
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
    /// A CCW rotation of the tetromino
    pub fn rotate(&mut self) {
        let (height, width) = (self.shape.len(), self.shape[0].len());
        let mut new_shape = vec![vec![BlockState::Emp; height]; width];
        for (row_idx, row) in self.shape.iter().enumerate() {
            for (col_idx, col) in row.iter().enumerate() {
                new_shape[width - col_idx - 1][row_idx] = *col;
            }
        }
        self.shape = new_shape;
    }

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
    ///  x
    /// xxx
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

    ///
    /// xx
    ///  xx
    ///
    pub fn emp() -> Self {
        Tetromino {
            colour: colours::GREEN,
            shape: vec![vec![]],
            starting_pos_y: 0,
            starting_pos_x: 0,
        }
    }
}
