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
pub mod cpu_solve;
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
    commands_queue: Vec<String>,
    current_command: Option<String>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            time_started: Instant::now(),
            items: vec![],
            buffer: String::new(),
            commands_queue: vec![],
            current_command: None,
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

    fn buffer_entered(&mut self, game: &mut SudokuGame) -> Option<String> {
        let buffer = self.buffer.clone();
        let mut command_words = buffer.split_whitespace();

        let Some(command_name) = command_words.next() else {
            return None;
        };

        let Some(item) = self.item_with_name(command_name) else {
            return None;
        };

        let mut buffer = command_words.collect::<Vec<_>>().join(" ");

        let before = buffer.clone();
        item.activated(game, &mut buffer);
        let name_after = item.name();

        if before == buffer {
            buffer.clear();
        }

        self.buffer = buffer;

        Some(name_after.to_string()) // calling item.activate() could hypothetically result in a
                                     // changing of name
    }

    fn process_queued_buffer_commands(&mut self, game: &mut SudokuGame) -> Result<(), String> {
        if let Some(current_command) = &self.current_command.clone() {
            if let Some(item) = self.item_with_name(current_command) {
                match item.status() {
                    StatusBarItemStatus::Waiting => return Ok(()),
                    _ => {
                        self.current_command = None;
                    }
                };
            }
        }

        while let Some(cmd) = self.commands_queue.pop() {
            self.buffer = cmd.to_string();
            if let Some(cmd_name) = self.buffer_entered(game) {
                self.current_command = Some(cmd_name.clone());
                if let Some(item) = self.item_with_name(&cmd_name) {
                    match item.status() {
                        StatusBarItemStatus::Err => Err(cmd_name)?,
                        StatusBarItemStatus::Waiting => return Ok(()),
                        StatusBarItemStatus::Ok(_) => continue,
                    };
                } else {
                    return Err("ChangedCommandName".to_string())?;
                }
            } else {
                return Err("BadCommandName".to_string())?;
            }
        }

        Ok(())
    }

    pub fn enter_buffer_commands(&mut self, commands: &[&str]) {
        self.commands_queue.append(
            &mut commands
                .iter()
                .flat_map(|x| {
                    x.lines()
                        .flat_map(|y| y.split('&').map(|z| z.trim().to_string()).rev())
                })
                .collect(),
        );
    }

    pub fn draw(&mut self, game: &mut SudokuGame, drawing: &DrawingSettings) {
        if let Err(message) = self.process_queued_buffer_commands(game) {
            if self.buffer.is_empty() {
                self.buffer = message;
            }
        }

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
            let font_color =
                if InputAction::is_function_down(i as u8 + 1, InputActionContext::Generic) {
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
            Some(InputAction::EnterBuffer) => {
                self.enter_buffer_commands(&[&self.buffer.clone()]);
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
            &format!("> {}", self.buffer),
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

        cursor_x += get_normal_line_width() + 16.0;

        draw_line(
            cursor_x,
            start_y,
            cursor_x,
            height,
            get_normal_line_width(),
            Color::from_rgba(30, 30, 30, 255),
        );

        cursor_x += 16.0;

        // Next, if there's a command queue, we need to display it
        let mut commands_queue = self.commands_queue.clone();
        if let Some(active_command) = self.current_command.as_ref() {
            commands_queue.push(format!("{}...", active_command));
        }
        commands_queue.reverse();

        let queue_string = format!("{}: [{}]", commands_queue.len(), commands_queue.join(", "));
        if !commands_queue.is_empty() {
            let _gobounds =
                draw_and_measure_text(drawing, &queue_string, cursor_x, cursor_y, font_size, WHITE);

            //cursor_x += bounds.0 + 3.0;
        }
    }
}
