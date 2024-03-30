#![allow(clippy::similar_names)]
use std::default::Default;
use std::str::FromStr;

use macroquad::color::Color;
use macroquad::input::{is_mouse_button_down, mouse_position, MouseButton};
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::is_mouse_button_pressed;
use macroquad::shapes::draw_rectangle;
use macroquad::window::{screen_height, screen_width};
use mlua::prelude::{LuaResult, LuaUserData, LuaUserDataMethods};
use mlua::Error::RuntimeError;
use mlua::{FromLuaMulti, Function, Lua, LuaOptions, StdLib, Table, Value};
use tracing::{debug, error, info, info_span, span, trace, warn, Level};

use crate::draw_helper::{draw_text_in_bounds, get_status_bar_height, DrawingSettings};
use crate::status_bar::shorthands::list::List;
use crate::sudoku_game::SudokuGame;
use crate::{config, AppColour};

use super::{cpu_solve, Item, ItemOkData, ItemStatus, StatusBar};

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
pub struct Eval;

enum LuaRun {
    File {
        code: String,
        allow_duplicate: bool,
        name: String,
    },
    Repl {
        code: String,
    },
}

struct LuaScript {
    name: String,
    lua: Lua,
}

impl Item for LuaScript {
    fn name(&self) -> String {
        self.generic_single_callback::<String>(None, "__ON_NAME_FUNCTION")
            .unwrap_or_else(|_| self.name.clone())
    }

    fn update(&mut self, game: &mut SudokuGame, status_bar: &mut StatusBar) -> (String, Color) {
        if let Err(e) = self.generic_game_callback(game, "__ON_UPDATE_FUNCTIONS__") {
            error!("Lua 'Update' error: {e}");
        }

        if let Err(e) = self.update_wait_funcs(game) {
            error!("Lua 'WaitMS' error: {e}");
        }

        game.flush_wanted_commands(status_bar);

        let (text, colour) = self
            .generic_single_callback::<(String, String)>(Some(game), "__ON_UPDATE_FUNCTION")
            .unwrap_or((String::new(), "StatusBarItemOkay".to_string()));
        (
            text,
            status_bar
                .drawing
                .colour(AppColour::from_str(&colour).unwrap_or_else(|()| {
                    warn!(
                        "Script {} attempted to use invalid status bar colour: {}",
                        self.name, colour
                    );
                    AppColour::StatusBarItemError
                })),
        )
    }

    fn activated(&mut self, game: &mut SudokuGame, status_bar: &mut StatusBar) {
        let status_bar_content = status_bar.buffer.clone();
        let Ok(func) = self
            .lua
            .globals()
            .get::<_, Function>("__ON_ACTIVATE_FUNCTION")
        else {
            error!("Lua '{}' error on activated (not found)", self.name);
            return;
        };
        if let Err(e) = self.lua.scope(|scope| {
            let bs = scope.create_userdata_ref_mut(game).unwrap();
            func.call::<_, ()>((bs, status_bar_content))
        }) {
            error!("Lua '{}' error: {}", self.name, e);
        }
    }
    fn board_init(&mut self, game: &mut SudokuGame, _status_bar: &mut StatusBar) {
        let span = span!(Level::INFO, "RunLua");
        let _enter = span.enter();

        if let Err(e) = self.generic_game_callback(game, "__ON_BOARDGEN_FUNCTIONS__") {
            error!("Lua 'BoardInit' error: {e}");
        }
    }

    fn display_mode(&self) -> super::DisplayMode {
        let display_mode = self
            .lua
            .globals()
            .get::<_, String>("__DISPLAY_MODE")
            .unwrap_or("Normal".to_string());

        match display_mode.as_str() {
            "Normal" => super::DisplayMode::Normal,
            "NameOnly" => super::DisplayMode::NameOnly,
            "StatusOnly" => super::DisplayMode::StatusOnly,
            "None" => super::DisplayMode::None,
            _ => panic!(
                "Invalid display mode '{}' requested by script '{}'",
                display_mode, self.name
            ),
        }
    }

