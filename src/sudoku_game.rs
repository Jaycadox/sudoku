use ndarray::{s, Array2, ArrayView, ArrayView2, Axis, Ix1};

use crate::status_bar::StatusBar;

pub struct SudokuGame {
    pub cells: Array2<u8>,
    pub unradified: Vec<u8>,
    pub selected_cell: Option<(u32, u32)>,
    pub reset_signalled: bool,
}

impl Clone for SudokuGame {
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            unradified: self.unradified.clone(),
            selected_cell: self.selected_cell,
            reset_signalled: self.reset_signalled,
        }
    }
}

impl SudokuGame {
    pub fn new(status_bar: Option<&mut StatusBar>) -> Self {
        let mut cells = Array2::zeros((9, 9));
        let inp =
            ".................................................................................";
        let inp = inp.replace('.', "0");
        let mut unradified = Vec::new();
        for (i, cell) in cells.iter_mut().enumerate() {
            let val = inp.chars().nth(i).unwrap().to_digit(10).unwrap() as u8;
            *cell = val;
            if val == 0 {
                unradified.push(i as u8);
            }
        }
        let mut game = SudokuGame {
            cells,
            unradified,
            selected_cell: None,
            reset_signalled: false,
        };

        if let Some(status_bar) = status_bar {
            status_bar.restart(&mut game);
        }

        game
    }

    fn generate_unradified(cells: &Array2<u8>) -> Vec<u8> {
        let mut unradified = Vec::new();
        for (i, cell) in cells.iter().enumerate() {
            if *cell == 0 {
                unradified.push(i as u8);
            }
        }
        unradified
    }

    pub fn reset(&mut self, mut to_state: SudokuGame) {
        for unradified in to_state.unradified.clone() {
            *to_state.cells.iter_mut().nth(unradified as usize).unwrap() = 0;
        }

        *self = to_state;
        self.unradified = Self::generate_unradified(&self.cells);
        self.reset_signalled = true;
    }

    #[allow(dead_code)]
    pub fn print_board(&self) {
        println!("{}", self.board_string());
    }

    #[allow(dead_code)]
    pub fn board_string(&self) -> String {
        let mut buf = String::new();
        for row in self.rows() {
            for cell in row {
                buf.push_str(&format!("{} ", *cell));
            }
            buf.push('\n');
        }

        buf
    }

    #[inline(always)]
    pub fn xy_pos_to_idx(x: u32, y: u32, size: u32) -> u32 {
        y * size + x
    }

    #[inline(always)]
    pub fn idx_pos_to_xy(idx: u32, size: u32) -> (u32, u32) {
        let x = idx % size;
        let y = idx / size;

        (x, y)
    }

    pub fn get_cells_in_sight(&self, cell_pos: (u32, u32)) -> Vec<u32> {
        let (sx, sy) = cell_pos;
        let (box_x, box_y) = (sx / 3, sy / 3);

        let mut cells_in_sight = Vec::with_capacity(21);
        let grid_length = self.cells.shape()[1] as u32;

        for grid_pos in 0..grid_length {
            cells_in_sight.push(sy * grid_length + grid_pos);
            cells_in_sight.push(grid_pos * grid_length + sx);
        }

        for inner_box_x in 0..3 {
            for inner_box_y in 0..3 {
                let (inner_sx, inner_sy) = (box_x * 3 + inner_box_x, box_y * 3 + inner_box_y);
                if inner_sx != sx && inner_sy != sy {
                    cells_in_sight.push(Self::xy_pos_to_idx(inner_sx, inner_sy, grid_length));
                }
            }
        }

        cells_in_sight
    }

    pub fn get_cells_which_see_number_at_pos(&self, cell_pos: (u32, u32)) -> Vec<u32> {
        let (sx, sy) = cell_pos;
        let current_selected = self.cells[(sy as usize, sx as usize)];
        self.get_all_cells_which_see_number(current_selected)
    }

    pub fn get_all_cells_which_see_number(&self, number: u8) -> Vec<u32> {
        let mut highlight_cells = Vec::with_capacity(21);
        let current_selected = number;
        if current_selected != 0 {
            let mut same_cells = Vec::new();
            for (i, cell) in self.cells.iter().enumerate() {
                if *cell == current_selected {
                    let (sx, sy) = Self::idx_pos_to_xy(i as u32, self.cells.shape()[1] as u32);
                    let box_coord = (sx / 3, sy / 3);
                    for bx in 0..3 {
                        for by in 0..3 {
                            let bx = box_coord.0 * 3 + bx;
                            let by = box_coord.1 * 3 + by;
                            highlight_cells.push(Self::xy_pos_to_idx(
                                bx,
                                by,
                                self.cells.shape()[1] as u32,
                            ));
                        }
                    }
                    same_cells.push(i as u32);
                }
            }
            for (i, _) in self.cells.iter().enumerate() {
                let rx = i % self.cells.shape()[1];
                let ry = i / self.cells.shape()[1];
                for scell in &same_cells {
                    let sx = *scell as usize % self.cells.shape()[1];
                    let sy = *scell as usize / self.cells.shape()[1];
                    if rx == sx || ry == sy {
                        highlight_cells.push(i as u32);
                    }
                }
            }
        }
        highlight_cells
    }

    pub fn rows(&self) -> Vec<ArrayView<u8, Ix1>> {
        (0..9)
            .map(|i| self.cells.index_axis(Axis(0), i))
            .collect::<Vec<_>>()
    }
    pub fn cols(&self) -> Vec<ArrayView<u8, Ix1>> {
        (0..9)
            .map(|i| self.cells.index_axis(Axis(1), i))
            .collect::<Vec<_>>()
    }
    pub fn boxes(&self) -> Array2<ArrayView2<u8>> {
        let boxes = (0..3)
            .flat_map(|i| {
                (0..3).map(move |j| self.cells.slice(s![i * 3..(i + 1) * 3, j * 3..(j + 1) * 3]))
            })
            .collect::<Vec<_>>();
        Array2::from_shape_vec((3, 3), boxes).expect("bad vector shape")
    }
}
