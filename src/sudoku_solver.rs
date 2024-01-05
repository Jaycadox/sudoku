use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::JoinHandle,
};

use bit_vec::BitVec;
use macroquad::miniquad::debug;
use threadpool::ThreadPool;

use crate::{sudoku_game::SudokuGame, task_status::TaskStatus};

pub struct SolveTask {
    _thread: JoinHandle<()>,
    rx: Receiver<Option<SudokuGame>>,
    status: TaskStatus<SudokuGame>,
}

impl SolveTask {
    pub fn new(game: &SudokuGame) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let game = game.clone();
        Self {
            rx,
            status: TaskStatus::<SudokuGame>::Waiting(std::time::Instant::now()),
            _thread: std::thread::spawn(move || {
                if let Err(e) = tx.send(solve(&game)) {
                    eprintln!("solve_task :: failed to send to parent thread, the game might have already reset. {e}");
                }
            }),
        }
    }

    pub fn get(&mut self) -> &TaskStatus<SudokuGame> {
        if let Ok(result) = self.rx.try_recv() {
            self.status = match result {
                Some(game) => TaskStatus::<SudokuGame>::Done(Box::new(game)),
                None => TaskStatus::<SudokuGame>::Failed,
            };
        }

        &self.status
    }
}

pub fn solve(game: &SudokuGame) -> Option<SudokuGame> {
    let start = std::time::Instant::now();
    let pool = Arc::new(Mutex::new(ThreadPool::new(num_cpus::get())));
    let mut game = game.clone();

    solve_basic_inner(&mut game, 0);
    let g = solve_inner(&mut game, 0, pool, 0);
    if g.is_some() {
        let end = std::time::Instant::now();
        let time = end.duration_since(start);
        debug!("solver :: took {}s", time.as_secs_f64());
    }
    g
}

fn get_cells_in_box(game: &SudokuGame, box_pos: (u32, u32)) -> Vec<u32> {
    let size = game.cells.shape()[1];
    let (start_x, start_y) = (box_pos.0 * 3, box_pos.1 * 3);

    let mut cells = Vec::with_capacity(9);

    for inner_box_x in 0..3 {
        for inner_box_y in 0..3 {
            let idx = SudokuGame::xy_pos_to_idx(
                start_x + inner_box_x,
                start_y + inner_box_y,
                size as u32,
            );
            cells.push(idx);
        }
    }

    cells
}

fn get_cells_in_col(game: &SudokuGame, col: u32) -> Vec<u32> {
    let size = game.cells.shape()[1];
    let mut cells = Vec::with_capacity(9);

    let mut idx = col;
    cells.push(idx);
    for _col in 0..size - 1 {
        idx += size as u32;
        cells.push(idx);
    }

    cells
}

fn get_cells_in_row(game: &SudokuGame, row: u32) -> Vec<u32> {
    let size = game.cells.shape()[1];
    let mut cells = Vec::with_capacity(9);

    let mut idx = row * size as u32;
    cells.push(idx);
    for _row in 0..size - 1 {
        idx += 1;
        cells.push(idx);
    }

    cells
}

