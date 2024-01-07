use std::{sync::mpsc::Receiver, thread::JoinHandle};

use macroquad::{
    color::{GRAY, GREEN, RED, YELLOW},
    rand::ChooseRandom,
};
use rand::Rng;

use crate::{
    status_bar::cpu_solver::{self, SolveTask},
    sudoku_game::SudokuGame,
    task_status::TaskStatus,
};

use super::StatusBarItem;

enum BoardGenUpdate {
    FinalResult(Option<SudokuGame>),
    ProgressReport(u8),
}

#[derive(Clone)]
enum BoardGenStatus {
    Done,
    Waiting(u8),
    NotStarted,
    Failed,
}

pub struct BoardGen {
    _thread: JoinHandle<()>,
    rx: Receiver<BoardGenUpdate>,
    status: BoardGenStatus,
}

impl Default for BoardGen {
    fn default() -> Self {
        let (_, rx) = std::sync::mpsc::channel();
        Self {
            _thread: std::thread::spawn(|| {}),
            rx,
            status: BoardGenStatus::NotStarted,
        }
    }
}

impl StatusBarItem for BoardGen {
    fn name(&self) -> &'static str {
        "BoardGen"
    }

    fn activated(&mut self, _old_game: &mut SudokuGame, buffer: &mut String) {
        let num_tiles_target = match buffer.parse::<u8>() {
            Ok(val) => val,
            Err(_) => {
                if !buffer.is_empty() {
                    *buffer = "BoardGen: failed to parse tiles target".to_string();
                    self.status = BoardGenStatus::Failed;
                    return;
                }
                30
            }
        };

        let mut game = SudokuGame::new(None);
        let count = game.cells.iter().count();

        if num_tiles_target as usize >= count {
            *buffer = format!("BoardGen: tiles target too large. max={count}");
            self.status = BoardGenStatus::Failed;
            return;
        }

        fn inner(game: &mut SudokuGame, start_idx: usize, count: usize) -> bool {
            if start_idx == count {
                return false;
            }

            let cell = game.cells.iter_mut().nth(start_idx).unwrap();
            let old_cell_val = *cell;

            let (sx, sy) =
                SudokuGame::idx_pos_to_xy(start_idx as u32, game.cells.shape()[1] as u32);
            let occupied = cpu_solver::get_occupied_numbers_at_cell(game, (sx, sy));

            let mut numbers = (1..=9).collect::<Vec<_>>();
            numbers.shuffle();

            let mut amount_valid = 0;
            for num in numbers {
                let valid = !occupied[num - 1];
                if valid {
                    amount_valid += 1;
                    *game.cells.iter_mut().nth(start_idx).unwrap() = num as u8;
                    if inner(game, start_idx + 1, count) {
                        return true;
                    }
                    if start_idx != count - 1 {
                        *game.cells.iter_mut().nth(start_idx).unwrap() = old_cell_val;
                    }
                }
            }
            amount_valid == 1 && start_idx == count - 1
        }

        inner(&mut game, 0, count);

        let (tx, rx) = std::sync::mpsc::channel();
        self.rx = rx;
        self._thread = std::thread::spawn(move || {
            let mut total_numbers = count;
            let mut attempted_cells = vec![];

            let mut previous_states: Vec<(SudokuGame, usize)> = vec![];
            let mut undo_count = 0;

            while total_numbers > num_tiles_target as usize {
                let random_tile_idx = rand::thread_rng().gen_range(0..count);
                if attempted_cells.contains(&random_tile_idx) {
                    continue;
                }
                if attempted_cells.len() == count - 1 {
                    if let Some(previous_state) = previous_states.pop() {
                        if undo_count < 10000 {
                            game = previous_state.0;
                            total_numbers = previous_state.1;
                            attempted_cells.clear();
                            undo_count += 1;
                            continue;
                        }
                    }

                    tx.send(BoardGenUpdate::FinalResult(None)).unwrap();
                    return;
                }

                attempted_cells.push(random_tile_idx);

                let og_value = *game.cells.iter_mut().nth(random_tile_idx).unwrap();
                if og_value == 0 {
                    continue;
                }

                let game_before_modification = game.clone();

                *game.cells.iter_mut().nth(random_tile_idx).unwrap() = 0;
                let mut solve_task = SolveTask::new(&game);
                while let TaskStatus::Waiting(_) = solve_task.get() {}
                match solve_task.get() {
                    TaskStatus::Done(solved_game) => {
                        if game_before_modification
                            .cells
                            .iter()
                            .nth(random_tile_idx)
                            .unwrap()
                            != solved_game.cells.iter().nth(random_tile_idx).unwrap()
                        {
                            *game.cells.iter_mut().nth(random_tile_idx).unwrap() = og_value;
                        } else {
                            total_numbers -= 1;
                            tx.send(BoardGenUpdate::ProgressReport(total_numbers as u8))
                                .unwrap();

                            previous_states.push((game.clone(), total_numbers));

                            attempted_cells.clear();
                        }
                    }
                    TaskStatus::Failed => {
                        *game.cells.iter_mut().nth(random_tile_idx).unwrap() = og_value;
                    }
                    _ => panic!("should be impossible"),
                }
            }
            game.unradified.clear();
            tx.send(BoardGenUpdate::FinalResult(Some(game.clone())))
                .unwrap();
        });
    }

    fn update(&mut self, game: &mut SudokuGame) -> (String, macroquad::prelude::Color) {
        if let Ok(status_update) = self.rx.try_recv() {
            self.status = match &status_update {
                BoardGenUpdate::FinalResult(Some(_)) => BoardGenStatus::Done,
                BoardGenUpdate::FinalResult(None) => BoardGenStatus::Failed,
                BoardGenUpdate::ProgressReport(progress) => BoardGenStatus::Waiting(*progress),
            };

            if let BoardGenUpdate::FinalResult(Some(new_game)) = status_update {
                game.reset(new_game);
            }
        }
        match self.status {
            BoardGenStatus::Done => ("finished".to_string(), GREEN),
            BoardGenStatus::Waiting(n) => (format!("{n}/81   "), YELLOW),
            BoardGenStatus::NotStarted => ("inactive".to_string(), GRAY),
            BoardGenStatus::Failed => ("failure ".to_string(), RED),
        }
    }

    fn board_init(&mut self, _game: &mut SudokuGame, _buffer: &mut String) {
        let old_status = self.status.clone();
        *self = Default::default();
        self.status = old_status;
    }

    fn status(&mut self) -> super::StatusBarItemStatus {
        super::StatusBarItemStatus::Ok(super::StatusBarItemOkData::None)
    }
}
