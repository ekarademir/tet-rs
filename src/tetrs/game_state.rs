use anyhow::Context;

use super::{colours, colours::Colour};

const NUM_ROWS: usize = 28;
const NUM_COLS: usize = 12;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockState {
    Emp,
    Filled,
    Eye,
    Tee,
    Ess,
    Zee,
    Arr,
    Ell,
}

pub struct GameState {
    pub blocks: [[BlockState; NUM_COLS]; NUM_ROWS],
    pub score: String,
    pub level: String,
    pub time_elapsed: u128,
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
        self.time_elapsed += 1;
        event_loop
            .send_event(GameEvent::Step)
            .context("Couldn't send GameEvent::Step")?;
        Ok(())
    }
}

impl std::default::Default for GameState {
    fn default() -> Self {
        GameState {
            blocks: [[BlockState::Emp; NUM_COLS]; NUM_ROWS],
            score: "0000".to_string(),
            level: "000".to_string(),
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
