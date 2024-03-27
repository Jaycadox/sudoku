use std::str::FromStr;

use macroquad::color::Color;
use tracing::{error, span, trace, Level};

use crate::draw_helper::AppColour;

use super::Item;

#[derive(Default)]
pub struct ColourOverwrite;

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8, u8)> {
    if hex.len() == 9 && hex.starts_with('#') {
        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
        let a = u8::from_str_radix(&hex[7..9], 16).ok()?;
        Some((r, g, b, a))
    } else {
        None
    }
}

impl Item for ColourOverwrite {
    fn name(&self) -> &'static str {
        "ColourOverwrite"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let span = span!(Level::INFO, "ColourOverwriteActivate");
        let _enter = span.enter();

        let mut args = status_bar.buffer.split_whitespace();
        let Some(app_colour_string) = args.next() else {
            error!("App colour not provided");
            return;
        };

        let Ok(app_colour) = AppColour::from_str(app_colour_string) else {
            error!("Invalid app colour name: {}", app_colour_string);
            return;
        };

        let Some(hex_string) = args.next() else {
            error!("App hex colour provided");
            return;
        };

        trace!("Attempting to parse: '{}'", hex_string);
        let Some((r, g, b, a)) = parse_hex_color(hex_string.trim()) else {
            error!("Failed to parse hex colour: '{}'", hex_string);
            return;
        };

        status_bar
            .drawing
            .add_override(app_colour, Color::from_rgba(r, g, b, a));
    }

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::None
    }
}
