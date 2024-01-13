use std::time::{Duration, Instant};

use macroquad::{color::WHITE, time::get_fps};

use super::{StatusBar, StatusBarItem};

#[derive(Default)]
pub struct Fps {
    target: Option<usize>,
    last_frame: Option<Instant>,
    last_delay: f64,
}

impl StatusBarItem for Fps {
    fn name(&self) -> &'static str {
        "Fps"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut StatusBar,
    ) {
        if status_bar.buffer.is_empty() {
            self.target = None;
            return;
        }

        let Ok(target) = status_bar.buffer.parse::<usize>() else {
            status_bar.buffer = "Fps: invalid target".to_string();
            return;
        };

        self.target = Some(target);
    }

    fn update(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
    ) -> (String, macroquad::prelude::Color) {
        if self.last_frame.is_none() {
            self.last_frame = Some(Instant::now());

            return (format!("{:<4}", get_fps()), WHITE);
        }

        let indicator = if let (Some(last_frame), Some(target)) = (self.last_frame, self.target) {
            let total_frame_time_ms = Instant::now().duration_since(last_frame).as_millis();
            let frame_time_ms = (total_frame_time_ms as i128 - self.last_delay as i128) as f64;
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
        (format!("{:<4}", output), WHITE)
    }
}
