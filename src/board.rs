use super::{N, N2, SIZE};
use crate::pos_util::*;

use std::collections::HashSet;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;

use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

/// Represents a sudoku board capable of being solved, count solutions and be
/// generated randomly.
///
/// # Example
/// ```
/// use sudoku::SudokuBoard;
/// use std::convert::TryFrom;
/// let mut sudoku = SudokuBoard::try_from("6.3.581...2.....3.1...3.5.........87.5...26..27.86...4.........4....6.7.5...1..2.").unwrap();
/// sudoku.solve();
/// println!("{}", sudoku);
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SudokuBoard([u8; SIZE]);

#[derive(Clone)]
struct Domains {
    domains: [[bool; N2]; SIZE],
    empty_positions: HashSet<usize>,
}

impl Domains {
    pub fn calculate_domains(board: &SudokuBoard) -> Self {
        let mut d = Self {
            domains: [[true; N2]; SIZE],
            empty_positions: HashSet::new(),
        };

        // for each cell
        for (pos, &value) in board.0.iter().enumerate() {
            // if the cell is assigned
            if value != 0 {
                // set all of its possible values to false
                d.domains[pos].fill(false);
                // update the domains as if the value was just assigned
                d.update_domains(pos, value);
            } else {
                d.empty_positions.insert(pos);
            }
        }
        d
    }

    pub fn update_domains(&mut self, pos: usize, value: u8) {
        assert!(value > 0);
        let value = (value - 1) as usize;

        // in all conflicting indexes (row, col, group) mark the new value as false
        for p in adjacent_positions(pos) {
            self.domains[p][value] = false;
        }

        self.empty_positions.remove(&pos);
    }
}

// Solving

// multiple (and similar) backtracking functions to avoid checking parameters
// to make them behave differently
impl SudokuBoard {
    /// Solves the sudoku in place, returns true if the sudoku could be solved.
    /// Gets the first solution, does not check for more.
    pub fn solve(&mut self) -> bool {
        let mut domains = Domains::calculate_domains(self);
        self.backtracking(&mut domains)
    }

    fn backtracking(&mut self, domains: &mut Domains) -> bool {
        // get the first empty postion
        let pos = self.get_empty_position(domains, SIZE / 2);
        if pos.is_none() {
            // if there's none, we found a solution
            return true;
        }
        let pos = pos.unwrap();
        let mut temp_domains: Domains;

        // try all possible values
        for n in self.get_possible(pos, domains, N) {
            // if the value can be fitted (maybe this check is unnecesary
            // because of get_possible and the domain calculations)
            if self.is_valid(pos, n) {
                // apply the value and update the domains
                self.0[pos] = n;
                temp_domains = domains.clone();
                domains.update_domains(pos, n);
                // if sudoku can still be solved
                if self.still_possible(domains) {
                    // continue searching
                    if self.backtracking(domains) {
                        // solution found
                        return true;
                    }
                }
                // backtrack: restore the position and the domains
                *domains = temp_domains;
            }
        }
        self.0[pos] = 0;

        false
    }

    /// Solves the sudoku finding at most `max` solutions.
    pub fn solve_all(&self, max: usize) -> Vec<SudokuBoard> {
        let mut domains = Domains::calculate_domains(self);
        let mut solutions = Vec::new();
        self.clone()
            .backtracking_all(&mut domains, max, 0, &mut solutions);
        solutions
    }

    fn backtracking_all(
        &mut self,
        domains: &mut Domains,
        max_solutions: usize,
        mut count: usize,
        solutions: &mut Vec<Self>,
    ) -> usize {
        // get the first empty postion
        let pos = self.get_empty_position(domains, SIZE / 2);
        if pos.is_none() {
            // if there's none, we found a solution.
            // add 1 to count
            solutions.push(self.clone());
            return 1 + count;
        }
        let pos = pos.unwrap();
        let mut temp_domains: Domains;

        // try all possible values
        for n in self.get_possible(pos, domains, N) {
            // when the limit is reached, stop searching
            if count >= max_solutions {
                return count;
            }
            // if the value can be fitted (maybe this check is unnecesary
            // because of get_possible and the domain calculations)
            if self.is_valid(pos, n) {
                // apply the value and update the domains
                self.0[pos] = n;
                temp_domains = domains.clone();
                domains.update_domains(pos, n);
                // if sudoku can still be solved
                if self.still_possible(domains) {
                    // continue searching
                    count = self.backtracking_all(domains, max_solutions, count, solutions);
                }
                // backtrack: restore the position and the domains
                *domains = temp_domains;
            }
        }
        self.0[pos] = 0;

        count
    }

