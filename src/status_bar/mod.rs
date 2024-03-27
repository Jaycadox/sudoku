use std::cmp::Ordering;
use std::{collections::VecDeque, fmt::Display, time::Instant};

use macroquad::{
    color::*,
    miniquad::window::screen_size,
    shapes::{draw_line, draw_rectangle},
};
use tracing::{debug, error, span, trace, warn, Level};

use crate::status_bar::shorthands::list::ShorthandList;
use crate::{
    draw_helper::*,
    input_helper::{InputAction, InputActionChar, InputActionContext},
    sudoku_game::{ResetSignal, SudokuGame},
};

use self::{add::BuiltinAdd, dummy::Dummy};

mod add;
mod background_image;
pub mod board_gen;
pub mod colour_overwrite;
pub mod cpu_solve;
mod dummy;
mod find;
mod font;
pub mod fps;
mod hard_reset;
pub mod on_board_init;
mod padding;
pub mod pencil_marks;
#[macro_use]
pub mod shorthands;
mod eval;

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

impl<'a> Display for StatusBarItemStatus<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StatusBarItemStatus::Ok(_) => "Ok",
                StatusBarItemStatus::Waiting => "Waiting",
                StatusBarItemStatus::Err => "Error",
            }
        )
    }
}

#[allow(dead_code)]
pub enum StatusBarDisplayMode {
    Normal,
    NameOnly,
    StatusOnly,
    None,
}

pub struct DrawHookData {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[allow(dead_code)]
pub enum StatusBarHookAction<T> {
    Continue(T),
    Stop,
}

pub trait StatusBarItem {
    fn name(&self) -> &'static str;
    fn activated(&mut self, game: &mut SudokuGame, status_bar: &mut StatusBar);

    fn update(&mut self, game: &mut SudokuGame, status_bar: &mut StatusBar) -> (String, Color) {
        let _ = game;
        (
            "".to_string(),
            status_bar.drawing.colour(AppColour::StatusBarItemOkay),
        )
    }

    fn board_init(&mut self, game: &mut SudokuGame, status_bar: &mut StatusBar) {
        let _ = status_bar;
        let _ = game;
    }

    fn status(&mut self) -> StatusBarItemStatus {
        StatusBarItemStatus::Ok(StatusBarItemOkData::None)
    }

    fn display_mode(&self) -> StatusBarDisplayMode {
        StatusBarDisplayMode::Normal
    }

    fn background_draw_hook(&self, data: &DrawHookData) -> StatusBarHookAction<()> {
        let _ = data;
        StatusBarHookAction::Continue(())
    }

    fn cell_text_draw_hook(
        &self,
        drawing: &DrawingSettings,
        game: &SudokuGame,
        index: u8,
        value: u8,
        data: &DrawHookData,
    ) -> StatusBarHookAction<()> {
        let _ = data;
        let _ = value;
        let _ = index;
        let _ = drawing;
        let _ = game;
        StatusBarHookAction::Continue(())
    }

    fn cell_text_colour_hook(
        &self,
        game: &SudokuGame,
        index: u8,
    ) -> Option<StatusBarHookAction<AppColour>> {
        let _index = index;
        let _game = game;
        None
    }

    fn shorthands(&self) -> Option<ShorthandList> {
        None
    }
}

pub struct StatusBar<'a> {
    time_started: Instant,
    items: Vec<Box<dyn StatusBarItem>>,
    pub buffer: String,
    pub drawing: &'a DrawingSettings,
    commands_queue: VecDeque<String>,
    current_command: Option<String>,
    command_history: Vec<String>,
    command_history_offset: usize,
}

impl<'a> StatusBar<'a> {
    pub fn new(drawing: &'a DrawingSettings) -> Self {
        Self {
            time_started: Instant::now(),
            items: vec![Box::<BuiltinAdd>::default()],
            buffer: String::new(),
            drawing,
            commands_queue: VecDeque::new(),
            current_command: None,
            command_history: Default::default(),
            command_history_offset: 0,
        }
    }

    pub fn add<T>(&mut self)
    where
        T: StatusBarItem + Default + 'static,
    {
        self.items.push(Box::<T>::default());
    }

