use crate::{config, draw_helper::DrawingSettings};

use super::StatusBarItem;

#[derive(Default)]
pub struct Font;

impl StatusBarItem for Font {
    fn name(&self) -> &'static str {
        "Font"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let mut args = status_bar.buffer.split_whitespace();

        let Some((file_name, size)) = args
            .next()
            .and_then(|x| Some((x, args.next()?.parse::<f32>().ok()?)))
        else {
            status_bar.buffer = "CouldNotParse".to_string();
            return;
        };

        let Some(font) = config::get_file(file_name, None)
            .and_then(|bytes| DrawingSettings::font_from_bytes(&bytes).ok())
        else {
            status_bar.buffer = "CouldNotLoadFont".to_string();
            return;
        };

        status_bar.drawing.set_font(font);
        status_bar.drawing.set_font_size(size);
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }
}
