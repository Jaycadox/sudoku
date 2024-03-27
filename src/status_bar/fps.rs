use std::time::{Duration, Instant};

use macroquad::time::get_fps;
use tracing::{debug, error, span, trace, Level};

use crate::draw_helper::AppColour;
use crate::shorthand;
use crate::status_bar::shorthands::list::List;

use super::{Item, StatusBar};

#[derive(Default)]
pub struct Fps {
    target: Option<usize>,
    last_frame: Option<Instant>,
    last_delay: f64,
}

impl Item for Fps {
    fn name(&self) -> &'static str {
        "Fps"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut StatusBar,
    ) {
        let span = span!(Level::INFO, "FpsTargetActivated");
        let _enter = span.enter();

        trace!("Attempting to parse: {}", status_bar.buffer);

        if status_bar.buffer.is_empty() {
            debug!("Buffer is empty, removing FPS target...");
            self.target = None;
            return;
        }

        let Ok(target) = status_bar.buffer.parse::<usize>() else {
            error!("Could not parse FPS target");
            status_bar.buffer = "Fps: invalid target".to_string();
            return;
        };

        debug!("Set FPS target to: {}", target);
        self.target = Some(target);
    }

    fn update(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut StatusBar,
    ) -> (String, macroquad::prelude::Color) {
        if self.last_frame.is_none() {
            self.last_frame = Some(Instant::now());

            return (
                format!("{:<4}", get_fps()),
                status_bar.drawing.colour(AppColour::StatusBarItem),
            );
        }

        let indicator = if let (Some(last_frame), Some(target)) = (self.last_frame, self.target) {
            let total_frame_time_ms = Instant::now().duration_since(last_frame).as_millis();
            let frame_time_ms =
                (i128::try_from(total_frame_time_ms).unwrap() - self.last_delay as i128) as f64;
            let target_frame_time_ms = 1000.0 / target as f64;
            if target_frame_time_ms > frame_time_ms {
                let diff = target_frame_time_ms;
                self.last_delay = diff - target_frame_time_ms;
                std::thread::sleep(Duration::from_secs_f64(diff / 1000.0));
            } else {
                self.last_delay = 0.0;
            }
            "*"
        } else {
            ""
        };

        self.last_frame = Some(Instant::now());
        let output = format!("{}{}", get_fps(), indicator);
        (
            format!("{output:<4}"),
            status_bar.drawing.colour(AppColour::StatusBarItem),
        )
    }

    fn shorthands(&self) -> Option<List> {
        shorthand![(r"^(\d+)fps$", "$1")]
    }
}
