use macroquad::{
    color::*,
    miniquad::window::screen_size,
    shapes::{draw_line, draw_rectangle},
};

use crate::{
    draw_helper::*,
    input_helper::{self, InputAction},
    sudoku_game::SudokuGame,
};

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
    fn activated(&mut self, game: &mut SudokuGame);
    fn update(&mut self, game: &mut SudokuGame) -> (String, Color);
    fn board_init(&mut self, game: &mut SudokuGame);
    fn status(&mut self) -> StatusBarItemStatus;
}

pub struct StatusBar {
    pub items: Vec<Box<dyn StatusBarItem>>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self { items: vec![] }
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
        for item in self.items.iter_mut() {
            item.board_init(game);
        }
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

        let key = input_helper::InputAction::get_last_input();
        if let Some(input_helper::InputAction::Function(x)) = key {
            if let Some(item) = self.items.get_mut(x as usize - 1) {
                item.activated(game);
            }
        }

        let item_count = self.items.len();
        for (i, item) in self.items.iter_mut().enumerate() {
            let last = i == item_count - 1;

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
            if !last {
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
        }
    }
}
