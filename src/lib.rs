mod board;
mod pos_util;

const N: usize = 3;
const N2: usize = N * N;
pub const SIZE: usize = N2 * N2;

pub use board::SudokuBoard;
