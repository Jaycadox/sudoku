# Sudoku
A cross-platform, native Sudoku app with integrated board generation and board solving.

## Downloading
Download the pre-compiled latest release [here](https://github.com/Jaycadox/sudoku/releases/latest)

## Usage guide
### First launch
At first launch, you will be met with a default Sudoku board and a status bar at the bottom of your screen, to generate a new board, perform the following
1. (optional) To set the number of filled tiles (up to 81 on a 9x9 grid), press the left control key and then type the number of desired tiles, you should see that number displayed at the end of the status bar, then press left control again to leave buffer enter mode.
2. To start board generation, press F2, this will indicate the progress of the generation task and will fill the board once completed. (the F2 is used because it is by default the second item in your status bar, you can press FX where X is the location of the status bar item inside of your status bar to activate that specific item, for example, by default, you can press F1 which will activate the solver and solve the board)

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
