use crate::{
    draw_helper::{draw_text_in_bounds, AppColour, DrawingSettings},
    sudoku_game::SudokuGame,
};

use super::{cpu_solve, DrawHookData, Item};

pub struct PencilMarks {
    num_max: u8,
}

impl Default for PencilMarks {
    fn default() -> Self {
        Self { num_max: 2 }
    }
}

impl Item for PencilMarks {
    fn name(&self) -> String {
        "PencilMarks".to_string()
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let Ok(num_str) = status_bar.buffer.trim().parse::<u8>() else {
            status_bar.buffer = "CouldNotParse".to_string();
            return;
        };

        self.num_max = num_str;
    }

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::None
    }

    fn cell_text_draw_hook(
        &self,
        drawing: &DrawingSettings,
        game: &SudokuGame,
        index: u8,
        value: u8,
        data: &DrawHookData,
    ) -> super::HookAction<()> {
        let in_sight = cpu_solve::get_occupied_numbers_at_cell(
            game,
            SudokuGame::idx_pos_to_xy(u32::from(index), game.cells.shape()[1] as u32),
        );
        let mut not_in_sight = vec![];
        for i in 1..=9 {
            if !in_sight[i - 1] {
                not_in_sight.push(i);
            }
        }

        let padding = data.w / 12.0;
        if not_in_sight.len() <= self.num_max as usize && value == 0 {
            let s = not_in_sight
                .chunks(5)
                .map(|chk| chk.iter().map(ToString::to_string).collect::<String>())
                .collect::<Vec<String>>();

            let mut y_cursor = data.y + padding;
            for line in s {
                draw_text_in_bounds(
                    drawing,
                    &line,
                    data.x + padding,
                    y_cursor,
                    data.w / 2.5,
                    drawing.colour(AppColour::BoardUnknownCell),
                    (None, None),
                );
                y_cursor += (data.w / 5.0) + padding;
            }
        }

        super::HookAction::Continue(())
    }
}
