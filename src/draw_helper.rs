use std::{cell::Cell, collections::HashMap, rc::Rc, str::FromStr, sync::Mutex};

use macroquad::{
    color::Color,
    shapes::draw_rectangle,
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
    font: Rc<Mutex<Font>>,
    colour_overrides: Rc<Mutex<HashMap<AppColour, Color>>>,
    padding_target: Cell<f32>,
    padding_start: Cell<f32>,
    padding_speed: Cell<f32>,
    font_size: Cell<f32>,
}

#[derive(Debug, Eq, Hash, PartialEq)]
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

impl FromStr for AppColour {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Background" => Ok(AppColour::Background),
            "StatusBar" => Ok(AppColour::StatusBar),
            "StatusBarSeparator" => Ok(AppColour::StatusBarSeparator),
            "StatusBarItemSelected" => Ok(AppColour::StatusBarItemSelected),
            "StatusBarItemOkay" => Ok(AppColour::StatusBarItemOkay),
            "StatusBarItemInProgress" => Ok(AppColour::StatusBarItemInProgress),
            "StatusBarItemError" => Ok(AppColour::StatusBarItemError),
            "StatusBarBufferEdit" => Ok(AppColour::StatusBarBufferEdit),
            "StatusBarItem" => Ok(AppColour::StatusBarItem),
            "BoardBox" => Ok(AppColour::BoardBox),
            "BoardLine" => Ok(AppColour::BoardLine),
            "BoardCellBackground" => Ok(AppColour::BoardCellBackground),
            "BoardSelectedCellBackground" => Ok(AppColour::BoardSelectedCellBackground),
            "BoardHighlightedCellBackground" => Ok(AppColour::BoardHighlightedCellBackground),
            "BoardMousedCellBackground" => Ok(AppColour::BoardMousedCellBackground),
            "BoardRadifiedCell" => Ok(AppColour::BoardRadifiedCell),
            "BoardCorrectCell" => Ok(AppColour::BoardCorrectCell),
            "BoardIncorrectCell" => Ok(AppColour::BoardIncorrectCell),
            "BoardUnknownCell" => Ok(AppColour::BoardUnknownCell),
            _ => Err(()),
        }
    }
}

impl Default for DrawingSettings {
    fn default() -> Self {
        Self {
            font: Rc::new(Mutex::new(
                Self::font_from_bytes(include_bytes!("./TWN19.ttf")).unwrap(),
            )),
            colour_overrides: Rc::new(Mutex::new(HashMap::default())),
            padding_target: Cell::new(30.0),
            padding_start: Cell::new(30.0),
            padding_speed: Cell::new(12.0),
            font_size: Cell::new(1.0),
        }
    }
}

impl DrawingSettings {
    pub fn font_from_bytes(bytes: &[u8]) -> Result<Font, macroquad::Error> {
        text::load_ttf_font_from_bytes(bytes)
    }

    pub fn add_override(&self, colour: AppColour, replaced_with: Color) {
        let mut overrides = self.colour_overrides.lock().unwrap();
        overrides.insert(colour, replaced_with);
    }

    pub fn colour(&self, colour: AppColour) -> Color {
        let overrides = self.colour_overrides.lock().unwrap();
        if let Some(colour) = overrides.get(&colour) {
            return *colour;
        }

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

    pub fn padding_target(&self) -> f32 {
        self.padding_target.get()
    }

    pub fn padding_start(&self) -> f32 {
        self.padding_start.get()
    }

    pub fn padding_speed(&self) -> f32 {
        self.padding_speed.get()
    }

    pub fn set_padding_target(&self, val: f32) {
        self.padding_target.set(val)
    }

    pub fn set_padding_start(&self, val: f32) {
        self.padding_start.set(val)
    }

    pub fn set_padding_speed(&self, val: f32) {
        self.padding_speed.set(val)
    }

    pub fn set_font(&self, font: Font) {
        let mut f = self.font.lock().unwrap();
        *f = font;
    }

    pub fn set_font_size(&self, size: f32) {
        self.font_size.set(size);
    }
}

pub fn draw_and_measure_text(
    drawing: &DrawingSettings,
    text: &str,
    mut x: f32,
    mut y: f32,
    font_size: f32,
    color: Color,
    width: Option<f32>,
    height: Option<f32>,
) -> (f32, f32) {
    let font = drawing.font.lock().unwrap();
    let font_size_mul = drawing.font_size.get();
    let params = TextParams {
        font: Some(&*font),
        color,
        font_size: (font_size * font_size_mul) as u16,
        ..Default::default()
    };

    let mut dim = measure_text(text, Some(&*font), (font_size * font_size_mul) as u16, 1.0);

    if let Some(width) = width {
        x += width / 2.0;
        x -= dim.width / 2.0;
    }

    let mut add_height = dim.height;
    if let Some(height) = height {
        y += height / 2.0;
        y -= dim.height / 2.0;
    } else {
        add_height = 0.0;
    }

    //draw_rectangle(x, y, dim.width, dim.height, macroquad::color::RED);
    draw_text_ex(text, x, y + add_height, params);
    (dim.width, dim.height)
}
