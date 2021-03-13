use sudoku::prelude::*;

fn main() {
    let puzzle = SudokuPuzzle::prepare().generate();
    println!("{}", puzzle);
}
