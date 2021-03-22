//! Contains funcionality of a sudoku puzzle: an unsolved
//! sudoku to present to the user.

use crate::board::SudokuBoard;
use crate::SIZE;

use std::time::{Duration, Instant};

use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::convert::TryFrom;
use std::fmt;

/// Length of the generated IDs
pub const ID_LEN: usize = 16;

/// A sudoku puzzle, a pair of a puzzle and a solution to it. Also gives some
/// stats about the puzzle.
///
/// # Example
///
/// To use it, call [SudokuPuzzle::prepare] and chain all the settings that you want.
/// Look [Generator::default] to see the default values.
/// For example:
/// ```
/// use sudoku::prelude::*;
///
/// let puzzle = SudokuPuzzle::prepare()
///     .with_difficulty(Difficulty::Easy)
///     .with_seed("SUDOKU")
///     .unique_solution(false)
///     .count_solutions(true)
///     .generate();
///
/// println!("{}", puzzle);
/// ```
pub struct SudokuPuzzle {
    /// Puzzle board generated
    pub puzzle: SudokuBoard,
    /// Solution of the board, may be not present
    pub solution: Option<SudokuBoard>,
    /// Stats about the generated puzzle
    pub stats: PuzzleStats,
}

/// Stats about a [SudokuPuzzle]
pub struct PuzzleStats {
    /// Number of empty positions
    pub empty_positions: usize,
    /// Difficulty of the generated puzzle
    pub difficulty: Difficulty,
    /// Number of possible solutions. At most, [MAX_SOLUTIONS_COUNT]
    pub possible_solutions: Option<usize>,
    /// Time durations measured during puzzle generation. `times.0` is the
    /// time taken to generate a complete board, and `times.1` is the time
    /// taken to generate the puzzle form the complete board.
    pub times: (Duration, Duration),
    /// Seed of the puzzle
    pub seed: String,
}

impl SudokuPuzzle {
    /// Create a configurable generator for a puzzle with [Generator::default]
    /// as default values.
    pub fn prepare() -> Generator {
        Generator::default()
    }

    /// Prints the CSV head line when writting a puzzle as csv.
    pub fn csv_head() -> &'static str {
        "puzzle,solution,seed,empty_positions,difficulty,possible_solutions,board_time_us,puzzle_time_us"
    }
}

impl fmt::Display for SudokuPuzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#},", self.puzzle)?;
            if let Some(solution) = self.solution.as_ref() {
                write!(f, "{:#}", solution)?;
            }
            write!(f, ",")?;
            let s = &self.stats;
            write!(
                f,
                "{},{},{},{},{},{}",
                s.seed,
                s.empty_positions,
                s.difficulty,
                if let Some(ps) = s.possible_solutions {
                    ps.to_string()
                } else {
                    String::default()
                },
                s.times.0.as_micros(),
                s.times.1.as_micros(),
            )
        } else {
            write!(f, "{}", self.puzzle)?;
            writeln!(f, "ID: {}", self.stats.seed)?;
            if let Some(solution_count) = self.stats.possible_solutions {
                writeln!(f, "Number of solutions: {}", solution_count)?;
            }
            if let Some(solution) = self.solution.as_ref() {
                write!(f, "Solution:\n{}", solution)?;
            }
            Ok(())
        }
    }
}

/// Configurable [SudokuPuzzle] generator
pub struct Generator {
    unique: bool,
    empty_positions: usize,
    seed: Option<String>,
    difficulty: Difficulty,
    count_solutions: bool,
    max_count_solutions: usize,
    show_solution: bool,
}

impl Generator {
    /// Generate the a puzzle from the generator.
    pub fn generate(&self) -> SudokuPuzzle {
        let seed = self.seed.as_ref().cloned().unwrap_or_else(|| {
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(ID_LEN)
                .map(char::from)
                .collect()
        });

        let mut rng: Pcg64 = Seeder::from(seed.clone()).make_rng();

        let now = Instant::now();
        let solution = SudokuBoard::generate(&mut rng);
        let solution_time = now.elapsed();

        let now = Instant::now();
        let mut puzzle = solution.clone();

        let mut positions: Vec<usize> = (0..SIZE).collect();
        positions.shuffle(&mut rng);

        let mut removed = 0;
        for pos in positions {
            let val = puzzle[pos];
            puzzle[pos] = 0;
            if !self.unique || puzzle.count_solutions(2) == 1 {
                removed += 1;
                if removed >= self.empty_positions {
                    break;
                }
            } else {
                puzzle[pos] = val;
            }
        }
        let puzzle_time = now.elapsed();

        let possible_solutions = if self.count_solutions {
            Some(puzzle.count_solutions(self.max_count_solutions))
        } else {
            None
        };

        let stats = PuzzleStats {
            empty_positions: removed,
            difficulty: self.difficulty.clone(),
            possible_solutions,
            times: (solution_time, puzzle_time),
            seed,
        };

        SudokuPuzzle {
            solution: if self.show_solution {
                Some(solution)
            } else {
                None
            },
            puzzle,
            stats,
        }
    }

    /// Configure if the puzzle should have an unique solution. `true` by default.
    pub fn unique_solution(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    /// Configure the difficulty of the puzzle. [Difficulty::Normal] by default.
    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        use Difficulty::*;
        self.empty_positions = match difficulty {
            Easy => 25,
            Normal => 35,
            Hard => 50,
            Insane => 64,
        };
        self.difficulty = difficulty;
        self
    }

    /// Seeds the puzzle. A random seed with length [ID_LEN] is generated by default.
    ///
    /// The same solved [SudokuBoard] would be generated with this seed.
    pub fn with_seed(mut self, seed: &str) -> Self {
        self.seed = Some(seed.to_string());
        self
    }

    /// Configure whether the solutions of the puzzle are counted. `false` by default
    pub fn count_solutions(mut self, do_count: bool) -> Self {
        self.count_solutions = do_count;
        self
    }

    /// Configure the maximum number of solutions to count
    pub fn max_count_solutions(mut self, max: usize) -> Self {
        self.max_count_solutions = max;
        self
    }

    /// If the solution is returned
    pub fn show_solution(mut self, do_show: bool) -> Self {
        self.show_solution = do_show;
        self
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            unique: true,
            empty_positions: 35,
            difficulty: Difficulty::Normal,
            seed: None,
            count_solutions: false,
            max_count_solutions: 256,
            show_solution: false,
        }
    }
}

/// Difficulty of the puzzles. Currently only changes the number
/// of empty positions.
#[derive(Clone)]
#[allow(missing_docs)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Insane,
}

impl Difficulty {
    /// Returns all the str representations of the difficulty levels
    pub const fn get_all() -> &'static [&'static str] {
        &["easy", "normal", "hard", "insane"]
    }
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "easy" => Ok(Self::Easy),
            "normal" => Ok(Self::Normal),
            "hard" => Ok(Self::Hard),
            "insane" => Ok(Self::Insane),
            _ => Err("Unknown difficulty"),
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Difficulty::*;
        write!(
            f,
            "{}",
            match self {
                Easy => "easy",
                Normal => "normal",
                Hard => "hard",
                Insane => "insane",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_board_with_seed() {
        let puzzle = SudokuPuzzle::prepare()
            .with_seed("TEST")
            .show_solution(true)
            .generate();
        let solution = SudokuBoard::generate_from_seed(&"TEST");

        assert_eq!(puzzle.solution.unwrap(), solution);
    }
}
