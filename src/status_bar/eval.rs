use std::default::Default;

use macroquad::color::{Color, WHITE};
use mlua::prelude::{LuaResult, LuaUserData, LuaUserDataMethods};
use mlua::Error::RuntimeError;
use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value};
use tracing::{debug, error, info, info_span, span, trace, warn, Level};

use crate::config;
use crate::status_bar::shorthands::list::ShorthandList;
use crate::sudoku_game::SudokuGame;

use super::{cpu_solve, StatusBar, StatusBarItem};

impl LuaUserData for SudokuGame {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "pretty_board_string",
            |_, s, ()| Ok(s.pretty_board_string()),
        );
        methods.add_method("board_string", |_, s, ()| Ok(s.board_string()));
        methods.add_method("solve", |_, s, ()| Ok(cpu_solve::solve(s)));
        methods.add_method_mut::<_, String, ()>("update_board_from_string", |_, s, inp| {
            let Some(grid) = SudokuGame::generate_cells_from_string(&inp) else {
                return Err(RuntimeError("Invalid cell format".parse().unwrap()));
            };
            s.cells = grid;
            Ok(())
        });
        methods.add_method_mut::<_, String, ()>("new_from_string", |_, s, inp| {
            let new_game = SudokuGame::new(Some(&inp));
            s.reset(new_game);
            Ok(())
        });
    }
}

#[derive(Default)]
pub struct Eval {
    scripts: Vec<LuaScript>,
}

enum LuaRun {
    File { code: String, name: String },
    Repl { code: String },
}

struct LuaScript {
    name: String,
    lua: Lua,
}

impl LuaScript {
    fn exec(name: &str, code: &str) -> LuaResult<LuaScript> {
        let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())?;

        let scr = Self {
            lua,
            name: name.to_string(),
        };

        scr.lua
            .load(
                r#"
events = {}

__ON_INIT_FUNCTIONS__ = {}
events["on_init"] = function(callback)
    table.insert(__ON_INIT_FUNCTIONS__, callback)
end

__ON_BOARDGEN_FUNCTIONS__ = {}
events["on_board_gen"] = function(callback)
    table.insert(__ON_BOARDGEN_FUNCTIONS__, callback)
end

"#,
            )
            .exec()?;

        let name_2 = name.to_string();
        scr.lua.globals().set(
            "info",
            scr.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                info!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = name.to_string();
        scr.lua.globals().set(
            "warn",
            scr.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                warn!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = name.to_string();
        scr.lua.globals().set(
            "error",
            scr.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                error!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = name.to_string();
        scr.lua.globals().set(
            "debug",
            scr.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                debug!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = name.to_string();
        scr.lua.globals().set(
            "trace",
            scr.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                trace!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        scr.lua.load(code).set_name(name).exec()?;

        Ok(scr)
    }

    fn generic_game_callback(&self, sudoku: &mut SudokuGame, name: &str) -> LuaResult<()> {
        let funcs = self.lua.globals().get::<_, Table>(name)?;

        for item in funcs.pairs::<Value, Function>() {
            let (_, func) = item?;

            self.lua.scope(|scope| {
                let bs = scope.create_userdata_ref_mut(sudoku)?;
                func.call(bs)
            })?;
        }

        Ok(())
    }
}

impl LuaRun {
    fn code(&self) -> String {
        match self {
            LuaRun::File { code, .. } => code.to_string(),
            LuaRun::Repl { code, .. } => code.to_string(),
        }
    }

    fn name(&self) -> String {
        match self {
            LuaRun::File { name, .. } => name.to_string(),
            LuaRun::Repl { .. } => "Repl".to_string(),
        }
    }

    fn run(&self, item: &mut Eval, game: &mut SudokuGame) -> LuaResult<Option<String>> {
        let span = span!(Level::INFO, "RunLua");
        let _enter = span.enter();

        let name = self.name();
        let code = self.code();

        match self {
            LuaRun::File { .. } => {
                info!("Executing Lua script: {name}...");
                let scr = LuaScript::exec(&name, &code)?;
                scr.generic_game_callback(game, "__ON_INIT_FUNCTIONS__")?;
                item.scripts.push(scr);
                Ok(None)
            }
            LuaRun::Repl { .. } => {
                let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())?;
                let chunk = lua.load(code).set_name(name);
                chunk.eval::<String>().map(Some)
            }
        }
    }
}

impl StatusBarItem for Eval {
    fn name(&self) -> &'static str {
        "Eval"
    }

    fn update(&mut self, _game: &mut SudokuGame, _status_bar: &mut StatusBar) -> (String, Color) {
        ("".to_string(), WHITE)
    }

    fn board_init(&mut self, game: &mut SudokuGame, _status_bar: &mut StatusBar) {
        let span = span!(Level::INFO, "RunLua");
        let _enter = span.enter();

        for scr in &self.scripts {
            if let Err(e) = scr.generic_game_callback(game, "__ON_BOARDGEN_FUNCTIONS__") {
                error!("Lua 'BoardInit' error: {e}");
            }
        }
    }

    fn activated(
        &mut self,
        game: &mut crate::sudoku_game::SudokuGame,
        status_bar: &mut super::StatusBar,
    ) {
        let span = span!(Level::INFO, "EvalActivated");
        let _enter = span.enter();

        let code = match (status_bar.buffer.get(0..1), status_bar.buffer.get(1..)) {
            (Some("@"), Some(file_name)) => {
                let Ok(file) = std::fs::read_to_string(config::get_file_path(file_name)) else {
                    status_bar.buffer = format!("FileNotFound: {file_name}");
                    return;
                };

                if let Some(idx) = self
                    .scripts
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.name == file_name)
                    .map(|(i, _)| i)
                    .next()
                {
                    warn!("Script with name '{file_name}' is already loaded, unloading...");
                    self.scripts.remove(idx);
                }

                LuaRun::File {
                    code: file,
                    name: file_name.to_string(),
                }
            }
            _ => LuaRun::Repl {
                code: status_bar.buffer.to_string(),
            },
        };

        match code.run(self, game) {
            Ok(Some(result)) => {
                status_bar.buffer = result;
            }
            Err(e) => {
                status_bar.buffer = format!("LuaError: {e}");
                error!("Lua Error: {e}");
            }
            _ => {}
        }
    }

    fn display_mode(&self) -> super::StatusBarDisplayMode {
        super::StatusBarDisplayMode::None
    }

    fn shorthands(&self) -> Option<ShorthandList> {
        shorthand!((r"^=(.*)", "$1"))
    }
}
