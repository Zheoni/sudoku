use std::convert::TryFrom;
use sudoku::SudokuBoard;

fn main() {
    let mut s = SudokuBoard::try_from(
        "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3",
    )
    .unwrap();
    println!("{}", s);
    s.solve();
    println!("{}", s);
}