    /// Counts the number of solutions of the sudoku.
    /// It stops counting when `max` is reached.
    pub fn count_solutions(&self, max: usize) -> usize {
        let mut domains = Domains::calculate_domains(self);
        self.clone().backtracking_count(&mut domains, max, 0)
    }

    fn backtracking_count(
        &mut self,
        domains: &mut Domains,
        max_solutions: usize,
        mut count: usize,
    ) -> usize {
        // get the first empty postion
        let pos = self.get_empty_position(domains, SIZE / 2);
        if pos.is_none() {
            // if there's none, we found a solution.
            // add 1 to count
            return 1 + count;
        }
        let pos = pos.unwrap();
        let mut temp_domains: Domains;

        // try all possible values
        for n in self.get_possible(pos, domains, N) {
            // when the limit is reached, stop searching
            if count >= max_solutions {
                return count;
            }
            // if the value can be fitted (maybe this check is unnecesary
            // because of get_possible and the domain calculations)
            if self.is_valid(pos, n) {
                // apply the value and update the domains
                self.0[pos] = n;
                temp_domains = domains.clone();
                domains.update_domains(pos, n);
                // if sudoku can still be solved
                if self.still_possible(domains) {
                    // continue searching
                    count = self.backtracking_count(domains, max_solutions, count);
                }
                // backtrack: restore the position and the domains
                *domains = temp_domains;
            }
        }
        self.0[pos] = 0;

        count
    }

    fn get_empty_position(&self, domains: &Domains, min_tie_to_solve: usize) -> Option<usize> {
        // Calculate the number of available values for each empty position
        let mut values: Vec<(u32, usize)> = domains
            .empty_positions
            .iter()
            .map(|&pos| {
                (
                    domains.domains[pos]
                        .iter()
                        .fold(0, |acc, &x| acc + if x { 1 } else { 0 }),
                    pos,
                )
            })
            .collect();

        values.sort_unstable();

        let &(min, mut min_index) = values.first()?;
        let tied: Vec<_> = values.iter().take_while(|(v, _)| *v == min).collect();
        if tied.len() > min_tie_to_solve {
            let mut min_restrictions = usize::MAX;
            for &(_, pos) in tied {
                let mut pos_restrictions = 0;

                for p in adjacent_positions(pos) {
                    if self.0[p] == 0 {
                        pos_restrictions += 1;
                    }
                }

                if pos_restrictions < min_restrictions {
                    min_restrictions = pos_restrictions;
                    min_index = pos;
                }
            }
        }
        Some(min_index)
    }

    fn get_possible(&self, pos: usize, domains: &Domains, min_possible_ordered: usize) -> Vec<u8> {
        let possible: Vec<_> = domains.domains[pos]
            .iter()
            .enumerate()
            .filter(|(_, &possible)| possible)
            .map(|(value, _)| value as u8 + 1)
            .collect();

        if possible.len() > min_possible_ordered {
            let mut values = domains
                .empty_positions
                .iter()
                .map(|&pos| domains.domains[pos])
                .fold([0; N2], |mut acc, domain| {
                    acc.iter_mut()
                        .zip(domain.iter())
                        .for_each(|(accref, x)| *accref += if *x { 0 } else { 1 });
                    acc
                })
                .iter()
                .enumerate()
                .map(|(i, x)| (*x, i as u8 + 1))
                .filter(|(_, x)| possible.contains(x))
                .collect::<Vec<_>>();
            values.sort_unstable();
            values.iter().map(|(_, x)| *x).collect()
        } else {
            possible
        }
    }

    fn still_possible(&self, domains: &Domains) -> bool {
        !domains
            .empty_positions
            .iter()
            .map(|&pos| domains.domains[pos])
            .map(|domain| domain.iter().fold(0, |acc, &x| acc + if x { 1 } else { 0 }))
            .any(|sum| sum == 0)
    }

    /// Checks if `n` can be placed at `pos`. It does not check if that will
    /// produce a dead end, just if its a legal move.
    pub fn is_valid(&self, pos: usize, n: u8) -> bool {
        for p in adjacent_positions(pos) {
            if n == self.0[p] {
                return false;
            }
        }
        true
    }

