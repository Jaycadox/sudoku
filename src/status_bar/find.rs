use tracing::{Level, span};
use crate::sudoku_game::{ResetSignal, SudokuGame};

use super::{StatusBar, StatusBarItem};

#[derive(Default)]
pub struct Find;

impl StatusBarItem for Find {
    fn name(&self) -> &'static str {
        "Find"
    }

    fn activated(
        &mut self,
        game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let span = span!(Level::INFO, "FindActivate");
        let _enter = span.enter();

        let mut buffer = status_bar.buffer.clone();
        let mut direction = FindDirection::Ahead;
        if buffer.len() == 2 {
            if buffer.chars().next() == Some('.') {
                direction = FindDirection::Behind;
                buffer.remove(0);
            }
        }

        if let Some(target) = buffer.chars().next().and_then(|x| x.to_digit(10)) {
            let size = game.cells.shape()[1];
            let cursor_pos = game.selected_cell.map(|x| SudokuGame::xy_pos_to_idx(x.0, x.1, size as u32)).unwrap_or(0);

            let Some((x, y)) = find(game, target as u8, cursor_pos, direction) else {
                status_bar.buffer = String::from("Could not find instance of character");
                return;
            };
            game.selected_cell = Some((x, y));
        } else {
            status_bar.buffer = String::from("Expected to start with single digit character");
        }


    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::NameOnly
    }
}

enum FindDirection {
    Ahead,
    Behind
}

impl FindDirection {
    fn to_offset(self) -> i8 {
        match self {
            FindDirection::Ahead => 1,
            FindDirection::Behind => -1
        }
    }
}

fn find(game: &mut SudokuGame, target: u8, start: u32, direction: FindDirection) -> Option<(u32, u32)> {
    let offset = direction.to_offset();
    let mut i: i64 = start as i64;

    if i != 0 {
        i += offset as i64;
    }

    while i >= 0 && i < game.cells.len() as i64 {
        let Some(num) = game.cells.iter().nth(i as usize) else {
            break;
        };

        if *num == target {
            let size = game.cells.shape()[1];
            return Some(SudokuGame::idx_pos_to_xy(i as u32, size as u32));
        }

        i += offset as i64;
    }

    None
}