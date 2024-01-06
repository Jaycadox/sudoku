use macroquad::{color::WHITE, time::get_fps};

use super::StatusBarItem;

#[derive(Default)]
pub struct Fps;

impl StatusBarItem for Fps {
    fn name(&self) -> &'static str {
        "Fps"
    }

    fn activated(&mut self, _game: &mut crate::sudoku_game::SudokuGame) {}

    fn update(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
    ) -> (String, macroquad::prelude::Color) {
        (format!("{:<4}", get_fps()), WHITE)
    }

    fn board_init(&mut self, _game: &mut crate::sudoku_game::SudokuGame) {}

    fn status(&mut self) -> super::StatusBarItemStatus {
        super::StatusBarItemStatus::Ok(super::StatusBarItemOkData::None)
    }
}
