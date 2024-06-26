# Sudoku

A cross-platform, native Sudoku app with integrated board generation, board solving, and Lua scripting.

## Downloading

Download the pre-compiled latest release [here](https://github.com/Jaycadox/sudoku/releases/latest)

## Usage guide

### First launch

At first launch, you will be met with a default Sudoku board and a status bar at the bottom of your screen, to generate
a new board, perform the following

1. (optional) To set the number of filled tiles (up to 81 on a 9x9 grid), press the left control key and then type the
   number of desired tiles, you should see that number displayed at the end of the status bar, then press left control
   again to leave buffer enter mode.
2. To start board generation, press F2, this will indicate the progress of the generation task and will fill the board
   once completed. (the F2 is used because it is by default the second item in your status bar, you can press FX where X
   is the location of the status bar item inside of your status bar to activate that specific item, for example, by
   default, you can press F1 which will activate the solver and solve the board)

### Default keybindings

| Key                         | Action                                                            |
|-----------------------------|-------------------------------------------------------------------|
| WASD/Arrows                 | Move selected tile on board in specified direction                |
| Tab                         | Clear all user placed tiles on board                              |
| Left control                | Activates buffer edit mode whilst held                            |
| Escape                      | Clears buffer if in buffer edit mode                              |
| 1-9                         | Set selected tile to specified number                             |
| Backspace/Del/0             | Clears currently selected cell                                    |
| F(x)                        | Activates status bar item of position X                           |
| Space                       | Auto fills the selected tile with the only legal move if possible |
| Left control + Left alt + V | Pastes into buffer                                                |
| Enter                       | Runs the command in the buffer                                    |
| Left control + LShift + tab | Hard reset, reloads config                                        |

### Open config directory

Enter buffer edit mode by pressing `Control`, then type `config` and press enter. A file explorer at the config
directory should have opened.

### Status bar modules

#### `BuiltinAdd`

* Syntax: `BuiltinAdd [module_name]`
* `BuiltinAdd` is the only status bar module added for you automatically, and it can be used to add other status bar
  modules
    * Shorthand: `+[module_name]`
* It is invisible on the status bar

#### `BackgroundImage`

* Syntax: `BackgroundImage ([file_name])|(opacity=[1-255])`
* Overwrites the default (black) background image, supports various file formats and searches in your config directory
* It is invisible on the status bar

#### `BoardGen`

* Syntax: `BoardGen ([num_filled_tiles]?)|([flat_board_str])`
    * Shorthand: `[flat_board_str]` (must be 81 characters long)
* Generates a Sudoku board with the specified amount of filled tiles (default = 30), or sets the board to the given
  board string

#### `ColourOverwrite`

* Syntax: `ColourOverwrite [colour_name] #[hex_colour]`
* Sets app colour of `colour_name` to `hex_colour`
* Valid colours:
    * Background
    * StatusBar
    * StatusBarSeparator
    * StatusBarItemSelected
    * StatusBarItemOkay
    * StatusBarItemInProgress
    * StatusBarItemError
    * StatusBarBufferEdit
    * StatusBarItem
    * BoardBox
    * BoardLine
    * BoardCellBackground
    * BoardSelectedCellBackground
    * BoardHighlightedCellBackground
    * BoardMousedCellBackground
    * BoardRadifiedCell
    * BoardCorrectCell
    * BoardIncorrectCell
    * BoardUnknownCell
* It is invisible on the status bar

#### `Eval`

* See [Scripting information](#scripting-information)

#### `CpuSolve`

* Syntax: `CpuSolve (run)?`
* Providing the `run` argument will compute the solved board, whilst providing no arguments will set the current board
  state to the computed solved board

#### `Find`

* Syntax: `Find \.?[number]`
    * Shorthand: `\.?[0-9]`
* Moves cursor to next (or previous, if `.` was specified) occurrence of `[number]`

#### `Font`

* Syntax: `Font [font_file] [font_size]`
* Sets display font to `font_file`, with size `font_size`
* It is invisible on the status bar

#### `Fps`

* Syntax: `Fps [target]?`
    * Shorthand: `[number]fps`
* Sets FPS limit to `target`, if not specified, FPS limit is removed

#### `HardReset`

* Syntax: `HardReset`
* Performs hard reset, as if you pressed `Control + Shift + Tab`

#### `OnBoardInit`

* Syntax: `OnBoardInit [cmd]`
* Adds `cmd` to a list of commands to be ran whenever a new board is generated
* It is invisible on the status bar

#### `Padding`

* Syntax: `Padding [start] [target] [speed]`
* Specifies animation for Sudoku board padding after board generation, going from `start` to `target` at `speed`
* It is invisible on the status bar

#### `PencilMarks`

* Syntax: `PencilMarks [number]`
* Displays available cell values for empty cells in the corner, `number` is the maximum number of available cells for it
  to start displaying

### Scripting information

To use Lua scripting, `Eval` must be added to the status bar, this can be done by either:

* Editing your `.sudokurc` to contain `BuiltinAdd Eval`
* Opening the buffer editor by pressing `Control`, and by typing `+Eval` (Note: this would need to be done
  every reload/restart)

There are two scripting modes, `Repl` and `File`.

#### Repl

A `Repl` is used to quickly evaluate a Lua expression, and write the
result to the buffer. Example usages:

* After entering buffer edit mode, you can type `Eval 1+1`, and it'll output `2` into the buffer
* The shorthand could also be used, whereby your buffer can start with `=`, and the text after will be piped into
  the `Eval` item. As such, `=1+1` will output `2` to the buffer
* In a `Repl` context, you have access to a global variable called `Game`, which is of type `Game` (userdata)

#### File

A `File` is used to load an entire Lua script, without necessarily printing it to the buffer (though it is possible
through the API). Scripts are searched for inside of your config directory. There are multiple ways to load script
files.

* Qualifying the `Eval` command with an `@` argument suffix, example: `Eval @script.lua`
* Shorthand can also be used, `=@script.lua`
* Either of the above can be placed inside your `.sudokurc`

Note that if you try to load a script, whereby a script of the same name has already been loaded, the previous one would
unload, and the new script would attempt to load. This behaviour can be bypassed by prefixing the script name with `!`, example: `=@!script.lua`, thus allowing multiple instances of the same Lua script at the same time.

A script of type `File` behaves as if it were a regular status bar item, meaning it can be invoked via the status bar or buffer input, and can respond to activations and read the buffer state and whatnot.

## Scripting API usage

### logging (global namespace)

#### `info(text: string)`

* Prints your message to the console with the `INFO` level. Note that this is the replaces the behaviour when
  using `print`

#### `warn(text: string)`

* Prints your message to the console with the `WARN` level.

#### `error(text: string)`

* Prints your message to the console with the `ERROR` level.

#### `debug(text: string)`

* Prints your message to the console with the `DEBUG` level. Note that this will only print text to the console when the
  application is compiled in `Debug` mode.

#### `trace(text: string)`

* Prints your message to the console with the `TRACE` level. Note that this will only print text to the console when the
  application is compiled in `Debug` mode.

### script

#### Note: Most functions which accept callbacks will add your callback to a list of callbacks, and then invoke them all. This is NOT the case for `script` function callbacks, as there can be only one implementation of their underlying function, due to ambiguities regarding callback return values.

#### `script.on_name(callback: function() -> string)`

* Invokes callback every frame, and uses the returned string for the name in the status bar

#### `script.on_update(callback: function(game: Game) -> (string, string))`

* Invokes callback every frame, first tuple string denotes item status text, and the second string denotes colour, of which valid colours are specified under the `ColourOverwrite` section

#### `script.on_activate(callback: function(game: Game, buffer: string))`

* Invokes callback when item is activated either manually or via the buffer, also provides the current state of the buffer

#### `script.display_none()`

* Makes script not display on status bar

#### `script.display_normal()`

* Makes script display name and status text on status bar

#### `script.display_name_only()`

* Makes script display name only on status bar

#### `script.display_status_only()`

* Makes script display status only on status bar

### events

#### `events.on_init(callback: function(Game))`

* Invokes callback when script properly starts, while also passing the current `Game`. Note that this is the only way of
  getting an instance of `Game` on script load.

#### `events.on_board_gen(callback: function(Game))`

* Invokes callback whenever a new board has been generated, and provides the new `Game`.

#### `events.on_update(callback: function(Game))`

* Invokes callback every frame, and provides the new `Game`.

#### `events.wait_ms(ms: int, callback: function(Game))`

* Invokes callback after `ms` milliseconds, and provides the current `Game`.

#### `events.repeat_ms(ms: int, callback: function(Game) -> boolean)`

* Repeatedly nvokes callback after `ms` milliseconds, and provides the current `Game`. Callback function returns `true`
  to continue repeating, and `false` to stop.

### Drawing

#### `drawing.screen_size() -> (int, int)`

* Returns size of screen/window (width, height) in pixels.

#### `drawing.game_size() -> (int, int)`

* Returns size of game (width, height) in pixels. In practice, this will return the size of the screen, minus the status
  bar.

#### `drawing.game_origin() -> (int, int)`

* Returns the origin of the game (x, y) in pixels. In practice, this is usually (0, 0).

#### `drawing.status_bar_size() -> (int, int)`

* Returns the size of the status bar (width, height) in pixels.

#### `drawing.status_bar_origin() -> (int, int)`

* Returns the origin of the status bar (x, y) in pixels.

#### `drawing.colour(name: string) -> (r, g, b, a)`

* Returns rgba values for `AppColour` of `name`. See `ColourOverwrite` for names.

#### `drawing.draw_rect(x: int, y: int, w: int, h: int, r: int, g: int, b: int, a: int)`

* Draws a rectangle at the specified location of the specified size, and the specified colour. Note that `x, y, w, h`
  are pixel values, and `r, g, b, a` are floats (0-1).

#### `drawing.draw_text(text: string, x: int, y: int, size: int, r: int, g: int, b: int, a: int)`

* Draws text with the active display font at requested position and with requested colour. Note that `x, y` are pixel
  values and `r, g, b, a` are floats (0-1).

### Cursor

#### `cursor.position() -> (int, int)`

* Returns x, y position of cursor in pixels

#### `cursor.down() -> boolean`

* Returns true if the left mouse button is down

#### `cursor.pressed() -> boolean`

* Returns true if the left mouse button was pressed down on the same frame

### Game (userdata)

#### `game:pretty_board_string() -> string`

* Returns a newline formatted version of the `Game`'s board state

#### `game:cells() -> Table<int, int>`

* Returns a flat table of the `Game`'s cell indexes, and cell values.

#### `game:unoccupied_cells_at(index: int) -> Table<int, int>`

* Returns a table of the numbers which could possibly be placed at a given cell index in a `Game`. Note that this
  function counts from 0, while Lua counts from 1. Table is of number index, number.

#### `game:board_string() -> string`

* Returns flat representation of board state as string. This format is used by other functions and the `BoardGen` item.

#### `game:solve() -> Game`

* Uses `CpuSolve` and returns a copy of the game, but in a solved state. The string board string can be obtained from
  this and set
  for the original `Game`, if one wishes to modify the current board.

#### `game:is_solved() -> boolean`

* Returns whether the current game is in a solved state

#### `game:update_board_from_string(board: string)`

* Sets the current game board state to the string board state, while keeping the list of ratified cells.

#### `game:new_from_string(board: string)`

* Resets the current game, and treats the new cells as ratified.

#### `game:enter_buffer_command(cmd: string)`

* Submits the buffer with `cmd`. Note that this will probably occur on the next frame, and at current, there is no way
  to ascertain the output of the command.
