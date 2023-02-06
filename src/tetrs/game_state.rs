use anyhow::Context;

use super::tetromino::{BlockState, CurrentTetromino};
use super::GameEvent;

pub const NUM_ROWS: usize = 28;
pub const NUM_COLS: usize = 12;
const MAX_SPEED: u8 = 42;
const MAX_LEVEL: u8 = 40;
const SCORE_PER_LEVEL: u128 = 20;

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
        if self.time_elapsed > self.current_speed() {
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
            self.commit();
            self.current_tetromino = self.next_tetromino.clone();
            self.next_tetromino = CurrentTetromino::next_one();
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
        if self.can_rotate() {
            self.current_tetromino.rotate();
        }
    }

    pub fn current_speed(&self) -> u8 {
        if self.level > MAX_LEVEL {
            MAX_SPEED - MAX_LEVEL
        } else {
            MAX_SPEED - self.level
        }
    }

    fn commit(&mut self) {
        let (t_width, t_height) = (
            self.current_tetromino.tetromino.shape[0].len(),
            self.current_tetromino.tetromino.shape.len(),
        );

        let (start_x, start_y) = (self.current_tetromino.x, self.current_tetromino.y as usize);

        for (y, row) in (start_y..start_y + t_height).enumerate() {
            for (x, col) in (start_x..start_x + t_width).enumerate() {
                let tetro_bit = self.current_tetromino.tetromino.shape[y][x];
                if tetro_bit != BlockState::Emp {
                    self.blocks[row][col] = self.current_tetromino.tetromino.shape[y][x];
                }
            }
        }
    }

    fn can_move(&self, dx: i8, dy: i8) -> bool {
        let mut tetro = self.current_tetromino.clone();
        let x = tetro.x as i8 + dx;
        tetro.x = if x >= 0 { x as usize } else { 0 };
        tetro.y += dy;
        self.can_do(&tetro)
    }

    fn can_rotate(&self) -> bool {
        let mut tetro = self.current_tetromino.clone();
        tetro.tetromino.rotate();
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
        let in_view = tetro_blocks_y + tetro_height;

        // Intersected part of the board
        let (board_row_start, board_row_end) = {
            if in_view < tetro_height {
                (0, in_view as usize)
            } else {
                let bend = (tetro_blocks_y + tetro_height) as usize;
                if bend < NUM_ROWS {
                    (
                        (tetro_blocks_y) as usize,
                        (tetro_blocks_y + tetro_height) as usize,
                    )
                } else {
                    ((tetro_blocks_y) as usize, NUM_ROWS)
                }
            }
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

    fn remove_lines(&mut self) -> u8 {
        let mut new_blocks = [[BlockState::Emp; NUM_COLS]; NUM_ROWS];

        let mut num_removed = 0;

        let mut copy_to = NUM_ROWS;
        for row in self.blocks.iter().rev() {
            let unfilled = row
                .iter()
                .map(|x| *x == BlockState::Emp)
                .reduce(|acc, x| acc || x)
                .unwrap_or(false);
            if unfilled {
                copy_to -= 1;
                new_blocks[copy_to][..NUM_COLS].copy_from_slice(row);
            } else {
                num_removed += 1;
            }
        }

        self.blocks = new_blocks;

        num_removed
    }

    fn update_score(&mut self, num_removed: u8) {
        if num_removed > 0 {
            self.score += num_removed as u128;
        }

        self.update_level();
    }

    fn update_level(&mut self) {
        self.level = (self.score / SCORE_PER_LEVEL) as u8;
    }

    fn update_blocks(&mut self) {
        self.tetromino_down();
        let n = self.remove_lines();
        self.update_score(n);
    }
}
