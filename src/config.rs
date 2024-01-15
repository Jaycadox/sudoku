use std::{io::Write, path::PathBuf};

use directories::ProjectDirs;
use tracing::{debug, info, span, trace, Level};

const DEFAULT_RC: &str =
    "BuiltinAdd CpuSolve BoardGen Fps OnBoardInit & OnBoardInit CpuSolve run & BoardGen 30";

pub fn get_file_path(name: &str) -> PathBuf {
    let span = span!(Level::INFO, "FileLoad");
    let _enter = span.enter();

    trace!("Attempting to get file: {}", name);
    ProjectDirs::from("io.github", "Jaycadox", "Sudoku")
        .map(|dirs| {
            let config_dir = dirs.config_dir();

            config_dir.join(name).to_path_buf()
        })
        .expect("unable to find root config directory")
}

pub fn get_file(name: &str, default: Option<&[u8]>) -> Option<Vec<u8>> {
    let file_path = get_file_path(name);
    info!("Loading file from: {}", file_path.display());
    if !std::path::Path::exists(&file_path) {
        debug!("File doesn't exist, attempting to create default...");
        let _ = std::fs::create_dir_all(file_path.parent().expect("Should have parent"));
        std::fs::File::create(&file_path)
            .ok()
            .and_then(|mut f| f.write_all(default?).ok())?;
    }

    std::fs::read(file_path).ok()
}

pub fn get_rc() -> String {
    let contents =
        get_file(".sudokurc", Some(DEFAULT_RC.as_bytes())).expect("Should always have value");
    let contents_str = String::from_utf8_lossy(&contents);
    contents_str.to_string()
}
