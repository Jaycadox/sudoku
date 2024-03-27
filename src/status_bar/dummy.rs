use super::Item;

#[derive(Default)]
pub struct Dummy;

impl Item for Dummy {
    fn name(&self) -> &'static str {
        "Dummy"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        _status_bar: &mut super::StatusBar,
    ) {
    }

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::None
    }
}
