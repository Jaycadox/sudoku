mod draw_helper;
mod input_helper;
mod status_bar;
mod sudoku_game;
mod task_status;

use draw_helper::*;
use input_helper::*;
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use status_bar::board_gen::BoardGen;
use status_bar::cpu_solve::*;
use status_bar::fps::Fps;
use status_bar::{StatusBar, StatusBarItemStatus};
use std::collections::HashSet;
use sudoku_game::SudokuGame;

fn draw_sudoku(game: &mut SudokuGame, drawing: &DrawingSettings, status_bar: &mut StatusBar) {
    if game.reset_signalled {
        status_bar.restart(game);
        game.reset_signalled = false;
    }

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

    let mut key = InputAction::get_last_input(InputActionContext::Generic);

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
                    let solver_value = status_bar.item_with_name("CpuSolve").map(|x| x.status());
                    match solver_value {
                        Some(StatusBarItemStatus::Ok(status_bar::StatusBarItemOkData::Game(
                            solved,
                        ))) => {
                            let solved_cell = solved.cells[(y as usize, x as usize)];
                            let our_cell = game.cells[(y as usize, x as usize)];
                            if solved_cell == our_cell {
                                Color::new(0.60, 0.60, 1.00, 1.00)
                            } else {
                                Color::new(1.00, 0.60, 0.60, 1.00)
                            }
                        }
                        _ => Color::new(0.60, 0.60, 0.60, 1.00),
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
                    if InputAction::is_key_down(KeyCode::LeftShift, InputActionContext::Generic) {
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
                    if InputAction::is_key_down(KeyCode::LeftShift, InputActionContext::Generic) {
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
                    if InputAction::is_key_down(KeyCode::LeftShift, InputActionContext::Generic) {
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
                    if InputAction::is_key_down(KeyCode::LeftShift, InputActionContext::Generic) {
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

    if let Some(InputAction::Reset) = key {
        game.reset(game.clone());
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

    let mut status_bar = StatusBar::new();
    status_bar.add::<SolveTask>();
    status_bar.add::<BoardGen>();
    status_bar.add::<Fps>();

    let mut game = SudokuGame::new(Some(&mut status_bar));

    loop {
        clear_background(BLACK);
        draw_sudoku(&mut game, &drawing, &mut status_bar);

        status_bar.draw(&mut game, &drawing);
        next_frame().await;
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
}
