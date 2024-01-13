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

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }
}
