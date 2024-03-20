use macroquad::{
    color::Color,
    math::Vec2,
    texture::{draw_texture_ex, Image, Texture2D},
};
use tracing::{debug, error, span, trace, Level};

use crate::config;

use super::StatusBarItem;

pub struct BackgroundImage {
    image: Option<Texture2D>,
    opacity: u8,
}

impl Default for BackgroundImage {
    fn default() -> Self {
        Self {
            image: None,
            opacity: 255,
        }
    }
}

impl StatusBarItem for BackgroundImage {
    fn name(&self) -> &'static str {
        "BackgroundImage"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let span = span!(Level::INFO, "BackgroundImageActivate");
        let _enter = span.enter();

        if status_bar.buffer.starts_with("opacity=") {
            let Ok(opacity) = &status_bar.buffer[8..].parse::<u8>() else {
                error!("Could not parse opacity input: '{}'", status_bar.buffer);
                status_bar.buffer = "CouldNotParse".to_string();
                return;
            };
            trace!("Set opacity to: {}", *opacity);
            self.opacity = *opacity;
            return;
        }

        let location = status_bar.buffer.trim().to_string();

        if let Some(file) = config::get_file(&location, None) {
            debug!("Found file '{}', size = {}", location, file.len());
            match Image::from_file_with_format(&file, None) {
                Ok(image) => {
                    let texture = Texture2D::from_image(&image);
                    self.image = Some(texture);
                    debug!("Loaded texture");
                }
                Err(e) => {
                    error!("Image is not valid: {}", e);
                }
            };
        } else {
            status_bar.buffer = "FileNotFound".to_string();
            error!("Could not find file: '{}'", location);
        }
    }

    fn update(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) -> (String, macroquad::prelude::Color) {
        let span = span!(Level::INFO, "BackgroundImageUpdate");
        let _enter = span.enter();

        (
            "".to_string(),
            status_bar
                .drawing
                .colour(crate::draw_helper::AppColour::StatusBarItem),
        )
    }

    fn background_draw_hook(&self, data: &super::DrawHookData) -> super::DrawHookAction {
        if let Some(texture) = &self.image {
            draw_texture_ex(
                texture,
                data.x,
                data.y,
                Color::from_rgba(255, 255, 255, self.opacity),
                macroquad::texture::DrawTextureParams {
                    dest_size: Some(Vec2::new(data.w, data.h)),
                    source: None,
                    rotation: 0.0,
                    flip_x: false,
                    flip_y: false,
                    pivot: None,
                },
            )
        }
        super::DrawHookAction::Continue
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }
}
