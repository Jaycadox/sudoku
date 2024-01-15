use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc,
    },
    thread::JoinHandle,
};

use macroquad::{
    color::Color,
    math::Vec2,
    texture::{draw_texture_ex, Image, Texture2D},
};
use tracing::{debug, error, info, span, trace, warn, Level};
use url::Url;

use crate::config;

use super::StatusBarItem;

pub struct BackgroundImage {
    image: Option<Texture2D>,
    rx: Receiver<Option<Vec<u8>>>,
    thread: JoinHandle<()>,
    should_stop: Arc<AtomicBool>,
    opacity: u8,
}

impl Default for BackgroundImage {
    fn default() -> Self {
        let (_, rx) = std::sync::mpsc::channel();
        let s = Self {
            image: None,
            rx,
            thread: std::thread::spawn(|| {}),
            should_stop: Arc::new(AtomicBool::new(false)),
            opacity: 255,
        };
        while !s.thread.is_finished() {}
        s
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

        if !self.thread.is_finished() {
            self.should_stop
                .store(true, std::sync::atomic::Ordering::Relaxed);
            warn!("Background image thread has been signalled to stop");
            return;
        }

        let location = status_bar.buffer.trim().to_string();

        trace!("Attempting to locate: {}...", location);
        if let Ok(url) = Url::parse(&location) {
            trace!("Location appears to be a URL");
            match url.scheme() {
                "http" | "https" => {
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.rx = rx;
                    let should_stop = Arc::clone(&self.should_stop);
                    self.thread = std::thread::spawn(move || {
                        let Ok(req) = reqwest::blocking::get(&location) else {
                            error!("Request failed");
                            let _ = tx.send(None);
                            return;
                        };
                        let Ok(data) = req.bytes() else {
                            error!("Failed to get request bytes");
                            let _ = tx.send(None);
                            return;
                        };
                        if !should_stop.load(Ordering::Relaxed) {
                            debug!("Downloaded resource from: {}", location);
                            let _ = tx.send(Some(data.to_vec()));
                        }
                    });
                }
                x => {
                    error!("Scheme is not HTTP/S: {}", x);
                    status_bar.buffer = "BadScheme".to_string();
                }
            }
        } else {
            trace!("Treating location as file...");
            if let Some(file) = config::get_file(&location, None) {
                debug!("Found file '{}', size = {}", location, file.len());
                let (tx, rx) = std::sync::mpsc::channel();
                self.rx = rx;
                let _ = tx.send(Some(file));
            } else {
                status_bar.buffer = "FileNotFound".to_string();
                error!("Could not find file: '{}'", location);
            }
        }
    }

    fn update(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) -> (String, macroquad::prelude::Color) {
        let span = span!(Level::INFO, "BackgroundImageUpdate");
        let _enter = span.enter();

        if let Ok(Some(data)) = self.rx.try_recv() {
            match Image::from_file_with_format(&data, None) {
                Ok(image) => {
                    let texture = Texture2D::from_image(&image);
                    self.image = Some(texture);
                    info!("Loaded texture");
                }
                Err(e) => {
                    error!("Image is not valid: {}", e);
                }
            };
        }

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
