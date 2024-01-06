use macroquad::{color::*, miniquad::window::screen_size, shapes::draw_rectangle};

use crate::{draw_helper::*, sudoku_game::SudokuGame};

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
        self.items.push(Box::new(T::default()));
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

        for item in self.items.iter_mut() {
            let bounds = draw_and_measure_text(
                drawing,
                &format!("{} :: ", item.name()),
                cursor_x,
                cursor_y,
                font_size,
                WHITE,
            );
            cursor_x += bounds.0 + 5.0;

            let (text, color) = item.update(game);
            let bounds =
                draw_and_measure_text(drawing, &text, cursor_x, cursor_y, font_size, color);
            cursor_x += bounds.0 + 5.0;
        }
    }
}
