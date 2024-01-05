use std::collections::HashSet;

use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;

mod sudoku_game;
use sudoku_game::SudokuGame;

const STATUS_BAR_HEIGHT: f32 = 50.0;

fn draw_sudoku(game: &mut SudokuGame) {
    let (mut width, mut height) = screen_size();
    height -= STATUS_BAR_HEIGHT;
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
                    if is_mouse_button_pressed(MouseButton::Left) {
                        game.selected_cell = Some((mx, my));
                        println!("clicked me");
                    }
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
                        if let Some(key) = get_last_key_pressed() {
                            if let Some(mut value) = match key {
                                KeyCode::Key1 => Some(1),
                                KeyCode::Key2 => Some(2),
                                KeyCode::Key3 => Some(3),
                                KeyCode::Key4 => Some(4),
                                KeyCode::Key5 => Some(5),
                                KeyCode::Key6 => Some(6),
                                KeyCode::Key7 => Some(7),
                                KeyCode::Key8 => Some(8),
                                KeyCode::Key9 => Some(9),
                                KeyCode::Backspace | KeyCode::Key0 | KeyCode::Delete => Some(0),
                                KeyCode::Space => Some(0xFF),
                                _ => None,
                            } {
                                if value == 0xFF && *cell == 0 {
                                    // Auto value
                                    let mut nums = HashSet::new();
                                    value = 0;
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
                                                println!("auto found");
                                                value = i;
                                                break;
                                            }
                                        }
                                    }
                                }

                                if value == 0 || game.cells[(y as usize, x as usize)] == 0 {
                                    game.cells[(y as usize, x as usize)] = value;
                                    // no idea why it needs to be this way
                                }
                            }
                        }
                    }
                }
            }

            if *cell != 0 {
                let number_size = measure_text(&format!("{cell}"), None, rect_size as u16, 1.0);
                let text_col = if unradified {
                    Color::new(0.60, 0.60, 1.00, 1.00)
                } else {
                    WHITE
                };
                draw_text(
                    &format!("{cell}"),
                    start_x + rect_size / 2.0 - number_size.width / 2.0,
                    start_y + rect_size / 2.0 - -number_size.height / 2.0,
                    rect_size,
                    text_col,
                );
            }
            draw_rectangle_lines(start_x, start_y, rect_size, rect_size, 2.0, GRAY);
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
            2.0,
            WHITE,
        );
    }
    if let Some((sx, sy)) = &mut game.selected_cell {
        if let Some(key) = get_last_key_pressed() {
            match key {
                KeyCode::Up | KeyCode::W => {
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
                KeyCode::Down | KeyCode::S => {
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
                KeyCode::Right | KeyCode::D => {
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
                KeyCode::Left | KeyCode::A => {
                    if is_key_down(KeyCode::LeftShift) {
                        if *sx < 3 {
                            *sx += *sx;
                        } else {
                            *sx -= 3;
                        }
                    } else if *sx == 0 {
                        *sx = 8;
                    } else {
                        *sx -= 1;
                    }
                }
                KeyCode::Tab => {
                    *game = SudokuGame::new();
                }
                _ => {}
            }
        }
    }
}

fn draw_status_bar(game: &mut SudokuGame) {
    let (mut width, mut height) = screen_size();

    let (start_x, start_y) = (0.0, height - STATUS_BAR_HEIGHT);
    let (bar_width, bar_height) = (width, STATUS_BAR_HEIGHT);

    draw_rectangle(
        start_x,
        start_y,
        bar_width,
        bar_height,
        Color::from_rgba(20, 20, 20, 255),
    );
}

#[macroquad::main("Sudoku")]
async fn main() {
    let mut game = SudokuGame::new();
    let mut i = 0;
    loop {
        i += 1;
        if i == 100 {
            i = 0;
            println!("fps: {}", get_fps());
        }
        clear_background(BLACK);
        draw_sudoku(&mut game);
        draw_status_bar(&mut game);
        next_frame().await;
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
}
