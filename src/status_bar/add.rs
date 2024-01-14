use tracing::{span, trace, Level};

use super::{
    board_gen::BoardGen, cpu_solve::SolveTask, fps::Fps, on_board_init::OnBoardInit, StatusBarItem,
};

#[derive(Default)]
pub struct BuiltinAdd;

impl StatusBarItem for BuiltinAdd {
    fn name(&self) -> &'static str {
        "BuiltinAdd"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let span = span!(Level::INFO, "BuiltinAddActivate");
        let _enter = span.enter();

        trace!(
            "Attempting to add items from input: '{}'",
            status_bar.buffer
        );

        let mut count = 0;
        for item in status_bar.buffer.to_lowercase().split_whitespace() {
            trace!("Adding item: '{}'...", item);
            match item {
                "boardgen" => status_bar.add::<BoardGen>(),
                "onboardinit" => status_bar.add::<OnBoardInit>(),
                "cpusolve" => status_bar.add::<SolveTask>(),
                "fps" => status_bar.add::<Fps>(),
                _ => {
                    status_bar.buffer = "BuiltinAdd: could not find item".to_string();
                    break;
                }
            };
            count += 1;
        }

        trace!("Added {} item/s to status bar", count);
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }
}
