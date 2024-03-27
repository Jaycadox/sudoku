use super::Item;

#[derive(Default)]
pub struct Padding;

impl Item for Padding {
    fn name(&self) -> &'static str {
        "Padding"
    }

    fn activated(
        &mut self,
        game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        // Format: {pad_start} {pad_target} {pad_speed}
        let mut args = status_bar.buffer.split_whitespace();
        let Some((pad_start, pad_target, pad_speed)) = args
            .next()
            .and_then(|x| x.parse::<f32>().ok())
            .and_then(|x| Some((x, args.next()?.parse::<f32>().ok()?)))
            .and_then(|(x, y)| Some((x, y, args.next()?.parse::<f32>().ok()?)))
        else {
            status_bar.buffer = "InvalidFormat".to_string();
            return;
        };

        status_bar.drawing.set_padding_start(pad_start);
        status_bar.drawing.set_padding_target(pad_target);
        status_bar.drawing.set_padding_speed(pad_speed);
        game.padding_progress = 0.0;
    }

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::None
    }
}
