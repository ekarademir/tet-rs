use anyhow::Context;

use super::{colours, colours::Colour};

const NUM_ROWS: usize = 28;
pub const UNRENDERED_ROWS: usize = 28;
const NUM_COLS: usize = 12;
const MAX_SPEED: u8 = 10;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockState {
    Emp,
    Unrendered,
    Eye,
    Tee,
    Ess,
    Zee,
    Arr,
    Ell,
}

pub struct GameState {
    pub blocks: [[BlockState; NUM_COLS]; NUM_ROWS + UNRENDERED_ROWS],
    pub score: u128,
    pub level: u8,
    pub time_elapsed: u8,
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

    fn move_blocks_down(&mut self) {
        //
    }
}

impl std::default::Default for GameState {
    fn default() -> Self {
        let mut blocks = [[BlockState::Emp; NUM_COLS]; UNRENDERED_ROWS + NUM_ROWS];
        for x in 0..UNRENDERED_ROWS {
            blocks[x] = [BlockState::Unrendered; NUM_COLS];
        }
        GameState {
            blocks,
            score: 0,
            level: 0,
            time_elapsed: 0,
        }
    }
}

pub struct Tetromino<const R: usize, const C: usize> {
    pub colour: Colour,
    pub shape: [[BlockState; C]; R],
}

pub const Eye: Tetromino<4, 1> = Tetromino {
    colour: colours::LIGHT_PURPLE,
    shape: [
        [BlockState::Eye],
        [BlockState::Eye],
        [BlockState::Eye],
        [BlockState::Eye],
    ],
};

pub const Tee: Tetromino<2, 3> = Tetromino {
    colour: colours::GRAY,
    shape: [
        [BlockState::Emp, BlockState::Tee, BlockState::Emp],
        [BlockState::Tee, BlockState::Tee, BlockState::Tee],
    ],
};

pub const Ess: Tetromino<2, 3> = Tetromino {
    colour: colours::MAROON,
    shape: [
        [BlockState::Emp, BlockState::Ess, BlockState::Ess],
        [BlockState::Ess, BlockState::Ess, BlockState::Emp],
    ],
};

pub const Zee: Tetromino<2, 3> = Tetromino {
    colour: colours::GREEN,
    shape: [
        [BlockState::Zee, BlockState::Zee, BlockState::Emp],
        [BlockState::Emp, BlockState::Zee, BlockState::Zee],
    ],
};

pub const Arr: Tetromino<3, 2> = Tetromino {
    colour: colours::RED,
    shape: [
        [BlockState::Arr, BlockState::Arr],
        [BlockState::Arr, BlockState::Emp],
        [BlockState::Arr, BlockState::Emp],
    ],
};

pub const Ell: Tetromino<3, 2> = Tetromino {
    colour: colours::BROWN,
    shape: [
        [BlockState::Ell, BlockState::Emp],
        [BlockState::Ell, BlockState::Emp],
        [BlockState::Ell, BlockState::Ell],
    ],
};