    fn status(&mut self) -> super::ItemStatus {
        super::ItemStatus::Ok(ItemOkData::LuaScript(self.name.clone()))
    }
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

script = {}

__ON_NAME_FUNCTION = {}
script["on_name"] = function(callback)
    __ON_NAME_FUNCTION = callback
end

__ON_UPDATE_FUNCTION = {}
script["on_update"] = function(callback)
    __ON_UPDATE_FUNCTION = callback
end

__ON_ACTIVATE_FUNCTION = function(_game, _buffer) end
script["on_activate"] = function(callback)
    __ON_ACTIVATE_FUNCTION = callback
end

__DISPLAY_MODE = "Normal"
script["display_none"] = function() __DISPLAY_MODE = "None" end
script["display_normal"] = function() __DISPLAY_MODE = "Normal" end
script["display_name_only"] = function() __DISPLAY_MODE = "NameOnly" end
script["display_status_only"] = function() __DISPLAY_MODE = "StatusOnly" end

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

    fn generic_single_callback<'lua, T: FromLuaMulti<'lua>>(
        &'lua self,
        sudoku: Option<&mut SudokuGame>,
        name: &str,
    ) -> LuaResult<T> {
        let func = self.lua.globals().get::<_, Function>(name)?;

        self.lua.scope(|scope| match sudoku {
            Some(sudoku) => {
                let bs = scope.create_userdata_ref_mut(sudoku)?;
                func.call::<_, T>(bs)
            }
            None => func.call::<_, T>(()),
        })
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

    fn run(&self, game: &mut SudokuGame, status_bar: &mut StatusBar) -> LuaResult<Option<String>> {
        let span = span!(Level::INFO, "RunLua");
        let _enter = span.enter();

        let name = self.name();
        let code = self.code();

        match self {
            LuaRun::File {
                allow_duplicate, ..
            } => {
                info!("Executing Lua script: {name}...");
                let scr = LuaScript::exec(&name, &code, status_bar)?;
                scr.generic_game_callback(game, "__ON_INIT_FUNCTIONS__")?;

                let mut remove = None;

                if !allow_duplicate {
                    for (i, item) in status_bar.items.iter_mut().enumerate() {
                        if let ItemStatus::Ok(ItemOkData::LuaScript(sc_name)) = item.status() {
                            if name == sc_name {
                                remove = Some(i);
                                break;
                            }
                        }
                    }
                }

                if let Some(i) = remove {
                    warn!("Script with name '{name}' is already loaded, replacing...");
                    status_bar.items[i] = Box::new(scr);
                } else {
                    status_bar.items.push(Box::new(scr));
                }

                Ok(None)
            }
            LuaRun::Repl { .. } => {
                let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())?;
                lua.scope(|ctx| {
                    let ud_game = ctx.create_userdata_ref_mut(game)?;
                    lua.globals().set("Game", ud_game)?;
                    let chunk = lua.load(code).set_name(name);
                    chunk.eval::<String>()
                })
                .map(Some)
            }
        }
    }
}

impl Item for Eval {
    fn name(&self) -> String {
        "Eval".to_string()
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
                let mut file_name = file_name.to_string();
                let mut allow_duplicate = false;
                if file_name.starts_with('!') {
                    allow_duplicate = true;
                    file_name.remove(0);
                }

                let Ok(file) = std::fs::read_to_string(config::get_file_path(&file_name)) else {
                    status_bar.buffer = format!("FileNotFound: {file_name}");
                    return;
                };

                LuaRun::File {
                    code: file,
                    allow_duplicate,
                    name: file_name.to_string(),
                }
            }
            _ => LuaRun::Repl {
                code: status_bar.buffer.to_string(),
            },
        };

        match code.run(game, status_bar) {
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

    fn display_mode(&self) -> super::DisplayMode {
        super::DisplayMode::None
    }

    fn shorthands(&self) -> Option<List> {
        shorthand!((r"^=(.*)", "$1"))
    }
}
