#![allow(clippy::similar_names)]
use std::default::Default;
use std::str::FromStr;

use macroquad::color::{Color, WHITE};
use macroquad::input::{is_mouse_button_down, mouse_position, MouseButton};
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::is_mouse_button_pressed;
use macroquad::shapes::draw_rectangle;
use macroquad::window::{screen_height, screen_width};
use mlua::prelude::{LuaResult, LuaUserData, LuaUserDataMethods};
use mlua::Error::RuntimeError;
use mlua::{Function, Lua, LuaOptions, StdLib, Table, Value};
use tracing::{debug, error, info, info_span, span, trace, warn, Level};

use crate::draw_helper::{draw_text_in_bounds, get_status_bar_height, DrawingSettings};
use crate::status_bar::shorthands::list::List;
use crate::sudoku_game::SudokuGame;
use crate::{config, AppColour};

use super::{cpu_solve, Item, StatusBar};

impl LuaUserData for SudokuGame {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "pretty_board_string",
            |_, s, ()| Ok(s.pretty_board_string()),
        );

        methods.add_method("cells", |lua, s, ()| {
            let table = lua.create_table()?;
            for cell in &s.cells {
                table.push(*cell)?;
            }
            Ok(table)
        });

        methods.add_method::<_, u32, _>("unoccupied_cells_at", |_, s, idx| {
            let in_sight = cpu_solve::get_occupied_numbers_at_cell(
                s,
                SudokuGame::idx_pos_to_xy(idx, s.cells.shape()[1] as u32),
            );
            let mut not_in_sight = vec![];
            for i in 1..=9 {
                if !in_sight[i - 1] {
                    not_in_sight.push(i);
                }
            }

            Ok(not_in_sight)
        });

        methods.add_method("board_string", |_, s, ()| Ok(s.board_string()));
        methods.add_method("solve", |_, s, ()| Ok(cpu_solve::solve(s)));
        methods.add_method("is_solved", |_, s, ()| Ok(s.is_solved()));
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
        methods.add_method_mut::<_, String, ()>("enter_buffer_command", |_, s, inp| {
            s.wanted_commands.push(inp);
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
    fn exec(name: &str, code: &str, status_bar: &StatusBar) -> LuaResult<LuaScript> {
        let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())?;

        let scr = Self {
            lua,
            name: name.to_string(),
        };

        scr.load_internal_lib()?;
        scr.load_logging_lib()?;
        scr.load_events_lib()?;
        scr.load_drawing_lib(status_bar.drawing.clone())?;
        scr.load_cursor_lib()?;

        scr.lua.load(code).set_name(name).exec()?;

        Ok(scr)
    }

    fn load_internal_lib(&self) -> LuaResult<()> {
        self.lua.globals().set(
            "__systime_ms__",
            self.lua.create_function(move |_, ()| {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|x| x.as_millis())
                    .unwrap_or(0);
                Ok(time)
            })?,
        )?;

        Ok(())
    }

    fn load_events_lib(&self) -> LuaResult<()> {
        self.lua
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

__ON_UPDATE_FUNCTIONS__ = {}
events["on_update"] = function(callback)
    table.insert(__ON_UPDATE_FUNCTIONS__, callback)
end

__WAIT_FUNCTIONS__ = {}
events["wait_ms"] = function(ms, callback)
    table.insert(__WAIT_FUNCTIONS__, { target = __systime_ms__() + ms, cb = callback, interval = -1 })
end

__WAIT_FUNCTIONS__ = {}
events["repeat_ms"] = function(ms, callback)
    table.insert(__WAIT_FUNCTIONS__, { target = __systime_ms__() + 1, cb = callback, interval = ms })
end

print = info
"#,
            )
            .exec()
    }

    fn load_drawing_lib(&self, draw_settings: DrawingSettings) -> LuaResult<()> {
        let drawing = self.lua.create_table()?;

        drawing.set(
            "screen_size",
            self.lua.create_function(|_, ()| Ok(screen_size()))?,
        )?;

        drawing.set(
            "game_size",
            self.lua.create_function(|_, ()| {
                let (x, mut y) = screen_size();
                y -= get_status_bar_height();

                Ok((x, y))
            })?,
        )?;
        drawing.set(
            "game_origin",
            self.lua.create_function(|_, ()| Ok((0.0f32, 0.0f32)))?,
        )?;
        drawing.set(
            "status_bar_size",
            self.lua.create_function(|_, ()| {
                let (x, y) = (screen_width(), get_status_bar_height());
                Ok((x, y))
            })?,
        )?;

        drawing.set(
            "status_bar_origin",
            self.lua.create_function(|_, ()| {
                let (x, y) = (0.0f32, screen_height() - get_status_bar_height());
                Ok((x, y))
            })?,
        )?;
        let draw_settings_2 = draw_settings.clone();
        drawing.set(
            "colour",
            self.lua
                .create_function::<String, _, _>(move |_, col_name| {
                    let Ok(colour) = AppColour::from_str(&col_name) else {
                        return Err(RuntimeError("Invalid colour name".to_string()));
                    };

                    let colour = draw_settings_2.colour(colour);
                    Ok((colour.r, colour.g, colour.b, colour.a))
                })?,
        )?;
        drawing.set(
            "draw_rect",
            self.lua.create_function(|_, (x, y, w, h, r, g, b, a)| {
                draw_rectangle(x, y, w, h, Color::new(r, g, b, a));
                Ok(())
            })?,
        )?;
        drawing.set(
            "draw_text",
            self.lua
                .create_function(move |_, (text, x, y, size, r, g, b, a)| {
                    let text: String = text;
                    draw_text_in_bounds(
                        &draw_settings,
                        &text,
                        x,
                        y,
                        size,
                        Color::new(r, g, b, a),
                        (None, None),
                    );
                    Ok(())
                })?,
        )?;

        self.lua.globals().set("drawing", drawing)?;
        Ok(())
    }

    fn load_cursor_lib(&self) -> LuaResult<()> {
        let cursor = self.lua.create_table()?;
        cursor.set(
            "position",
            self.lua.create_function(|_, ()| Ok(mouse_position()))?,
        )?;
        cursor.set(
            "down",
            self.lua
                .create_function(|_, ()| Ok(is_mouse_button_down(MouseButton::Left)))?,
        )?;
        cursor.set(
            "pressed",
            self.lua
                .create_function(|_, ()| Ok(is_mouse_button_pressed(MouseButton::Left)))?,
        )?;

        self.lua.globals().set("cursor", cursor)?;
        Ok(())
    }

    fn load_logging_lib(&self) -> LuaResult<()> {
        let name_2 = self.name.to_string();
        self.lua.globals().set(
            "info",
            self.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                info!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = self.name.to_string();
        self.lua.globals().set(
            "warn",
            self.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                warn!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = self.name.to_string();
        self.lua.globals().set(
            "error",
            self.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                error!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = self.name.to_string();
        self.lua.globals().set(
            "debug",
            self.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                debug!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        let name_2 = self.name.to_string();
        self.lua.globals().set(
            "trace",
            self.lua.create_function::<String, (), _>(move |_, text| {
                let span = info_span!("LUA");
                let _enter = span.enter();
                trace!("{}: {}", name_2, text);
                Ok(())
            })?,
        )?;

        Ok(())
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
    fn update_wait_funcs(&self, sudoku: &mut SudokuGame) -> LuaResult<()> {
        let funcs = self.lua.globals().get::<_, Table>("__WAIT_FUNCTIONS__")?;
        let mut remove_keys = vec![];

        for item in funcs.pairs::<Value, Table>() {
            let (key, table) = item?;
            let wanted_ms = table.get::<_, u128>("target")?;
            let func = table.get::<_, Function>("cb")?;
            let interval = table.get::<_, i32>("interval")?;
            let mut repeating = interval != -1;

            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|x| x.as_millis())
                .unwrap_or(0);

            if time < wanted_ms {
                continue;
            }

            self.lua.scope(|scope| {
                let bs = scope.create_userdata_ref_mut(sudoku)?;
                if repeating {
                    repeating = func.call::<_, bool>(bs)?;
                    Ok(())
                } else {
                    func.call(bs)
                }
            })?;

            if repeating {
                table.set("target", time + interval as u128)?;
            } else {
                remove_keys.push(key);
            }
        }

        let funcs = self.lua.globals().get::<_, Table>("__WAIT_FUNCTIONS__")?;
        for key in remove_keys {
            funcs.raw_remove(key)?;
        }

        Ok(())
    }
}

impl LuaRun {
    fn code(&self) -> String {
        match self {
            LuaRun::File { code, .. } | LuaRun::Repl { code, .. } => code.to_string(),
        }
    }

    fn name(&self) -> String {
        match self {
            LuaRun::File { name, .. } => name.to_string(),
            LuaRun::Repl { .. } => "Repl".to_string(),
        }
    }

    fn run(
        &self,
        item: &mut Eval,
        game: &mut SudokuGame,
        status_bar: &StatusBar,
    ) -> LuaResult<Option<String>> {
        let span = span!(Level::INFO, "RunLua");
        let _enter = span.enter();

        let name = self.name();
        let code = self.code();

        match self {
            LuaRun::File { .. } => {
                info!("Executing Lua script: {name}...");
                let scr = LuaScript::exec(&name, &code, status_bar)?;
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

impl Item for Eval {
    fn name(&self) -> &'static str {
        "Eval"
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

                if let Some((idx, _)) = self
                    .scripts
                    .iter()
                    .enumerate()
                    .find(|(_, x)| x.name == file_name)
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

        match code.run(self, game, status_bar) {
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

    fn update(&mut self, game: &mut SudokuGame, status_bar: &mut StatusBar) -> (String, Color) {
        for scr in &self.scripts {
            if let Err(e) = scr.generic_game_callback(game, "__ON_UPDATE_FUNCTIONS__") {
                error!("Lua 'Update' error: {e}");
            }

            if let Err(e) = scr.update_wait_funcs(game) {
                error!("Lua 'WaitMS' error: {e}");
            }
        }

        game.flush_wanted_commands(status_bar);
        (String::new(), WHITE)
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

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::None
    }

    fn shorthands(&self) -> Option<List> {
        shorthand!((r"^=(.*)", "$1"))
    }
}