    /// Checks if `n` can be placed in the row `row`.
    pub fn is_valid_row(&self, row: usize, n: u8) -> bool {
        for p in row_positions(row) {
            if n == self.0[p] {
                return false;
            }
        }
        true
    }

    /// Checks if `n` can be placed in the column `col`.
    pub fn is_valid_col(&self, col: usize, n: u8) -> bool {
        for p in col_positions(col) {
            if n == self.0[p] {
                return false;
            }
        }
        true
    }

    /// Checks if `n` can be placed at the row `row` and the column `col` but
    /// only against the corresponding group.
    pub fn is_valid_group(&self, row: usize, col: usize, n: u8) -> bool {
        for p in group_positions(row, col) {
            if n == self.0[p] {
                return false;
            }
        }
        true
    }
}

// Generate
impl SudokuBoard {
    /// Generates a solved board from a seed.
    pub fn generate_from_seed<T: std::hash::Hash>(seed: T) -> Self {
        let mut rng = Seeder::from(seed).make_rng();
        Self::generate(&mut rng)
    }

    /// Generates a solved board using a PRNG.
    pub fn generate(rng: &mut Pcg64) -> Self {
        // loop while the board is not solved
        let mut solution = Self::default();

        // fill the groups in the main diagonal
        for i in 0..N {
            let mut numbers = (1..=N2 as u8).collect::<Vec<u8>>();
            numbers.shuffle(rng);

            for (p, val) in group_positions(i * N, i * N).zip(numbers) {
                solution.0[p] = val;
            }
        }

        let mut domains = Domains::calculate_domains(&solution);

        // change some random positions to increase randomness
        let sustitutions = rng.gen_range(10..20);
        use std::iter::FromIterator;
        let mut empty_positions = Vec::from_iter(domains.empty_positions.iter().cloned());
        for _ in 0..sustitutions {
            let pos = empty_positions.remove(rng.gen_range(0..empty_positions.len()));
            let mut possible = solution.get_possible(pos, &domains, usize::MAX);
            possible.shuffle(rng);
            loop {
                let value = possible
                    .pop()
                    .expect("Error: No possible value while generating");
                solution.0[pos] = value;

                if solution.count_solutions(1) == 1 {
                    domains.update_domains(pos, value);
                    break;
                }
            }
        }

        solution.solve();
        solution
    }
}

// Interface
impl SudokuBoard {
    /// Returns the 1 line representation of the board.
    /// Scanning row by row. A dot means an empty position.
    pub fn to_line_string(&self) -> String {
        self.0
            .iter()
            .map(|&x| match x {
                0 => ".".to_string(),
                x => x.to_string(),
            })
            .collect()
    }
}

impl Default for SudokuBoard {
    fn default() -> Self {
        SudokuBoard([0; SIZE])
    }
}

impl fmt::Display for SudokuBoard {
    /// Pretty format for a sudoku
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_row(f: &mut fmt::Formatter<'_>, row: &[u8]) -> fmt::Result {
            write!(f, "║")?;
            for (i, &n) in row.iter().enumerate().take(N2) {
                if n != 0 {
                    write!(f, "{: ^3}", n)?;
                } else {
                    write!(f, "   ")?;
                }
                if i % N != N - 1 {
                    write!(f, "│")?;
                } else {
                    write!(f, "║")?;
                }
            }
            writeln!(f)
        }

        fn fmt_border(
            f: &mut fmt::Formatter<'_>,
            left: char,
            num_sep: char,
            group_sep: char,
            right: char,
            regular: char,
        ) -> fmt::Result {
            let num_border: String = std::iter::repeat(regular).take(3).collect();
            write!(f, "{}", left)?;
            for i in 0..N2 {
                write!(f, "{}", num_border)?;
                if i != N2 - 1 {
                    if i % N != N - 1 {
                        write!(f, "{}", num_sep)?;
                    } else {
                        write!(f, "{}", group_sep)?;
                    }
                }
            }
            writeln!(f, "{}", right)
        }

        fmt_border(f, '╔', '═', '╦', '╗', '═')?;

        for i in 0..N2 {
            fmt_row(f, &self.0[i * N2..(i + 1) * N2])?;
            if i != N2 - 1 {
                if i % N != N - 1 {
                    fmt_border(f, '║', '┼', '║', '║', '─')?;
                } else {
                    fmt_border(f, '╠', '═', '╬', '╣', '═')?;
                }
            }
        }
        fmt_border(f, '╚', '═', '╩', '╝', '═')
    }
}

