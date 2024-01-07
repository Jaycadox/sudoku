use std::time::Instant;

use macroquad::{
    color::*,
    miniquad::{window::screen_size, KeyCode},
    shapes::{draw_line, draw_rectangle},
};

use crate::{
    draw_helper::*,
    input_helper::{self, InputAction, InputActionChar, InputActionContext},
    sudoku_game::SudokuGame,
};

pub mod board_gen;
pub mod cpu_solver;
pub mod fps;

#[allow(dead_code)]
pub enum StatusBarItemOkData<'a> {
    Game(&'a SudokuGame),
    None,
}

pub enum StatusBarItemStatus<'a> {
    Ok(StatusBarItemOkData<'a>),
    Waiting,
    Err,
}

pub trait StatusBarItem {
    fn name(&self) -> &'static str;
    fn activated(&mut self, game: &mut SudokuGame, buffer: &mut String);
    fn update(&mut self, game: &mut SudokuGame) -> (String, Color);
    fn board_init(&mut self, game: &mut SudokuGame, buffer: &mut String);
    fn status(&mut self) -> StatusBarItemStatus;
}

pub struct StatusBar {
    time_started: Instant,
    pub items: Vec<Box<dyn StatusBarItem>>,
    pub buffer: String,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            time_started: Instant::now(),
            items: vec![],
            buffer: String::new(),
        }
    }

    pub fn add<T>(&mut self)
    where
        T: StatusBarItem + Default + 'static,
    {
        self.items.push(Box::<T>::default());
    }

    pub fn item_with_name(&mut self, name: &str) -> Option<&mut dyn StatusBarItem> {
        for item in self.items.iter_mut() {
            if item.name() == name {
                return Some(item.as_mut());
            }
        }

        None
    }

    pub fn restart(&mut self, game: &mut SudokuGame) {
        let mut buffer = self.buffer.clone();
        for item in self.items.iter_mut() {
            item.board_init(game, &mut buffer);
        }
        self.buffer = buffer;
    }

    fn should_draw_buffer_line(&self) -> bool {
        let duration = Instant::now().duration_since(self.time_started);
        let duration_secs = duration.as_secs_f32();
        let num_half_secs = duration_secs / 0.5;
        let whole_num_half_secs = num_half_secs as u32;
        whole_num_half_secs % 2 == 0
    }

    pub fn draw(&mut self, game: &mut SudokuGame, drawing: &DrawingSettings) {
        let (width, height) = screen_size();
        let status_bar_height = get_status_bar_height();

        let (start_x, start_y) = (0.0, height - status_bar_height);
        let (bar_width, bar_height) = (width, status_bar_height);

        draw_rectangle(
            start_x,
            start_y,
            bar_width,
            bar_height,
            Color::from_rgba(20, 20, 20, 255),
        );

        let mut cursor_x = 20.0;

        let font_size = status_bar_height * 0.9;
        let cursor_y = start_y + (font_size / 1.25);

        let mut buffer = self.buffer.clone();

        let key = input_helper::InputAction::get_last_input(InputActionContext::Generic);
        if let Some(input_helper::InputAction::Function(x)) = key {
            if let Some(item) = self.items.get_mut(x as usize - 1) {
                let before = buffer.clone();
                item.activated(game, &mut buffer);
                if before == buffer {
                    buffer.clear();
                }
            }
        }

        for (i, item) in self.items.iter_mut().enumerate() {
            let font_color = if InputAction::is_function_down(i as u8 + 1) {
                Color::from_rgba(200, 200, 255, 255)
            } else {
                WHITE
            };

            let bounds = draw_and_measure_text(
                drawing,
                &format!("{} :: ", item.name()),
                cursor_x,
                cursor_y,
                font_size,
                font_color,
            );
            cursor_x += bounds.0;
            let (text, color) = item.update(game);
            let bounds =
                draw_and_measure_text(drawing, &text, cursor_x, cursor_y, font_size, color);
            cursor_x += bounds.0;
            cursor_x += 8.0;
            draw_line(
                cursor_x,
                start_y,
                cursor_x,
                height,
                get_normal_line_width(),
                Color::from_rgba(30, 30, 30, 255),
            );
            cursor_x += 16.0;
        }

        self.buffer = buffer;

        // Now that each status bar item has been drawn, we can start to draw the buffer input
        let mut ignore_next_input = false;
        match InputAction::get_last_input(InputActionContext::Buffer) {
            Some(InputAction::ClearBuffer) => {
                self.buffer.clear();
            }
            Some(InputAction::PasteBuffer) => {
                if let Ok(txt) = arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
                    self.buffer.push_str(&txt);
                    ignore_next_input = true;
                }
            }
            _ => {}
        };

        let key = InputAction::get_last_input_char(InputActionContext::Buffer);

        match key {
            Some(InputActionChar::Char(c)) => {
                self.time_started = Instant::now();
                if !ignore_next_input {
                    self.buffer.push(c)
                }
            }
            Some(InputActionChar::Backspace) => {
                let _ = self.buffer.pop();
            }
            Some(InputActionChar::Clear) => self.buffer.clear(),
            None => {}
        };

        let color = if InputAction::is_key_down(KeyCode::LeftControl, InputActionContext::Buffer) {
            YELLOW
        } else {
            WHITE
        };

        let bounds = draw_and_measure_text(
            drawing,
            &format!(">{}", self.buffer),
            cursor_x,
            cursor_y,
            font_size,
            color,
        );

        cursor_x += bounds.0 + 3.0;

        if self.should_draw_buffer_line() {
            let line_padding = status_bar_height * 0.25;
            draw_line(
                cursor_x,
                start_y + line_padding,
                cursor_x,
                height - line_padding,
                get_normal_line_width(),
                color,
            );
        }
    }
}
