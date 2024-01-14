use std::rc::Rc;

use macroquad::{
    color::Color,
    text::{self, draw_text_ex, measure_text, Font, TextParams},
    window::screen_height,
};

const STATUS_BAR_PERCENTAGE: f32 = 0.03;
const NORMAL_LINE_PERCENTAGE: f32 = 0.001;
const BOX_LINE_PERCENTAGE: f32 = 0.004;

pub fn get_status_bar_height() -> f32 {
    screen_height() * STATUS_BAR_PERCENTAGE
}

pub fn get_normal_line_width() -> f32 {
    (screen_height() * NORMAL_LINE_PERCENTAGE).max(2.0) as u32 as f32
}

pub fn get_box_line_width() -> f32 {
    (screen_height() * BOX_LINE_PERCENTAGE).max(3.0) as u32 as f32
}

#[derive(Clone)]
pub struct DrawingSettings {
    font: Rc<Font>,
}

pub enum AppColour {
    Background,
    StatusBar,
    StatusBarSeparator,
    StatusBarItemSelected,
    StatusBarItemOkay,
    StatusBarItemInProgress,
    StatusBarItemError,
    StatusBarBufferEdit,
    StatusBarItem,
    BoardBox,
    BoardLine,
    BoardCellBackground,
    BoardSelectedCellBackground,
    BoardHighlightedCellBackground,
    BoardMousedCellBackground,
    BoardRadifiedCell,
    BoardCorrectCell,
    BoardIncorrectCell,
    BoardUnknownCell,
}

impl Default for DrawingSettings {
    fn default() -> Self {
        Self {
            font: Rc::new(text::load_ttf_font_from_bytes(include_bytes!("./TWN19.ttf")).unwrap()),
        }
    }
}

impl DrawingSettings {
    pub fn colour(&self, colour: AppColour) -> Color {
        match colour {
            AppColour::Background => Color::from_rgba(0, 0, 0, 255),
            AppColour::StatusBar => Color::from_rgba(20, 20, 20, 255),
            AppColour::StatusBarSeparator => Color::from_rgba(30, 30, 30, 255),
            AppColour::StatusBarItemSelected => Color::from_rgba(200, 200, 255, 255),
            AppColour::StatusBarItemOkay => Color::from_rgba(0, 255, 0, 255),
            AppColour::StatusBarItemInProgress => Color::from_rgba(255, 255, 0, 255),
            AppColour::StatusBarItemError => Color::from_rgba(255, 0, 0, 255),
            AppColour::StatusBarBufferEdit => Color::from_rgba(255, 255, 0, 255),
            AppColour::StatusBarItem => Color::from_rgba(255, 255, 255, 255),
            AppColour::BoardBox => Color::from_rgba(255, 255, 255, 255),
            AppColour::BoardLine => Color::from_rgba(128, 128, 128, 255),
            AppColour::BoardCellBackground => Color::from_rgba(0, 0, 0, 0),
            AppColour::BoardSelectedCellBackground => Color::from_rgba(255, 255, 255, 124),
            AppColour::BoardHighlightedCellBackground => Color::from_rgba(255, 255, 255, 71),
            AppColour::BoardMousedCellBackground => Color::from_rgba(110, 110, 110, 255),
            AppColour::BoardRadifiedCell => Color::from_rgba(255, 255, 255, 255),
            AppColour::BoardCorrectCell => Color::from_rgba(153, 153, 255, 255),
            AppColour::BoardIncorrectCell => Color::from_rgba(255, 153, 153, 255),
            AppColour::BoardUnknownCell => Color::from_rgba(213, 213, 213, 255),
        }
    }
}

pub fn draw_and_measure_text(
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
