use super::StatusBarItem;

#[derive(Default)]
pub struct OnBoardInit {
    command_list: Vec<String>,
}

impl StatusBarItem for OnBoardInit {
    fn name(&self) -> &'static str {
        "OnBoardInit"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        self.command_list.push(status_bar.buffer.clone());
    }

    fn board_init(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        status_bar
            .enter_buffer_commands(&self.command_list.iter().map(|x| &x[..]).collect::<Vec<_>>());
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }
}
