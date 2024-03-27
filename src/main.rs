#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
use std::collections::HashSet;

use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use tracing::{debug, span, trace, warn, Level};

use draw_helper::{
    draw_text_in_bounds, get_box_line_width, get_normal_line_width, get_status_bar_height,
    AppColour, DrawingSettings,
};
use input_helper::{InputAction, InputActionContext};
use status_bar::{DrawHookData, HookAction, StatusBar};
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

    let padding = update_padding(game, drawing);

    let (mut width, mut height) = screen_size();

    run_background_draw_hook(width, height, status_bar);

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

    try_move_selected_from_input(mouse_pos, &key, game);
    for (y, row) in game.clone().rows().iter().enumerate() {
        let y = y as f32;
        for (x, _) in row.iter().enumerate() {
            let x = x as f32;
            let (start_x, start_y) = (
                x_pad + s_padding + (x * rect_size),
                y_pad + s_padding + (y * rect_size),
            );

            draw_generic_cell_background(
                (start_x, start_y),
                rect_size,
                drawing,
                (x, y),
                &highlight_cells,
                mouse_pos,
            );
            draw_cell_text(
                game,
                (x, y),
                (start_x, start_y),
                rect_size,
                drawing,
                &mut key,
                status_bar,
            );
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
    let game_copy = game.clone();
    let boxes = game_copy.boxes();
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

    handle_keyboard_movement(game, &key);
    if let Some(InputAction::Reset) = key {
        debug!("Manual reset triggered...");
        game.reset(game.clone());
    }
}

fn run_background_draw_hook(width: f32, height: f32, status_bar: &mut StatusBar<'_>) {
    let draw_hook_data = DrawHookData {
        x: 0.0,
        y: 0.0,
        w: width,
        h: height,
    };

    for item in status_bar.items() {
        let resp = item.background_draw_hook(&draw_hook_data);
        if let HookAction::Stop = resp {
            break;
        }
    }
}

fn draw_cell_text(
    game: &mut SudokuGame,
    pos: (f32, f32),
    start: (f32, f32),
    rect_size: f32,
    drawing: &DrawingSettings,
    key: &mut Option<InputAction>,
    status_bar: &mut StatusBar<'_>,
) {
    let idx = SudokuGame::xy_pos_to_idx(pos.0 as u32, pos.1 as u32, game.cells.shape()[1] as u32)
        as usize;
    let cell = *game.cells.iter().nth(idx).unwrap();
    let unradified = game.unradified.contains(&(idx as u8));

    if let Some((sx, sy)) = game.selected_cell {
        handle_selected_cell_input(pos, (sx, sy), start, rect_size, drawing, key, game);
    }

    let cancelled = run_cell_text_draw_hook(start, rect_size, status_bar, drawing, game, pos, cell);

    if cell != 0 && !cancelled {
        let text_col =
            run_cell_text_colour_hook(unradified, pos.0, pos.1, game, drawing, status_bar);

        let _ = draw_text_in_bounds(
            drawing,
            &format!("{cell}"),
            start.0,
            start.1,
            rect_size,
            text_col,
            (Some(rect_size), Some(rect_size)),
        );
    }
}

fn run_cell_text_colour_hook(
    unradified: bool,
    x: f32,
    y: f32,
    game: &mut SudokuGame,
    drawing: &DrawingSettings,
    status_bar: &mut StatusBar<'_>,
) -> Color {
    let text_col = if unradified {
        assert!(x >= 0.0);
        assert!(y >= 0.0);
        let index = SudokuGame::xy_pos_to_idx(
            x as u32,
            y as u32,
            u32::try_from(game.cells.shape()[0]).unwrap(),
        );
        let mut col = drawing.colour(AppColour::BoardUnknownCell);
        for item in status_bar.items() {
            if let Some(status) = item.cell_text_colour_hook(game, u8::try_from(index).unwrap()) {
                match status {
                    HookAction::Continue(colour) => {
                        col = drawing.colour(colour);
                    }
                    HookAction::Stop => break,
                }
            }
        }

        col
    } else {
        drawing.colour(AppColour::BoardRadifiedCell)
    };
    text_col
}

fn run_cell_text_draw_hook(
    start: (f32, f32),
    rect_size: f32,
    status_bar: &mut StatusBar<'_>,
    drawing: &DrawingSettings,
    game: &mut SudokuGame,
    pos: (f32, f32),
    cell: u8,
) -> bool {
    let draw_hook_data = DrawHookData {
        x: start.0,
        y: start.1,
        w: rect_size,
        h: rect_size,
    };

    let mut cancelled = false;
    for item in status_bar.items() {
        if let HookAction::Stop = item.cell_text_draw_hook(
            drawing,
            game,
            SudokuGame::xy_pos_to_idx(pos.0 as u32, pos.1 as u32, game.cells.shape()[1] as u32)
                as u8,
            cell,
            &draw_hook_data,
        ) {
            cancelled = true;
            break;
        }
    }
    cancelled
}

fn handle_selected_cell_input(
    pos: (f32, f32),
    size: (u32, u32),
    start: (f32, f32),
    rect_size: f32,
    drawing: &DrawingSettings,
    key: &mut Option<InputAction>,
    game: &mut SudokuGame,
) {
    let idx = SudokuGame::xy_pos_to_idx(pos.0 as u32, pos.1 as u32, game.cells.shape()[1] as u32)
        as usize;
    let cell = game.cells.iter().nth(idx).unwrap();
    let unradified = game.unradified.contains(&(idx as u8));
    if (pos.0 - size.0 as f32).abs() < f32::EPSILON && (pos.1 - size.1 as f32).abs() < f32::EPSILON
    {
        draw_rectangle(
            start.0,
            start.1,
            rect_size,
            rect_size,
            drawing.colour(AppColour::BoardSelectedCellBackground),
        );
        if unradified {
            if let Some(ref mut value) = *key {
                if matches!(value, InputAction::AutoPlay) && *cell == 0 {
                    // Auto value
                    do_auto_play(value, pos.0, pos.1, game);
                }
                match value {
                    InputAction::NumberEntered(_) | InputAction::Clear => {
                        game.cells[(pos.1 as usize, pos.0 as usize)] = match value {
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

fn draw_generic_cell_background(
    start: (f32, f32),
    rect_size: f32,
    drawing: &DrawingSettings,
    pos: (f32, f32),
    highlight_cells: &[u32],
    mouse_pos: Option<(u32, u32)>,
) {
    draw_rectangle(
        start.0,
        start.1,
        rect_size,
        rect_size,
        drawing.colour(AppColour::BoardCellBackground),
    );

    if highlight_cells.contains(&(pos.1 as u32 * 9 + pos.0 as u32)) {
        draw_rectangle(
            start.0,
            start.1,
            rect_size,
            rect_size,
            drawing.colour(AppColour::BoardHighlightedCellBackground),
        );
    }

    if let Some((mx, my)) = mouse_pos {
        if (pos.0 - mx as f32).abs() < f32::EPSILON && (pos.1 - my as f32).abs() < f32::EPSILON {
            draw_rectangle(
                start.0,
                start.1,
                rect_size,
                rect_size,
                drawing.colour(AppColour::BoardMousedCellBackground),
            );
        }
    }
}

fn update_padding(game: &mut SudokuGame, drawing: &DrawingSettings) -> f32 {
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
    lerp(
        drawing.padding_start(),
        drawing.padding_target(),
        game.padding_progress,
    )
}

fn do_auto_play(value: &mut InputAction, x: f32, y: f32, game: &mut SudokuGame) {
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

fn handle_keyboard_movement(game: &mut SudokuGame, key: &Option<InputAction>) {
    if let Some((sx, sy)) = &mut game.selected_cell {
        if let Some(ref key) = *key {
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
}

fn try_move_selected_from_input(
    mouse_pos: Option<(u32, u32)>,
    key: &Option<InputAction>,
    game: &mut SudokuGame,
) {
    if let Some((mx, my)) = mouse_pos {
        let mut change_selected_to_cursor = false;

        if let Some(ref value) = *key {
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