    pub fn items(&self) -> impl Iterator<Item = &dyn StatusBarItem> {
        self.items.iter().map(|x| x.as_ref())
    }

    pub fn item_with_name(&mut self, name: &str) -> Option<&mut dyn StatusBarItem> {
        for item in self.items.iter_mut() {
            if item.name().to_lowercase() == name.to_lowercase() {
                return Some(item.as_mut());
            }
        }

        None
    }

    pub fn index_with_name(&self, name: &str) -> Option<usize> {
        for (i, item) in self.items.iter().enumerate() {
            if item.name().to_lowercase() == name.to_lowercase() {
                return Some(i);
            }
        }

        None
    }

    pub fn restart(&mut self, game: &mut SudokuGame) {
        let span = span!(Level::INFO, "BoardInit");
        let _enter = span.enter();

        let buffer = self.buffer.clone();

        let len = self.items.len();
        for idx in 0..len {
            let mut dummy_item: Box<dyn StatusBarItem + 'static> = Box::<Dummy>::default();

            let Some(item) = self.items.get_mut(idx) else {
                continue;
            };

            std::mem::swap(item, &mut dummy_item);
            let mut item = dummy_item;
            item.board_init(game, self);

            *self.items.get_mut(idx).unwrap() = item;
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
        let span = span!(Level::TRACE, "RunCommand");
        let _enter = span.enter();

        let buffer = self.buffer.clone();
        let og_command = buffer.clone();

        let mut command_words = buffer.split_whitespace();

        let command_name = command_words.next()?;

        let mut buffer = command_words.collect::<Vec<_>>().join(" ");

        let mut dummy_item: Box<dyn StatusBarItem + 'static> = Box::<Dummy>::default();
        let mut idx = self.index_with_name(command_name);

        if idx.is_none() {
            trace!("Regular command handler not found, attempting shorthands...");
            // Attempt shorthands
            for (i, item) in self.items.iter().enumerate() {
                if let Some(sh) = item.shorthands() {
                    if let Some(sh_out) = sh.apply_to_string(&og_command) {
                        trace!(
                            "Found shorthand for '{}', converting '{}' into {}",
                            item.name(),
                            og_command,
                            sh_out
                        );
                        buffer = sh_out;
                        idx = Some(i);
                        break;
                    }
                }
            }
        }

        let idx = idx?;

        let item = self.items.get_mut(idx)?;

        std::mem::swap(item, &mut dummy_item);
        let mut item = dummy_item;

        let before = buffer.clone();
        self.buffer.clone_from(&buffer);
        item.activated(game, self);
        let name_after = item.name();
        if before == self.buffer {
            self.buffer.clear();
        }

        self.command_history.push(og_command); // TODO: find if vec already contains item and move it back to the front

        *self.items.get_mut(idx).unwrap() = item;
        Some(name_after.to_string()) // calling item.activate() could hypothetically result in a
                                     // changing of name
    }

    fn process_queued_buffer_commands(&mut self, game: &mut SudokuGame) -> Result<(), String> {
        let span = span!(Level::TRACE, "QueuedCommands");
        let _enter = span.enter();

        if let Some(current_command) = &self.current_command.clone() {
            let span = span!(Level::TRACE, "Wait");
            let _enter = span.enter();

            if let Some(item) = self.item_with_name(current_command) {
                let item_name = item.name();
                match item.status() {
                    // Commands won't wait for eachother, this
                    // means the application loads faster, but I might in the future add a
                    // syntactic way of specifying this behaviour
                    StatusBarItemStatus::Waiting => {}
                    x => {
                        trace!(
                            "Command with name '{}' finished with status: {}",
                            item_name,
                            x
                        );
                        self.current_command = None;
                    }
                };
            }
        }
        while let Some(cmd) = self.commands_queue.pop_front() {
            let span = span!(Level::TRACE, "Run");
            let _enter = span.enter();

            trace!("Attempting to run: '{}'", cmd);
            self.buffer = cmd.to_string();
            if let Some(cmd_name) = self.buffer_entered(game) {
                trace!("Ran command with name '{}'", cmd_name);
                self.current_command = Some(cmd_name.clone());
                if let Some(item) = self.item_with_name(&cmd_name) {
                    match item.status() {
                        StatusBarItemStatus::Err => Err(cmd_name)?,
                        StatusBarItemStatus::Waiting => return Ok(()),
                        StatusBarItemStatus::Ok(_) => continue,
                    };
                } else {
                    warn!("Unable to query status of command: '{}'", cmd_name);
                    return Err(format!("ChangedCommandName: {cmd_name}"))?;
                }
            } else {
                error!("Unable to find handler for command: '{}'", cmd);
                return Err(format!("BadCommand: {cmd}"))?;
            }
        }

        Ok(())
    }

    pub fn enter_buffer_commands(&mut self, commands: &[&str]) {
        let span = span!(Level::TRACE, "EnterCommands");
        let _enter = span.enter();

        let mut commands = commands
            .iter()
            .flat_map(|x| {
                x.lines()
                    .flat_map(|y| y.split('&').map(|z| z.trim().to_string()))
            })
            .collect::<VecDeque<_>>();

        trace!(
            "Adding {} commands to queue: {:?}",
            commands.len(),
            commands
        );

        self.commands_queue.append(&mut commands);
    }

    pub fn draw(&mut self, game: &mut SudokuGame, drawing: &DrawingSettings) {
        self.process_inputs(game);
        self.render(game, drawing);
    }

    fn process_inputs(&mut self, game: &mut SudokuGame) {
        let span = span!(Level::INFO, "ProcessStatusBar");
        let _enter = span.enter();

        if let Err(message) = self.process_queued_buffer_commands(game) {
            self.buffer = message;
        };

        let mut i = 0;
        for raw_idx in 0..self.items.len() {
            let mut dummy_item: Box<dyn StatusBarItem + 'static> = Box::<Dummy>::default();

            let item = self.items.get_mut(raw_idx).unwrap();
            std::mem::swap(item, &mut dummy_item);
            let mut item = dummy_item;

            let display_mode = item.display_mode();
            let display = !matches!(display_mode, StatusBarDisplayMode::None);

            if display
                && InputAction::is_function_pressed(
                    i + 1,
                    if game.input.enter_buffer {
                        InputActionContext::Buffer
                    } else {
                        InputActionContext::Generic
                    },
                    &game.input,
                )
            {
                debug!(
                    "Activated status bar item via manual input: {}",
                    item.name()
                );
                game.input.enter_buffer = false;
                let before = self.buffer.clone();
                item.activated(game, self);
                if before == self.buffer {
                    self.buffer.clear();
                }
            }

            if display {
                i += 1;
            }

            self.items[raw_idx] = item;
        }

        if let Some(InputAction::ClearBuffer) = InputAction::get_last_input(
            if game.input.enter_buffer {
                InputActionContext::Buffer
            } else {
                InputActionContext::Generic
            },
            &game.input,
        ) {
            game.input.enter_buffer = !game.input.enter_buffer;
        }

        let mut ignore_next_input = false;
        let mut should_reset_history_pos = true;
        match InputAction::get_last_input(InputActionContext::Buffer, &game.input) {
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
                game.input.enter_buffer = false;
                self.enter_buffer_commands(&[&self.buffer.clone()]);
            }
            Some(InputAction::HardReset) => {
                game.reset_signalled = ResetSignal::Hard;
                return;
            }
            Some(InputAction::UpBuffer) => {
                should_reset_history_pos = false;
                if self.command_history_offset < self.command_history.len() {
                    self.command_history_offset += 1;
                }

                if let Some(item) = self
                    .command_history
                    .get(self.command_history.len() - self.command_history_offset)
                {
                    self.buffer = item.to_string();
                }
            }
            Some(InputAction::DownBuffer) => {
                should_reset_history_pos = false;
                match self.command_history_offset.cmp(&1) {
                    Ordering::Greater => {
                        self.command_history_offset -= 1;
                    }
                    Ordering::Equal => {
                        self.buffer = String::new();
                        self.command_history_offset = 0;
                    }
                    _ => {}
                };

                if let Some(item) = self
                    .command_history
                    .get(self.command_history.len() - self.command_history_offset)
                {
                    self.buffer = item.to_string();
                }
            }
            _ => {
                should_reset_history_pos = false;
            }
        };

        // Action performed whereby the position didn't change, meaning the current scroll state was discarded
        if should_reset_history_pos {
            self.command_history_offset = 0;
        }

        let key = InputAction::get_last_input_char(InputActionContext::Buffer, &game.input);

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
    }

