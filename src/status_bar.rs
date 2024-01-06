use macroquad::{
    color::{Color, GREEN, RED, WHITE, YELLOW},
    miniquad::window::screen_size,
    shapes::draw_rectangle,
};

use crate::{
    draw_helper::*,
    sudoku_game::SudokuGame,
    task_status::{GetTask, TaskStatus},
};

pub fn draw_status_bar(game: &mut SudokuGame, drawing: &DrawingSettings) {
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