// Algorithm to deduce certain tiles, a lot faster than solve inner but it can't
// solve on its own, should speed up the backtrace algorithm
fn solve_basic_inner(game: &mut SudokuGame, depth: usize) {
    const APPLY_ON_ROWS_AND_COLS: bool = false; // seems to decrease performance

    if depth > 1000 {
        eprintln!("solve basic inner stuck in recursion");
        return;
    }

    fn run_solve_stage(
        num: u8,
        game: &mut SudokuGame,
        cell_group: Vec<u32>,
        invalid_tiles: &[u32],
    ) -> bool {
        let size = game.cells.shape()[1];
        let cell_group = cell_group
            .iter()
            .filter(|x| {
                !invalid_tiles.contains(x) && {
                    let (sx, sy) = SudokuGame::idx_pos_to_xy(**x, size as u32);
                    game.cells[(sy as usize, sx as usize)] == 0
                }
            })
            .collect::<Vec<_>>();
        if cell_group.len() == 1 {
            let (sx, sy) = SudokuGame::idx_pos_to_xy(*cell_group[0], size as u32);
            game.cells[(sy as usize, sx as usize)] = num;
            true
        } else {
            false
        }
    }

    let mut made_change = false;
    for num in 1..=9 {
        let invalid_tiles = game.get_all_cells_which_see_number(num);
        for box_x in 0..3 {
            for box_y in 0..3 {
                let cells_in_box = get_cells_in_box(game, (box_x, box_y));
                if run_solve_stage(num, game, cells_in_box, &invalid_tiles) {
                    made_change = true;
                }
            }
        }

        if APPLY_ON_ROWS_AND_COLS {
            for col_or_row in 0..9 {
                let cells_in_row = get_cells_in_row(game, col_or_row);
                if run_solve_stage(num, game, cells_in_row, &invalid_tiles) {
                    made_change = true;
                }

                let cells_in_col = get_cells_in_col(game, col_or_row);
                if run_solve_stage(num, game, cells_in_col, &invalid_tiles) {
                    made_change = true;
                }
            }
        }
    }
    if made_change {
        solve_basic_inner(game, depth + 1);
    }
}

fn solve_inner(
    game: &mut SudokuGame,
    mut start_idx: usize,
    thread_pool: Arc<Mutex<ThreadPool>>,
    depth: usize,
) -> Option<SudokuGame> {
    let size = game.cells.shape()[1] as u32;
    // Find next blank cell
    while start_idx != game.cells.len() {
        let cell_pos = SudokuGame::idx_pos_to_xy(start_idx as u32, size);
        if game.cells[(cell_pos.1 as usize, cell_pos.0 as usize)] != 0 {
            start_idx += 1;
        } else {
            break;
        }
    }

    if start_idx == game.cells.len() {
        return Some(game.clone());
    }

    let cell_pos = SudokuGame::idx_pos_to_xy(start_idx as u32, size);
    let occupied = get_occupied_numbers_at_cell(game, cell_pos);
    let valid_moves = (1..=9).filter(|x| !occupied[x - 1]).collect::<Vec<usize>>();

    if depth != 0 {
        for num in valid_moves {
            let old = game.cells[(cell_pos.1 as usize, cell_pos.0 as usize)];
            game.cells[(cell_pos.1 as usize, cell_pos.0 as usize)] = num as u8;
            if let Some(game) = solve_inner(game, start_idx + 1, thread_pool.clone(), depth + 1) {
                return Some(game);
            }
            game.cells[(cell_pos.1 as usize, cell_pos.0 as usize)] = old;
        }
    } else {
        let (tx, rx) = std::sync::mpsc::channel();
        for num in valid_moves {
            let tx = tx.clone();
            let mut game = game.clone();
            let thread_pool_2 = thread_pool.clone();
            thread_pool.lock().unwrap().execute(move || {
                game.cells[(cell_pos.1 as usize, cell_pos.0 as usize)] = num as u8;
                if let Some(game) =
                    solve_inner(&mut game, start_idx + 1, thread_pool_2.clone(), depth + 1)
                {
                    let _ = tx.send(game);
                }
            });
        }

        if let Ok(game) = rx.recv_timeout(std::time::Duration::from_secs(10)) {
            return Some(game);
        }
    }

    None
}

fn get_occupied_numbers_at_cell(game: &SudokuGame, cell_pos: (u32, u32)) -> BitVec {
    let mut vec = BitVec::from_elem(9, false);
    let size = game.cells.shape()[1] as u32;

    game.get_cells_in_sight(cell_pos)
        .iter()
        .map(|idx| {
            let (sx, sy) = SudokuGame::idx_pos_to_xy(*idx, size);
            game.cells[(sy as usize, sx as usize)]
        })
        .for_each(|cell_num| {
            if cell_num != 0 {
                vec.set((cell_num - 1) as usize, true);
            }
        });

    vec
}
