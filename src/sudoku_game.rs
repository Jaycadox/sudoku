use ndarray::{s, Array2, ArrayView, ArrayView2, Axis, Ix1};

#[derive(Clone)]
pub struct SudokuGame {
    pub cells: Array2<u8>,
    pub unradified: Vec<u8>,
    pub selected_cell: Option<(u32, u32)>,
}

impl SudokuGame {
    pub fn new() -> Self {
        let mut cells = Array2::zeros((9, 9));
        let inp =
            "050010040107000602000905000208030501040070020901080406000401000304000709020060010";
        let inp = inp.replace('.', "0");
        let mut unradified = Vec::new();
        for (i, cell) in cells.iter_mut().enumerate() {
            let val = inp.chars().nth(i).unwrap().to_digit(10).unwrap() as u8;
            *cell = val;
            if val == 0 {
                unradified.push(i as u8);
            }
        }
        SudokuGame {
            cells,
            unradified,
            selected_cell: None,
        }
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
