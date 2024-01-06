use std::collections::HashSet;

use macroquad::prelude::*;
use macroquad::{miniquad::window::screen_size, text};
mod sudoku_game;
mod sudoku_solver;
mod task_status;
use sudoku_game::SudokuGame;
use sudoku_solver::SolveTask;
use task_status::{GetTask, TaskStatus};

const STATUS_BAR_PERCENTAGE: f32 = 0.03;
const NORMAL_LINE_PERCENTAGE: f32 = 0.001;
const BOX_LINE_PERCENTAGE: f32 = 0.004;

fn get_status_bar_height() -> f32 {
    screen_height() * STATUS_BAR_PERCENTAGE
}

fn get_normal_line_width() -> f32 {
    (screen_height() * NORMAL_LINE_PERCENTAGE).max(2.0) as u32 as f32
}

fn get_box_line_width() -> f32 {
    (screen_height() * BOX_LINE_PERCENTAGE).max(3.0) as u32 as f32
}

struct DrawingSettings {
    font: Font,
}

impl Default for DrawingSettings {
    fn default() -> Self {
        Self {
            font: text::load_ttf_font_from_bytes(include_bytes!("./TWN19.ttf")).unwrap(),
        }
    }
}

enum InputAction {
    NumberEntered(u8),
    Function(u8),
    Reset,
    Clear,
    AutoPlay,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

impl TryFrom<KeyCode> for InputAction {
    type Error = String;
    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        Ok(match value {
            KeyCode::Key1 => InputAction::NumberEntered(1),
            KeyCode::Key2 => InputAction::NumberEntered(2),
            KeyCode::Key3 => InputAction::NumberEntered(3),
            KeyCode::Key4 => InputAction::NumberEntered(4),
            KeyCode::Key5 => InputAction::NumberEntered(5),
            KeyCode::Key6 => InputAction::NumberEntered(6),
            KeyCode::Key7 => InputAction::NumberEntered(7),
            KeyCode::Key8 => InputAction::NumberEntered(8),
            KeyCode::Key9 => InputAction::NumberEntered(9),
            KeyCode::Backspace | KeyCode::Key0 | KeyCode::Delete => InputAction::Clear,
            KeyCode::Space => InputAction::AutoPlay,
            KeyCode::Tab => InputAction::Reset,
            KeyCode::F1 => InputAction::Function(1),
            KeyCode::F2 => InputAction::Function(2),
            KeyCode::F3 => InputAction::Function(3),
            KeyCode::F4 => InputAction::Function(4),
            KeyCode::F5 => InputAction::Function(5),
            KeyCode::F6 => InputAction::Function(6),
            KeyCode::F7 => InputAction::Function(7),
            KeyCode::F8 => InputAction::Function(8),
            KeyCode::F9 => InputAction::Function(9),
            KeyCode::F10 => InputAction::Function(10),
            KeyCode::F11 => InputAction::Function(11),
            KeyCode::F12 => InputAction::Function(12),
            KeyCode::W | KeyCode::Up => InputAction::MoveUp,
            KeyCode::A | KeyCode::Left => InputAction::MoveLeft,
            KeyCode::S | KeyCode::Down => InputAction::MoveDown,
            KeyCode::D | KeyCode::Right => InputAction::MoveRight,
            _ => Err("Not a recognised key".to_string())?,
        })
    }
}

