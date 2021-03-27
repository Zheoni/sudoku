#![deny(missing_docs)]

//! # Sudoku
//! Library to solve and generate sudokus.
//!
//! - For solving sudokus: [board::SudokuBoard]
//! - For generating sudokus: [puzzle::SudokuPuzzle]
//!
//! # Quick start
//! ## Solving a sodoku
//! ```
//! use sudoku::prelude::*;
//!
//! // Create a sudoku from a sudoku string i.e. scanning row
//! // by row and '.' are empty spaces.
//! let mut sudoku = SudokuBoard::try_from(
//!     "4..8.....87..12...1..53..8....3........2.97.192....4.3...1973....9......7..4....2"
//! ).unwrap();
//!
//! // This will solve the sudoku in place.
//! sudoku.solve();
//!
//! println!("{}", sudoku);
//! ```
//! ## Generating a sudoku
//! ```
//! use sudoku::prelude::*;
//!
//! let mut puzzle = SudokuPuzzle::prepare()
//!     .with_given_difficulty(Difficulty::Hard)
//!     .generate();
//! println!("{}", puzzle);
//! ```

pub mod board;
mod pos_util;
pub mod prelude;
pub mod puzzle;

const N: usize = 3;
const N2: usize = N * N;
/// Size of the board
pub const SIZE: usize = N2 * N2;
