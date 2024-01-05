use std::collections::HashSet;

use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use ndarray::{s, Array2, ArrayView, ArrayView2, Axis, Ix1};

#[derive(Clone)]
struct SudokuGame {
    cells: Array2<u8>,
    unradified: Vec<u8>,
    selected_cell: Option<(u32, u32)>,
}

impl SudokuGame {
    fn new() -> Self {
        let mut cells = Array2::zeros((9, 9));
        let inp =
            "050010040107000602000905000208030501040070020901080406000401000304000709020060010";
        let inp = inp.replace('.', "0");
        let mut unradified = Vec::new();
        for (i, cell) in cells.iter_mut().enumerate() {
            let val = inp.chars().nth(i).unwrap().to_digit(10).unwrap() as u8;
            *cell = val;
            if val == 0 {
                unradified.push(i as u8);
            }
        }
        SudokuGame {
            cells,
            unradified,
            selected_cell: None,
        }
    }
    fn rows(&self) -> Vec<ArrayView<u8, Ix1>> {
        (0..9)
            .map(|i| self.cells.index_axis(Axis(0), i))
            .collect::<Vec<_>>()
    }
    fn cols(&self) -> Vec<ArrayView<u8, Ix1>> {
        (0..9)
            .map(|i| self.cells.index_axis(Axis(1), i))
            .collect::<Vec<_>>()
    }
    fn boxes(&self) -> Array2<ArrayView2<u8>> {
        let boxes = (0..3)
            .flat_map(|i| {
                (0..3).map(move |j| self.cells.slice(s![i * 3..(i + 1) * 3, j * 3..(j + 1) * 3]))
            })
            .collect::<Vec<_>>();
        Array2::from_shape_vec((3, 3), boxes).expect("bad vector shape")
    }
}

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

    let mut highlight_cells = Vec::new();
    if let Some((sx, sy)) = game.selected_cell {
        let current_selected = game.cells[(sy as usize, sx as usize)];
        if current_selected != 0 {
            let mut same_cells = Vec::new();
            for (i, cell) in game.clone().cells.iter().enumerate() {
                if *cell == current_selected {
                    let sx = i % game.cells.shape()[1];
                    let sy = i / game.cells.shape()[1];
                    let box_coord = (sx / 3, sy / 3);
                    for bx in 0..3 {
                        for by in 0..3 {
                            let bx = box_coord.0 * 3 + bx;
                            let by = box_coord.1 * 3 + by;
                            highlight_cells.push(by as u32 * 9 + bx as u32);
                        }
                    }
                    same_cells.push(i as u32);
                }
            }
            for (i, _) in game.cells.iter().enumerate() {
                let rx = i % game.cells.shape()[1];
                let ry = i / game.cells.shape()[1];
                for scell in &same_cells {
                    let sx = *scell as usize % game.cells.shape()[1];
                    let sy = *scell as usize / game.cells.shape()[1];
                    if rx == sx || ry == sy {
                        highlight_cells.push(i as u32);
                    }
                }
            }
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
