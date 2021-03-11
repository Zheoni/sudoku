use std::convert::TryFrom;
use std::time::{Duration, Instant};
use sudoku::SudokuBoard;

fn main() {
    let mut s = SudokuBoard::try_from(
        "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3",
    )
    .unwrap();
    println!("{}", s);
    let now = Instant::now();
    s.solve();
    let elapsed = now.elapsed();
    println!("{}", s);
    println!("Elapsed: {}", elapsed.as_secs_f64());
}
