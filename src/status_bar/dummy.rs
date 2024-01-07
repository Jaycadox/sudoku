use macroquad::color::WHITE;

use super::StatusBarItem;

#[derive(Default)]
pub struct Dummy;

impl StatusBarItem for Dummy {
    fn name(&self) -> &'static str {
        "Dummy"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        _status_bar: &mut super::StatusBar,
    ) {
    }

    fn update(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
    ) -> (String, macroquad::prelude::Color) {
        ("".to_string(), WHITE)
    }

    fn board_init(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        _status_bar: &mut super::StatusBar,
    ) {
    }

    fn status(&mut self) -> super::StatusBarItemStatus {
        super::StatusBarItemStatus::Ok(super::StatusBarItemOkData::None)
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }
}
