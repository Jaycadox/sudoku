use super::{board_gen::BoardGen, cpu_solve::SolveTask, fps::Fps, StatusBarItem};

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
        let buffer = status_bar.buffer.to_lowercase();
        match &buffer[..] {
            "boardgen" => status_bar.add::<BoardGen>(),
            "cpusolve" => status_bar.add::<SolveTask>(),
            "fps" => status_bar.add::<Fps>(),
            _ => {
                status_bar.buffer = "BuiltinAdd: could not find item".to_string();
            }
        };
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }
}
