use tracing::{span, trace, Level};

use super::Item;

#[derive(Default)]
pub struct OnBoardInit {
    command_list: Vec<String>,
}

impl Item for OnBoardInit {
    fn name(&self) -> String {
        "OnBoardInit".to_string()
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let span = span!(Level::INFO, "OnBoardInitActivated");
        let _enter = span.enter();

        self.command_list.push(status_bar.buffer.clone());
        trace!(
            "Command '{}' has been added, total = {}",
            status_bar.buffer,
            self.command_list.len()
        );
    }

    fn board_init(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let span = span!(Level::INFO, "OnBoardInitBoardReset");
        let _enter = span.enter();

        trace!(
            "Board has been reset, queueing {} command/s...",
            self.command_list.len()
        );

        status_bar
            .enter_buffer_commands(&self.command_list.iter().map(|x| &x[..]).collect::<Vec<_>>());
    }

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::None
    }
}
