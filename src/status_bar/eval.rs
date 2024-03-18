use mlua::{Lua, LuaOptions, StdLib};

use crate::status_bar::shorthands::list::ShorthandList;

use super::StatusBarItem;

#[derive(Default)]
pub struct Eval;

impl StatusBarItem for Eval {
    fn name(&self) -> &'static str {
        "Eval"
    }

    fn activated(
        &mut self,
        _game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let Ok(lua) = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()) else {
            status_bar.buffer = "LuaStartFailed".to_string();
            return;
        };
        match lua.load(&status_bar.buffer).eval::<String>() {
            Ok(num) => {
                status_bar.buffer = num;
            }
            Err(e) => status_bar.buffer = format!("LuaError: {}", e),
        }
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }

    fn shorthands(&self) -> Option<ShorthandList> {
        shorthand!((r"^=(.*)", "$1"))
    }
}