fn draw_sudoku(game: &mut SudokuGame, drawing: &DrawingSettings) {
    let (mut width, mut height) = screen_size();
    height -= get_status_bar_height();
    let padding = 30.0;
    let s_padding = padding / 2.0;
    width -= padding; // padding
    height -= padding; // padding

    let rect_size = f32::min(width / 9.0, height / 9.0);

    let x_pad = (width - (9.0 * rect_size)) / 2.0;
    let y_pad = (height - (9.0 * rect_size)) / 2.0;

    let raw_mouse_pos = mouse_position();

    let mut mouse_pos: Option<(u32, u32)> = None;

    if raw_mouse_pos.0 > x_pad + s_padding
        && raw_mouse_pos.1 > y_pad + s_padding
        && raw_mouse_pos.0 < x_pad + s_padding + (rect_size * 9.0)
        && raw_mouse_pos.1 < y_pad + s_padding + (rect_size * 9.0)
    {
        mouse_pos = Some((
            ((raw_mouse_pos.0 - x_pad - s_padding) / (rect_size)) as u32,
            ((raw_mouse_pos.1 - y_pad - s_padding) / (rect_size)) as u32,
        ));
    }

    let highlight_cells = match game.selected_cell {
        Some(pos) => game.get_cells_which_see_number_at_pos(pos),
        _ => vec![],
    };

    let mut key = get_last_key_pressed().and_then(|key| InputAction::try_from(key).ok());

    if let Some((mx, my)) = mouse_pos {
        let mut change_selected_to_cursor = false;

        if let Some(ref value) = key {
            if let Some((sx, sy)) = game.selected_cell {
                let cell_value = game.cells[(sy as usize, sx as usize)];
                if cell_value != 0 && matches!(value, InputAction::NumberEntered(_)) {
                    change_selected_to_cursor = true;
                }

                if cell_value == 0 && matches!(value, InputAction::Clear) {
                    change_selected_to_cursor = true;
                }
            } else {
                change_selected_to_cursor = true;
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) || change_selected_to_cursor {
            game.selected_cell = Some((mx, my));
        }
    }
    for (y, row) in game.clone().rows().iter().enumerate() {
        let y = y as f32;
        for (x, cell) in row.iter().enumerate() {
            let x = x as f32;
            let (start_x, start_y) = (
                x_pad + s_padding + (x * rect_size),
                y_pad + s_padding + (y * rect_size),
            );

            let unradified = game.unradified.contains(&(y as u8 * 9 + x as u8));

            if highlight_cells.contains(&(y as u32 * 9 + x as u32)) {
                draw_rectangle(
                    start_x,
                    start_y,
                    rect_size,
                    rect_size,
                    Color::new(1.00, 1.00, 1.00, 0.20),
                );
            }

            if let Some((mx, my)) = mouse_pos {
                if x == mx as f32 && y == my as f32 {
                    draw_rectangle(start_x, start_y, rect_size, rect_size, DARKGRAY);
                }
            }

            if let Some((sx, sy)) = game.selected_cell {
                if x == sx as f32 && y == sy as f32 {
                    draw_rectangle(
                        start_x,
                        start_y,
                        rect_size,
                        rect_size,
                        Color::new(1.00, 1.00, 1.00, 0.35),
                    );
                    if unradified {
                        if let Some(ref mut value) = key {
                            if matches!(value, InputAction::AutoPlay) && *cell == 0 {
                                // Auto value
                                let mut nums = HashSet::new();
                                *value = InputAction::NumberEntered(0);
                                {
                                    // By box
                                    let box_cord = (x as u32 / 3, y as u32 / 3);
                                    let box_id = box_cord.1 * 3 + box_cord.0;
                                    if let Some(b) = game.boxes().iter().nth(box_id as usize) {
                                        for num in b {
                                            if *num != 0 {
                                                nums.insert(*num);
                                            }
                                        }
                                    }
                                }

                                {
                                    // By row
                                    if let Some(b) = game.rows().get(y as usize) {
                                        for num in b {
                                            if *num != 0 {
                                                nums.insert(*num);
                                            }
                                        }
                                    }
                                }

                                {
                                    // By col
                                    if let Some(b) = game.cols().get(x as usize) {
                                        for num in b {
                                            if *num != 0 {
                                                nums.insert(*num);
                                            }
                                        }
                                    }
                                }

                                if nums.len() == 8 {
                                    for i in 1..=9 {
                                        if !nums.contains(&i) {
                                            *value = InputAction::NumberEntered(i);
                                            break;
                                        }
                                    }
                                }
                            }
                            match value {
                                InputAction::NumberEntered(_) | InputAction::Clear => {
                                    game.cells[(y as usize, x as usize)] = match value {
                                        InputAction::NumberEntered(num) => *num,
                                        InputAction::Clear => 0,
                                        _ => panic!("tried to place invalid cell input"),
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if *cell != 0 {
                let number_size = measure_text(&format!("{cell}"), None, rect_size as u16, 1.0);
                let text_col = if unradified {
                    if let TaskStatus::Done(solved) = game.get_task_status() {
                        let solved_cell = solved.cells[(y as usize, x as usize)];
                        let our_cell = game.cells[(y as usize, x as usize)];
                        if solved_cell == our_cell {
                            Color::new(0.60, 0.60, 1.00, 1.00)
                        } else {
                            Color::new(1.00, 0.60, 0.60, 1.00)
                        }
                    } else {
                        Color::new(0.60, 0.60, 0.60, 1.00)
                    }
                } else {
                    WHITE
                };

                let _ = draw_and_measure_text(
                    drawing,
                    &format!("{cell}"),
                    start_x + rect_size / 2.0 - number_size.width / 2.0,
                    start_y + rect_size / 2.0 - -number_size.height / 2.0,
                    rect_size,
                    text_col,
                );
            }
            draw_rectangle_lines(
                start_x,
                start_y,
                rect_size,
                rect_size,
                get_normal_line_width(),
                GRAY,
            );
        }
    }
    let cgame = game.clone();
    let boxes = cgame.boxes();
    for (index, _) in boxes.iter().enumerate() {
        let x = (index % boxes.shape()[1]) as f32 * 3.0;
        let y = (index / boxes.shape()[1]) as f32 * 3.0;
        let (start_x, start_y) = (
            x_pad + s_padding + (x * rect_size),
            y_pad + s_padding + (y * rect_size),
        );
        draw_rectangle_lines(
            start_x,
            start_y,
            rect_size * 3.0,
            rect_size * 3.0,
            get_box_line_width(),
            WHITE,
        );
    }

    if let Some((sx, sy)) = &mut game.selected_cell {
        if let Some(ref key) = key {
            match key {
                InputAction::MoveUp => {
                    if is_key_down(KeyCode::LeftShift) {
                        if *sy < 3 {
                            *sy += 6;
                        } else {
                            *sy -= 3;
                        }
                    } else if *sy == 0 {
                        *sy = 8;
                    } else {
                        *sy -= 1;
                    }
                }
                InputAction::MoveDown => {
                    if is_key_down(KeyCode::LeftShift) {
                        if *sy > 5 {
                            *sy -= 6;
                        } else {
                            *sy += 3;
                        }
                    } else if *sy == 8 {
                        *sy = 0;
                    } else {
                        *sy += 1;
                    }
                }
                InputAction::MoveRight => {
                    if is_key_down(KeyCode::LeftShift) {
                        if *sx > 5 {
                            *sx -= 6;
                        } else {
                            *sx += 3;
                        }
                    } else if *sx == 8 {
                        *sx = 0;
                    } else {
                        *sx += 1;
                    }
                }
                InputAction::MoveLeft => {
                    if is_key_down(KeyCode::LeftShift) {
                        if *sx < 3 {
                            *sx += 6;
                        } else {
                            *sx -= 3;
                        }
                    } else if *sx == 0 {
                        *sx = 8;
                    } else {
                        *sx -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(ref key_pressed) = key {
        match key_pressed {
            InputAction::Reset => {
                *game = SudokuGame::new();
            }
            InputAction::Function(1) => {
                if is_key_down(KeyCode::LeftShift) {
                    game.solve_task = Some(SolveTask::new(game));
                } else if let TaskStatus::Done(solved_game) = game.get_task_status() {
                    game.cells = solved_game.clone().cells;
                }
            }
            _ => {}
        }
    }
}

fn draw_and_measure_text(
    drawing: &DrawingSettings,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    color: Color,
) -> (f32, f32) {
    let params = TextParams {
        font: Some(&drawing.font),
        color,
        font_size: font_size as u16,
        ..Default::default()
    };
    draw_text_ex(text, x, y, params);
    let dim = measure_text(text, Some(&drawing.font), font_size as u16, 1.0);
    (dim.width, dim.height)
}

fn draw_status_bar(game: &mut SudokuGame, drawing: &DrawingSettings) {
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

    let bounds = draw_and_measure_text(
        drawing,
        "CpuSolver :: ",
        cursor_x,
        cursor_y,
        font_size,
        WHITE,
    );
    cursor_x += bounds.0 + 5.0;
    match game.get_task_status() {
        TaskStatus::Done(_) => {
            draw_and_measure_text(drawing, "done  ", cursor_x, cursor_y, font_size, GREEN);
        }
        TaskStatus::Waiting(start_time) => {
            draw_and_measure_text(
                drawing,
                &format!(
                    "{:.3}s",
                    std::time::Instant::now()
                        .duration_since(*start_time)
                        .as_secs_f32()
                ),
                cursor_x,
                cursor_y,
                font_size,
                YELLOW,
            );
        }
        TaskStatus::Failed => {
            draw_and_measure_text(drawing, "failed", cursor_x, cursor_y, font_size, RED);
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Sudoku".to_owned(),
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let drawing = DrawingSettings::default();
    let mut game = SudokuGame::new();
    let mut i = 0;
    loop {
        i += 1;
        if i == 100 {
            i = 0;
            println!("fps: {}", get_fps());
        }
        clear_background(BLACK);
        draw_sudoku(&mut game, &drawing);
        draw_status_bar(&mut game, &drawing);
        next_frame().await;
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
}
