use tracing::{Level, span, trace};

use crate::{config, shorthand};
use crate::status_bar::{
    background_image::BackgroundImage, colour_overwrite::ColourOverwrite, font::Font,
    hard_reset::HardReset, padding::Padding, pencil_marks::PencilMarks,
};
use crate::status_bar::eval::Eval;
use crate::status_bar::find::Find;
use crate::status_bar::shorthands::list::ShorthandList;

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

        if status_bar.buffer == "__show_config" {
            let config_dir = config::get_file_path("");
            if let Err(e) = opener::open(config_dir) {
                status_bar.buffer = format!("Error opening config dir: {e}");
            }
            return;
        }

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
                "colouroverwrite" => status_bar.add::<ColourOverwrite>(),
                "backgroundimage" => status_bar.add::<BackgroundImage>(),
                "pencilmarks" => status_bar.add::<PencilMarks>(),
                "padding" => status_bar.add::<Padding>(),
                "hardreset" => status_bar.add::<HardReset>(),
                "find" => status_bar.add::<Find>(),
                "font" => status_bar.add::<Font>(),
                "eval" => status_bar.add::<Eval>(),
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

    fn shorthands(&self) -> Option<ShorthandList> {
        shorthand!((r"^config$", "__show_config"))
    }
}
