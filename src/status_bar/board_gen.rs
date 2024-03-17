use std::{
    sync::{atomic::AtomicBool, mpsc::Receiver, Arc},
    thread::JoinHandle,
};

use macroquad::rand::ChooseRandom;
use rand::Rng;
use tracing::{debug, error, span, trace, Level};

use crate::{
    draw_helper::AppColour,
    status_bar::cpu_solve::{self, SolveTask},
    sudoku_game::SudokuGame,
    task_status::TaskStatus,
};

use super::{StatusBar, StatusBarItem};

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
    thread: JoinHandle<()>,
    rx: Receiver<BoardGenUpdate>,
    status: BoardGenStatus,
    should_stop: Arc<AtomicBool>,
}

impl Default for BoardGen {
    fn default() -> Self {
        let (_, rx) = std::sync::mpsc::channel();
        let gen = Self {
            thread: std::thread::spawn(|| {}),
            rx,
            status: BoardGenStatus::NotStarted,
            should_stop: Arc::new(AtomicBool::new(false)),
        };

        while !gen.thread.is_finished() {}

        gen
    }
}

impl StatusBarItem for BoardGen {
    fn name(&self) -> &'static str {
        "BoardGen"
    }

    fn activated(&mut self, game: &mut SudokuGame, status_bar: &mut StatusBar) {
        let span = span!(Level::INFO, "BoardGenActivate");
        let _enter = span.enter();

        if status_bar.buffer.len() == game.cells.len() {
            trace!("Assuming user wants to create board from string");
            let new_game = SudokuGame::new(Some(&status_bar.buffer));
            game.reset(new_game);
            return;
        }

        if !self.thread.is_finished() {
            self.should_stop
                .store(true, std::sync::atomic::Ordering::Relaxed);
            debug!("BoardGen thread is already active, signalling shutdown...");
            return;
        }

        self.should_stop
            .store(false, std::sync::atomic::Ordering::Relaxed);

        trace!(
            "Parsing requested number of remaining tiles: {}...",
            status_bar.buffer
        );

        self.status = BoardGenStatus::Waiting(81);
        let num_tiles_target = match status_bar.buffer.parse::<u8>() {
            Ok(val) => val,
            Err(_) => {
                if !status_bar.buffer.is_empty() {
                    error!("Failed to parse number of remaining tiles");
                    status_bar.buffer = "BoardGen: failed to parse tiles target".to_string();
                    self.status = BoardGenStatus::Failed;
                    return;
                }
                let default = 30;
                trace!("Input is empty, so using default ({})", default);
                default
            }
        };

        let mut game = SudokuGame::new(None);
        let count = game.cells.iter().count();

        if num_tiles_target as usize >= count {
            error!("Input is too large, max = {}", count);
            status_bar.buffer = format!("BoardGen: tiles target too large. max={count}");
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
            let occupied = cpu_solve::get_occupied_numbers_at_cell(game, (sx, sy));

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

        {
            let span = span!(Level::INFO, "GenerateFilled");
            let _enter = span.enter();
            trace!("Starting filled board generation...");
            inner(&mut game, 0, count);
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.rx = rx;

        let span = span.clone();
        let should_stop = Arc::clone(&self.should_stop);
        self.thread = std::thread::spawn(move || {
            let _parent_enter = span.enter();

            let span = span!(Level::INFO, "Prune");
            let _enter = span.enter();

            let mut total_numbers = count;
            let mut attempted_cells = vec![];

            let mut previous_states: Vec<(SudokuGame, usize)> = vec![];
            let mut undo_count = 0;

            trace!(
                "Starting to prune filled board to match target ({})...",
                num_tiles_target
            );
            while total_numbers > num_tiles_target as usize
                && !should_stop.load(std::sync::atomic::Ordering::Relaxed)
            {
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

                    error!("Failed to prune due to too many retries");
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
                            if total_numbers % 10 == 0 {
                                trace!(
                                    "{}% complete...",
                                    ((num_tiles_target as f32 / total_numbers as f32) * 100.0)
                                        as u32
                                );
                            }

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

            debug!("Board successfully pruned");
            game.unradified.clear();
            tx.send(BoardGenUpdate::FinalResult(Some(game.clone())))
                .unwrap();
        });
    }

    fn update(
        &mut self,
        game: &mut SudokuGame,
        status_bar: &mut StatusBar,
    ) -> (String, macroquad::prelude::Color) {
        let span = span!(Level::INFO, "BoardGenUpdate");
        let _enter = span.enter();

        while let Ok(status_update) = self.rx.try_recv() {
            self.status = match &status_update {
                BoardGenUpdate::FinalResult(Some(_)) => BoardGenStatus::Done,
                BoardGenUpdate::FinalResult(None) => BoardGenStatus::Failed,
                BoardGenUpdate::ProgressReport(progress) => BoardGenStatus::Waiting(*progress),
            };

            if let BoardGenUpdate::FinalResult(Some(new_game)) = status_update {
                trace!("Received final result from BoardGen thread");
                game.reset(new_game);
            }
        }
        match self.status {
            BoardGenStatus::Done => (
                "done ".to_string(),
                status_bar.drawing.colour(AppColour::StatusBarItemOkay),
            ),
            BoardGenStatus::Waiting(n) => (
                format!("{n}/81"),
                status_bar
                    .drawing
                    .colour(AppColour::StatusBarItemInProgress),
            ),
            BoardGenStatus::NotStarted => (
                "ready".to_string(),
                status_bar.drawing.colour(AppColour::StatusBarItem),
            ),
            BoardGenStatus::Failed => (
                "fail ".to_string(),
                status_bar.drawing.colour(AppColour::StatusBarItemError),
            ),
        }
    }

    fn board_init(&mut self, _game: &mut SudokuGame, _status_bar: &mut StatusBar) {
        let old_status = self.status.clone();
        *self = Default::default();
        self.status = old_status;
    }

    fn status(&mut self) -> super::StatusBarItemStatus {
        match self.status {
            BoardGenStatus::NotStarted | BoardGenStatus::Done => {
                super::StatusBarItemStatus::Ok(super::StatusBarItemOkData::None)
            }
            BoardGenStatus::Waiting(_) => super::StatusBarItemStatus::Waiting,
            BoardGenStatus::Failed => super::StatusBarItemStatus::Err,
        }
    }
}
