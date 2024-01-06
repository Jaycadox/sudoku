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

pub struct DrawingSettings {
    font: Font,
}

impl Default for DrawingSettings {
    fn default() -> Self {
        Self {
            font: text::load_ttf_font_from_bytes(include_bytes!("./TWN19.ttf")).unwrap(),
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
