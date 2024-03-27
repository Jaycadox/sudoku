use crate::sudoku_game::ResetSignal;

use super::Item;

#[derive(Default)]
pub struct HardReset;

impl Item for HardReset {
    fn name(&self) -> &'static str {
        "HardReset"
    }

    fn activated(
        &mut self,
        game: &mut crate::sudoku_game::SudokuGame,
        _status_bar: &mut super::StatusBar,
    ) {
        game.reset_signalled = ResetSignal::Hard;
    }

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::NameOnly
    }
}
