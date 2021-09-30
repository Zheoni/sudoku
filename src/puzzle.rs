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
///     .with_given_difficulty(Difficulty::Easy)
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
                "{seed},{empty},{difficulty:#},{possible_sol},{boardtime},{puzzletime}",
                seed = s.seed,
                empty = s.empty_positions,
                difficulty = s.difficulty,
                possible_sol = if let Some(ps) = s.possible_solutions {
                    ps.to_string()
                } else {
                    String::default()
                },
                boardtime = s.times.0.as_micros(),
                puzzletime = s.times.1.as_micros(),
            )
        } else {
            write!(f, "{}", self.puzzle)?;
            writeln!(f, "ID: {}", self.stats.seed)?;
            writeln!(f, "{}", self.stats.difficulty)?;
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
    seed: Option<String>,
    seed_length: usize,
    difficulty: GeneratorDifficulty,
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
                .take(self.seed_length)
                .map(char::from)
                .collect()
        });

        let difficulty = match &self.difficulty {
            GeneratorDifficulty::Given(d) => d.clone(),
            GeneratorDifficulty::Random => {
                let difficulty_name = Difficulty::get_all()
                    .choose(&mut thread_rng())
                    .expect("No difficulties while generating a random one");
                Difficulty::try_from(*difficulty_name)
                    .expect("Difficulty could not be built while generating a random one")
            }
        };

        let empty_positions = match &difficulty {
            Difficulty::Easy => 25,
            Difficulty::Normal => 35,
            Difficulty::Hard => 50,
            Difficulty::Insane => 64,
        };

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
                if removed >= empty_positions {
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
            difficulty,
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
    pub fn with_given_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = GeneratorDifficulty::Given(difficulty);
        self
    }

    /// Configure that each time [Generator::generate] is called, a random
    /// [Difficulty] is used.
    pub fn with_random_difficulty(mut self) -> Self {
        self.difficulty = GeneratorDifficulty::Random;
        self
    }

    /// Configures the generator difficulty directly to a given one or random.
    /// [Difficulty::Normal] by default.
    pub fn with_difficulty(mut self, difficulty: GeneratorDifficulty) -> Self {
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

    /// Sets the length of the auto-generated seed of the puzzle. 8 by default.
    pub fn with_seed_length(mut self, seed_length: usize) -> Self {
        self.seed_length = seed_length;
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
            difficulty: GeneratorDifficulty::Given(Difficulty::Normal),
            seed: None,
            seed_length: 8,
            count_solutions: false,
            max_count_solutions: 256,
            show_solution: false,
        }
    }
}

/// Difficulty that the [Generator] will use
#[derive(Clone, Debug)]
pub enum GeneratorDifficulty {
    /// Exact [Difficulty] for all the puzzles.
    Given(Difficulty),
    /// Random [Difficulty] each time the [Generator::generate] function is called.
    Random,
}

impl GeneratorDifficulty {
    /// Returns all the str representations of the difficulty levels
    pub fn get_all() -> &'static [&'static str; 5] {
        &["easy", "normal", "hard", "insane", "random"]
    }
}

impl TryFrom<&str> for GeneratorDifficulty {
    type Error = &'static str;
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "random" => Ok(Self::Random),
            _ => Difficulty::try_from(val).map(Self::Given),
        }
    }
}

impl fmt::Display for GeneratorDifficulty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            match self {
                GeneratorDifficulty::Random => write!(f, "random"),
                GeneratorDifficulty::Given(d) => write!(f, "{:#}", d),
            }
        } else {
            match self {
                GeneratorDifficulty::Random => write!(f, "Random"),
                GeneratorDifficulty::Given(d) => write!(f, "{}", d),
            }
        }
    }
}

/// Difficulty of the puzzles. Currently only changes the number
/// of empty positions.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Insane,
}

impl Difficulty {
    /// Returns all the str representations of the difficulty levels
    pub const fn get_all() -> &'static [&'static str; 4] {
        &["easy", "normal", "hard", "insane"]
    }

    /// Returns the str representation of the difficulty
    pub const fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
            Difficulty::Insane => "insane",
        }
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
        let name = self.as_str().to_string();

        let name = if !f.alternate() {
            let mut c = name.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        } else {
            name
        };

        write!(f, "{}", name)
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

    #[test]
    fn all_difficulties_strings() {
        for &d_str in Difficulty::get_all() {
            let d = Difficulty::try_from(d_str).unwrap();
            assert_eq!(format!("{:#}", d).as_str(), d_str);
        }
    }

    #[test]
    fn generator_difficulty_and_difficulty() {
        let g_ds = GeneratorDifficulty::get_all();
        let ds = Difficulty::get_all();

        assert_eq!(ds.len(), g_ds.len() - 1);

        for &d_str in ds {
            assert!(g_ds.iter().any(|&g_d| g_d == d_str));
        }
        assert!(g_ds.iter().any(|&g_d| g_d == "random"));
    }
}