impl TryFrom<&str> for SudokuBoard {
    type Error = &'static str;

    /// Tries to converts a sudoku board string representation into a sudoku board.
    ///
    /// **May not work correctly for SIZE > 3** because in one char does not fit the value
    fn try_from(s: &str) -> Result<SudokuBoard, Self::Error> {
        if s.len() != SIZE {
            return Err("Invalid str len, must be SIZE");
        }

        s.chars()
            .map(|c| match c {
                '.' => Ok(0),
                c => c.to_digit(10).ok_or("Invalid character").map(|d| d as u8),
            })
            .collect::<Result<Vec<u8>, Self::Error>>()
            .and_then(|v| v.try_into())
    }
}

impl TryFrom<[u8; SIZE]> for SudokuBoard {
    type Error = &'static str;

    fn try_from(arr: [u8; SIZE]) -> Result<SudokuBoard, Self::Error> {
        if arr.iter().all(|&d| d <= N2 as u8) {
            Ok(SudokuBoard(arr))
        } else {
            Err("Values must be between 0 and sqrt(SIZE)")
        }
    }
}

impl TryFrom<Vec<u8>> for SudokuBoard {
    type Error = &'static str;
    fn try_from(vec: Vec<u8>) -> Result<SudokuBoard, Self::Error> {
        vec.as_slice().try_into()
    }
}

impl TryFrom<&[u8]> for SudokuBoard {
    type Error = &'static str;
    fn try_from(slice: &[u8]) -> Result<SudokuBoard, Self::Error> {
        <[u8; SIZE]>::try_from(slice)
            .map_err(|_| "Could not convert slice to array")
            .and_then(SudokuBoard::try_from)
    }
}

impl std::ops::Index<usize> for SudokuBoard {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl std::ops::IndexMut<usize> for SudokuBoard {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn line_string() {
        let s = SudokuBoard::try_from(
            "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3",
        )
        .unwrap();
        assert_eq!(
            s.to_line_string(),
            "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3"
        )
    }

    #[test]
    fn line_string_zeroes() {
        let s = SudokuBoard::try_from(
            "002....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3",
        )
        .unwrap();
        assert_eq!(
            s.to_line_string(),
            "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3"
        )
    }

    #[test]
    fn solve_1() {
        let mut s = SudokuBoard::try_from(
            "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3",
        )
        .unwrap();
        s.solve();
        assert_eq!(
            s.to_line_string(),
            "542971638917386254836542791723859146469123875158467329384715962695238417271694583"
        );
    }

    #[test]
    fn solve_all_1() {
        let s = SudokuBoard::try_from(
            "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3",
        )
        .unwrap();
        let solutions = s.solve_all(10);
        assert_eq!(solutions.len(), 1);
        assert_eq!(
            solutions[0].to_line_string(),
            "542971638917386254836542791723859146469123875158467329384715962695238417271694583"
        );
    }

    #[test]
    fn count_1() {
        let s = SudokuBoard::try_from(
            "..2....3.....86.5..365...91........6.691...7...8............9.......8.17.716..5.3",
        )
        .unwrap();
        assert_eq!(s.count_solutions(10), 1);
    }

    #[test]
    fn solve_all_more() {
        let s = SudokuBoard::try_from(
            "9265714833514862798749235165823671941492582677631..8252387..651617835942495612738",
        )
        .unwrap();
        let solutions = s.solve_all(10);
        assert_eq!(solutions.len(), 2);
        let solutions = solutions
            .iter()
            .map(|s| s.to_line_string())
            .collect::<Vec<_>>();
        assert!(solutions.contains(
            &"926571483351486279874923516582367194149258267763194825238749651617835942495612738"
                .to_string()
        ));
        assert!(solutions.contains(
            &"926571483351486279874923516582367194149258267763149825238794651617835942495612738"
                .to_string()
        ));
    }

    #[test]
    fn count_more() {
        let s = SudokuBoard::try_from(
            "9265714833514862798749235165823671941492582677631..8252387..651617835942495612738",
        )
        .unwrap();
        assert_eq!(s.count_solutions(10), 2);
    }

    #[test]
    fn generate() {
        use rand::SeedableRng;
        use rand_pcg::Pcg64;
        let s = SudokuBoard::generate(&mut Pcg64::from_entropy());
        for (pos, &val) in s.0.iter().enumerate() {
            assert_ne!(val, 0);
            assert!(s.is_valid(pos, val));
        }
    }
}
