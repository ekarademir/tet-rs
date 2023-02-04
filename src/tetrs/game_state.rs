use anyhow::Context;

use super::{colours, colours::Colour};

const NUM_ROWS: usize = 28;
pub const UNRENDERED_ROWS: usize = 28;
const NUM_COLS: usize = 12;
const MAX_SPEED: u8 = 10;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockState {
    Emp,
    Eye,
    Tee,
    Ess,
    Zee,
    Arr,
    Ell,
}

pub struct CurrentTetromino {
    pub tetromino: Tetromino,
    pub x: usize,
    pub y: usize,
}

pub struct GameState {
    pub blocks: [[BlockState; NUM_COLS]; NUM_ROWS + UNRENDERED_ROWS],
    pub score: u128,
    pub level: u8,
    pub time_elapsed: u8,
    pub current_tetromino: Option<CurrentTetromino>,
}

#[derive(Debug)]
pub enum GameEvent {
    Step,
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

    fn step_to_pass(&self) -> u8 {
        if self.level > 10 {
            MAX_SPEED
        } else {
            MAX_SPEED - self.level
        }
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

impl std::default::Default for GameState {
    fn default() -> Self {
        GameState {
            blocks: [[BlockState::Emp; NUM_COLS]; UNRENDERED_ROWS + NUM_ROWS],
            score: 0,
            level: 0,
            time_elapsed: 0,
            current_tetromino: None,
        }
    }
}

pub struct Tetromino {
    pub colour: Colour,
    pub shape: Vec<Vec<BlockState>>,
}

impl Tetromino {
    pub fn eye() -> Self {
        Tetromino {
            colour: colours::LIGHT_PURPLE,
            shape: vec![
                vec![BlockState::Eye],
                vec![BlockState::Eye],
                vec![BlockState::Eye],
                vec![BlockState::Eye],
            ],
        }
    }

    pub fn tee() -> Self {
        Tetromino {
            colour: colours::GRAY,
            shape: vec![
                vec![BlockState::Emp, BlockState::Tee, BlockState::Emp],
                vec![BlockState::Tee, BlockState::Tee, BlockState::Tee],
            ],
        }
    }

    pub fn ess() -> Self {
        Tetromino {
            colour: colours::MAROON,
            shape: vec![
                vec![BlockState::Emp, BlockState::Ess, BlockState::Ess],
                vec![BlockState::Ess, BlockState::Ess, BlockState::Emp],
            ],
        }
    }

    pub fn zee() -> Self {
        Tetromino {
            colour: colours::GREEN,
            shape: vec![
                vec![BlockState::Zee, BlockState::Zee, BlockState::Emp],
                vec![BlockState::Emp, BlockState::Zee, BlockState::Zee],
            ],
        }
    }

    pub fn arr() -> Self {
        Tetromino {
            colour: colours::RED,
            shape: vec![
                vec![BlockState::Arr, BlockState::Arr],
                vec![BlockState::Arr, BlockState::Emp],
                vec![BlockState::Arr, BlockState::Emp],
            ],
        }
    }

    pub fn ell() -> Self {
        Tetromino {
            colour: colours::BROWN,
            shape: vec![
                vec![BlockState::Ell, BlockState::Emp],
                vec![BlockState::Ell, BlockState::Emp],
                vec![BlockState::Ell, BlockState::Ell],
            ],
        }
    }
}