    fn render(&mut self, game: &mut SudokuGame, drawing: &DrawingSettings) {
        let span = span!(Level::INFO, "RenderStatusBar");
        let _enter = span.enter();

        let (width, height) = screen_size();
        let status_bar_height = get_status_bar_height();

        let (start_x, start_y) = (0.0, height - status_bar_height);
        let (bar_width, bar_height) = (width, status_bar_height);

        draw_rectangle(
            start_x,
            start_y,
            bar_width,
            bar_height,
            drawing.colour(AppColour::StatusBar),
        );

        let mut cursor_x = 20.0;

        let font_size = status_bar_height * 0.9;
        let cursor_y = start_y;
        let mut visible_index = 1;
        for i in 0..self.items.len() {
            let item = self.items.get_mut(i).unwrap();

            let mut dummy_item: Box<dyn StatusBarItem + 'static> = Box::<Dummy>::default();

            let display_mode = item.display_mode();

            std::mem::swap(item, &mut dummy_item);
            let mut item = dummy_item;

            if matches!(item.display_mode(), StatusBarDisplayMode::None) {
                let _ = item.update(game, self);
                self.items[i] = item;
                continue;
            }

            let font_color = if InputAction::is_function_down(
                visible_index as u8,
                InputActionContext::Generic,
                &game.input,
            ) {
                drawing.colour(AppColour::StatusBarItemSelected)
            } else {
                drawing.colour(AppColour::StatusBarItem)
            };

            visible_index += 1;
            let (text, color) = item.update(game, self);

            if matches!(
                display_mode,
                StatusBarDisplayMode::Normal | StatusBarDisplayMode::NameOnly
            ) {
                let (suffix, font_color) = if let StatusBarDisplayMode::Normal = display_mode {
                    (" ::", font_color)
                } else {
                    ("", color)
                };
                let bounds = draw_and_measure_text(
                    drawing,
                    &format!("{}{}", item.name(), suffix),
                    cursor_x,
                    cursor_y,
                    font_size,
                    font_color,
                    (None, Some(status_bar_height)),
                );
                cursor_x += bounds.0 + 8.0;
            }
            if matches!(
                display_mode,
                StatusBarDisplayMode::Normal | StatusBarDisplayMode::StatusOnly
            ) {
                let bounds = draw_and_measure_text(
                    drawing,
                    &text,
                    cursor_x,
                    cursor_y,
                    font_size,
                    color,
                    (None, Some(status_bar_height)),
                );
                cursor_x += bounds.0;
            }

            cursor_x += 16.0;
            draw_line(
                cursor_x,
                start_y,
                cursor_x,
                height,
                get_normal_line_width(),
                drawing.colour(AppColour::StatusBarSeparator),
            );
            cursor_x += 16.0;
            self.items[i] = item;
        }

        // Now that each status bar item has been drawn, we can start to draw the buffer input

        let color = if game.input.enter_buffer {
            drawing.colour(AppColour::StatusBarBufferEdit)
        } else {
            drawing.colour(AppColour::StatusBarItem)
        };

        let bounds = draw_and_measure_text(
            drawing,
            &format!("> {}", self.buffer),
            cursor_x,
            cursor_y,
            font_size,
            color,
            (None, Some(status_bar_height)),
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
            drawing.colour(AppColour::StatusBarSeparator),
        );

        cursor_x += 16.0;

        // Next, if there's a command queue, we need to display it
        let mut commands_queue = self.commands_queue.iter().cloned().collect::<Vec<_>>();
        if let Some(active_command) = self.current_command.as_ref() {
            commands_queue.push(format!("{}...", active_command));
        }
        commands_queue.reverse();

        let queue_string = format!("{}: [{}]", commands_queue.len(), commands_queue.join(", "));
        if !commands_queue.is_empty() {
            let _gobounds = draw_and_measure_text(
                drawing,
                &queue_string,
                cursor_x,
                cursor_y,
                font_size,
                drawing.colour(AppColour::StatusBarItem),
                (None, Some(status_bar_height)),
            );

            //cursor_x += bounds.0 + 3.0;
        }
    }
}
