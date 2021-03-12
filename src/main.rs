use sudoku::SudokuBoard;

fn main() {
    let s = SudokuBoard::generate_from_seed("SUDOKU");
    println!("{}", s);
}
