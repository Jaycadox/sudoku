use std::collections::HashSet;

use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use tracing::{debug, span, trace, warn, Level};

use draw_helper::*;
use input_helper::*;
use status_bar::{DrawHookData, StatusBar, StatusBarHookAction, StatusBarItemStatus};
use sudoku_game::SudokuGame;

use crate::sudoku_game::ResetSignal;

mod config;
mod draw_helper;
mod input_helper;
mod status_bar;
mod sudoku_game;
mod task_status;

fn draw_sudoku(game: &mut SudokuGame, drawing: &DrawingSettings, status_bar: &mut StatusBar) {
    let span = span!(Level::INFO, "DrawSudoku");
    let _enter = span.enter();

    if game.reset_signalled == ResetSignal::Soft {
        debug!("Game reset signalled, resetting status bar...");
        status_bar.restart(game);
        game.reset_signalled = ResetSignal::None;
    }

    fn lerp(start: f32, end: f32, t: f32) -> f32 {
        start * (1.0 - t) + end * t
    }

    game.padding_progress = f32::min(
        1.0,
        lerp(
            game.padding_progress,
            1.0,
            drawing.padding_speed() * get_frame_time(),
        ),
    );
    let padding = lerp(
        drawing.padding_start(),
        drawing.padding_target(),
        game.padding_progress,
    );

    let (mut width, mut height) = screen_size();

    let draw_hook_data = DrawHookData {
        x: 0.0,
        y: 0.0,
        w: width,
        h: height,
    };

    for item in status_bar.items() {
        let resp = item.background_draw_hook(&draw_hook_data);
        if let StatusBarHookAction::Stop = resp {
            break;
        }
    }

    height -= get_status_bar_height();
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

    let mut key = InputAction::get_last_input(InputActionContext::Generic, &game.input);

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

        if matches!(key, Some(InputAction::Function(_))) || matches!(key, Some(InputAction::Reset))
        {
            change_selected_to_cursor = false;
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

            draw_rectangle(
                start_x,
                start_y,
                rect_size,
                rect_size,
                drawing.colour(AppColour::BoardCellBackground),
            );

            let unradified = game.unradified.contains(&(y as u8 * 9 + x as u8));

            if highlight_cells.contains(&(y as u32 * 9 + x as u32)) {
                draw_rectangle(
                    start_x,
                    start_y,
                    rect_size,
                    rect_size,
                    drawing.colour(AppColour::BoardHighlightedCellBackground),
                );
            }

            if let Some((mx, my)) = mouse_pos {
                if x == mx as f32 && y == my as f32 {
                    draw_rectangle(
                        start_x,
                        start_y,
                        rect_size,
                        rect_size,
                        drawing.colour(AppColour::BoardMousedCellBackground),
                    );
                }
            }

            if let Some((sx, sy)) = game.selected_cell {
                if x == sx as f32 && y == sy as f32 {
                    draw_rectangle(
                        start_x,
                        start_y,
                        rect_size,
                        rect_size,
                        drawing.colour(AppColour::BoardSelectedCellBackground),
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

            let draw_hook_data = DrawHookData {
                x: start_x,
                y: start_y,
                w: rect_size,
                h: rect_size,
            };

            let mut cancelled = false;
            for item in status_bar.items() {
                if let StatusBarHookAction::Stop = item.cell_text_draw_hook(
                    drawing,
                    game,
                    SudokuGame::xy_pos_to_idx(x as u32, y as u32, game.cells.shape()[1] as u32)
                        as u8,
                    *cell,
                    &draw_hook_data,
                ) {
                    cancelled = true;
                    break;
                }
            }

            if *cell != 0 && !cancelled {
                let text_col = if unradified {
                    let solver_value = status_bar.item_with_name("CpuSolve").map(|x| x.status());
                    match solver_value {
                        Some(StatusBarItemStatus::Ok(status_bar::StatusBarItemOkData::Game(
                            solved,
                        ))) => {
                            let solved_cell = solved.cells[(y as usize, x as usize)];
                            let our_cell = game.cells[(y as usize, x as usize)];
                            if solved_cell == our_cell {
                                drawing.colour(AppColour::BoardCorrectCell)
                            } else {
                                drawing.colour(AppColour::BoardIncorrectCell)
                            }
                        }
                        _ => drawing.colour(AppColour::BoardUnknownCell),
                    }
                } else {
                    drawing.colour(AppColour::BoardRadifiedCell)
                };

                let _ = draw_and_measure_text(
                    drawing,
                    &format!("{cell}"),
                    start_x,
                    start_y,
                    rect_size,
                    text_col,
                    (Some(rect_size), Some(rect_size)),
                );
            }
            draw_rectangle_lines(
                start_x,
                start_y,
                rect_size,
                rect_size,
                get_normal_line_width(),
                drawing.colour(AppColour::BoardLine),
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
            drawing.colour(AppColour::BoardBox),
        );
    }

    if let Some((sx, sy)) = &mut game.selected_cell {
        if let Some(ref key) = key {
            match key {
                InputAction::MoveUp => {
                    if InputAction::is_key_down(
                        KeyCode::LeftShift,
                        InputActionContext::Generic,
                        &game.input,
                    ) {
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
                    if InputAction::is_key_down(
                        KeyCode::LeftShift,
                        InputActionContext::Generic,
                        &game.input,
                    ) {
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
                    if InputAction::is_key_down(
                        KeyCode::LeftShift,
                        InputActionContext::Generic,
                        &game.input,
                    ) {
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
                    if InputAction::is_key_down(
                        KeyCode::LeftShift,
                        InputActionContext::Generic,
                        &game.input,
                    ) {
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
        debug!("Manual reset triggered...");
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
    #[cfg(debug_assertions)]
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    #[cfg(not(debug_assertions))]
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
    'outer: loop {
        let rc = config::get_rc();

        trace!("Loading drawing settings...");
        let drawing = DrawingSettings::default();

        let mut status_bar = StatusBar::new(&drawing);
        status_bar.enter_buffer_commands(&[&rc[..]]);

        let mut game = SudokuGame::new(None);

        loop {
            let span = span!(Level::TRACE, "MainLoop");
            let _enter = span.enter();

            clear_background(drawing.colour(AppColour::Background));
            draw_sudoku(&mut game, &drawing, &mut status_bar);

            status_bar.draw(&mut game, &drawing);
            let should_continue = game.reset_signalled != ResetSignal::Hard;
            next_frame().await;

            if !should_continue {
                warn!("Hard resetting...");
                continue 'outer;
            }
        }
    }
}
