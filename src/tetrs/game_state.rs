use anyhow::Context;

use super::tetromino::{BlockState, CurrentTetromino, Tetromino};

pub const NUM_ROWS: usize = 28;
pub const NUM_COLS: usize = 12;
const MAX_SPEED: u8 = 10;

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
