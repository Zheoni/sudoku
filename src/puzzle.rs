use crate::board::SudokuBoard;
use crate::SIZE;

use std::time::{Duration, Instant};

use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::fmt;

pub const MAX_SOLUTIONS_COUNT: usize = 256;
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
    pub solution: SudokuBoard,
    pub puzzle: SudokuBoard,
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

    /// Returns a string formatted to display the puzzle to a user.
    pub fn to_puzzle_string(&self, show_solution: bool) -> String {
        let mut s = String::new();

        s.push_str(&format!("{}", self.puzzle));
        s.push_str(&format!("ID: {}", self.stats.seed));
        if let Some(solution_count) = self.stats.possible_solutions {
            s.push_str(&format!(
                "\nNumber of solutions: {}\n",
                if solution_count < MAX_SOLUTIONS_COUNT {
                    solution_count.to_string()
                } else {
                    format!("+{}", MAX_SOLUTIONS_COUNT - 1)
                }
            ))
        }
        if show_solution {
            s.push_str(&format!("\nSolution: \n{}", self.solution));
        }

        s
    }
}

impl fmt::Display for SudokuPuzzle {
    /// Writes `[to_puzzle_string](false)`
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_puzzle_string(false))
    }
}

pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Insane,
}

/// Configurable [SudokuPuzzle] generator
pub struct Generator {
    unique: bool,
    empty_positions: usize,
    seed: Option<String>,
    difficulty: Difficulty,
    count_solutions: bool,
}

impl Generator {
    /// Generate the a puzzle from the generator.
    pub fn generate(self) -> SudokuPuzzle {
        let seed = self.seed.unwrap_or_else(|| {
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
                if removed == self.empty_positions {
                    break;
                }
            } else {
                puzzle[pos] = val;
            }
        }
        let puzzle_time = now.elapsed();

        let possible_solutions = if self.count_solutions {
            Some(puzzle.count_solutions(MAX_SOLUTIONS_COUNT))
        } else {
            None
        };

        let stats = PuzzleStats {
            empty_positions: self.empty_positions,
            difficulty: self.difficulty,
            possible_solutions,
            times: (solution_time, puzzle_time),
            seed,
        };

        SudokuPuzzle {
            solution,
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
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            unique: true,
            empty_positions: 35,
            difficulty: Difficulty::Normal,
            seed: None,
            count_solutions: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_board_with_seed() {
        let puzzle = SudokuPuzzle::prepare().with_seed("TEST").generate();
        let solution = SudokuBoard::generate_from_seed("TEST");

        assert_eq!(puzzle.solution, solution);
    }
}
