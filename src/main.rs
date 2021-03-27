use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgGroup, ArgMatches, SubCommand};
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, stdout, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use sudoku::prelude::*;

fn integer_validator(val: String) -> Result<(), String> {
    if val.chars().all(|c| c.is_digit(10)) {
        Ok(())
    } else {
        Err(format!("\"{}\" is not valid number", val))
    }
}

enum OutputFormat {
    Pretty,
    Line,
    Csv,
}

impl OutputFormat {
    const fn get_all() -> &'static [&'static str] {
        &["pretty", "line", "csv"]
    }
}

impl TryFrom<&str> for OutputFormat {
    type Error = &'static str;
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "pretty" => Ok(Self::Pretty),
            "csv" => Ok(Self::Csv),
            "line" => Ok(Self::Line),
            _ => Err("Unknown format"),
        }
    }
}

#[derive(Debug)]
enum Error {
    IOError(io::Error),
    ErrorMessage(&'static str),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<&'static str> for Error {
    fn from(err: &'static str) -> Self {
        Error::ErrorMessage(err)
    }
}

fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .about("Sudoku solver and generator")
        .arg(
            Arg::with_name("output")
                .help("Output to file")
                .short("o")
                .long("output")
                .takes_value(true)
                .value_name("FILE")
                .global(true)
        )
        .arg(
            Arg::with_name("format")
                .help("Format to output sudokus")
                .short("F")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .default_value("pretty")
                .possible_values(OutputFormat::get_all())
                .global(true)
        )
        .arg(
            Arg::with_name("count_solutions")
                .help("Count the number of solutions up to a limit. Limit can be set with --limit")
                .long("count")
                .required(false)
                .global(true)
        )
        .arg(
            Arg::with_name("multiple_limit")
                .help("Sets the limit when working with multiple sudoku solutions")
                .long("limit")
                .takes_value(true)
                .value_name("MAX")
                .default_value("255")
                .validator(integer_validator)
                .global(true)
        )
        .subcommand(
            SubCommand::with_name("solve")
                .alias("s")
                .about("Solve sudokus")
                .arg(
                    Arg::with_name("all")
                        .help("Gets multiple solutions up to a limit. Limit can be set with --limit")
                        .long("all")
                )
                .group(
                    ArgGroup::with_name("multiple_solutions")
                        .args(&["count_solutions", "all"])
                )
                .arg(
                    Arg::with_name("sudoku")
                        .help("Sudoku string or string separared by a newline")
                        .empty_values(false)
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("file")
                        .help("Path to input file, 1 line per sudoku")
                        .short("f")
                        .long("file")
                        .value_name("FILE")
                        .takes_value(true)
                        .empty_values(false)
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("from_seed")
                        .help("Solves the sudoku puzzle generated from a seed")
                        .short("s")
                        .long("seed")
                        .takes_value(true)
                        .value_name("SEED")
                        .conflicts_with_all(&["all", "count_solutions"])
                        .multiple(true)
                )
                .group(
                    ArgGroup::with_name("input")
                        .args(&["sudoku", "file", "from_seed"])
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("generate")
                .aliases(&["g", "gen"])
                .about("Generate sudokus")
                .arg(
                    Arg::with_name("amount")
                        .help("Amount of sudokus to generate")
                        .short("n")
                        .long("amount")
                        .takes_value(true)
                        .value_name("AMOUNT")
                        .default_value("1")
                        .validator(integer_validator)
                )
                .arg(
                    Arg::with_name("difficulty")
                        .help("Difficulty of the puzzles")
                        .short("d")
                        .long("difficulty")
                        .takes_value(true)
                        .default_value("normal")
                        .possible_values(GeneratorDifficulty::get_all())
                )
                .arg(
                    Arg::with_name("show_solution")
                        .help("Show the solution of the puzzle")
                        .long("show-solution")
                )
                .arg(
                    Arg::with_name("allow_multiple")
                        .help("Allow multiple solutions")
                        .short("m")
                        .long("allow-multiple")
                )
                .arg(
                    Arg::with_name("from_seed")
                        .help("Generate a puzzle from a seed. (The dificulty and uniqueness of solution must match to get the same puzzle)")
                        .next_line_help(true)
                        .short("s")
                        .long("seed")
                        .takes_value(true)
                        .value_name("SEED")
                )
        )
        .get_matches();

    let mut output: BufWriter<Box<dyn Write>> = if let Some(filename) = matches.value_of("output") {
        let path = Path::new(filename);
        let file = File::create(path)?;

        BufWriter::new(Box::from(file))
    } else {
        BufWriter::new(Box::from(stdout()))
    };

    let format: OutputFormat = matches
        .value_of("format")
        .expect("No format value, not even the default")
        .try_into()
        .expect("Unknown format");

    let multiple_limit = matches
        .value_of("multiple_limit")
        .expect("No value in multiple limit, not even the default.")
        .parse::<usize>()
        .expect("Invalid multiple limit value, cannot parse. However it did pass the validator.");

    match matches.subcommand() {
        ("solve", Some(sub_m)) => handle_solve(sub_m, &mut output, format, multiple_limit)?,
        ("generate", Some(sub_m)) => handle_generate(sub_m, &mut output, format, multiple_limit)?,
        _ => {
            let puzzle = SudokuPuzzle::prepare()
                .count_solutions(matches.is_present("count_solutions"))
                .max_count_solutions(multiple_limit)
                .generate();
            match format {
                OutputFormat::Pretty => writeln!(&mut output, "{}", puzzle)?,
                OutputFormat::Csv => {
                    writeln!(&mut output, "{}\n{:#}", SudokuPuzzle::csv_head(), puzzle)?
                }
                OutputFormat::Line => writeln!(&mut output, "{:#}", puzzle.puzzle)?,
            }
        }
    }

    output.flush()?;

    Ok(())
}

fn handle_solve(
    matches: &ArgMatches,
    output: &mut BufWriter<Box<dyn Write>>,
    format: OutputFormat,
    multiple_limit: usize,
) -> Result<(), Error> {
    use OutputFormat::*;
    let from_seeds = matches.is_present("from_seed");
    let count_solutions = matches.is_present("count_solutions");
    let all_solutions = matches.is_present("all");
    eprintln!("Parsing inputs...");
    let inputs: Vec<String> = if matches.is_present("sudoku") {
        matches
            .values_of("sudoku")
            .unwrap()
            .map(String::from)
            .collect()
    } else if matches.is_present("file") {
        matches
            .values_of_os("file")
            .unwrap()
            .map(Path::new)
            .map(|path| {
                if !path.is_file() {
                    return Err(Error::ErrorMessage("Input path is not a file"));
                }
                let file = File::open(path)?;
                let buffered = BufReader::new(file);

                let mut parsed_lines = Vec::new();

                for line in buffered.lines() {
                    let line = line?;
                    let line = line.trim();
                    if !line.is_empty() {
                        parsed_lines.push(line.to_string())
                    }
                }

                Ok(parsed_lines)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect()
    } else if matches.is_present("from_seed") {
        matches
            .values_of("from_seed")
            .unwrap()
            .map(String::from)
            .collect()
    } else {
        panic!("No input for solve");
    };

    eprintln!("Start solving {} sudoku(s)", inputs.len());

    if matches!(format, Csv) {
        writeln!(output, "input,result")?;
    }

    for input in &inputs {
        let mut board = if from_seeds {
            SudokuBoard::generate_from_seed(&input)
        } else {
            SudokuBoard::try_from(input.as_str())?
        };

        #[allow(clippy::collapsible_if)]
        if all_solutions {
            let solutions = board.solve_all(multiple_limit);
            if solutions.is_empty() {
                match format {
                    Pretty => writeln!(output, "{}:\n\tNo solution", input)?,
                    Line | Csv => writeln!(output, "{},no_solution", input)?,
                }
            } else {
                for sol in solutions {
                    match format {
                        Pretty => writeln!(output, "{}:\n{}", input, sol)?,
                        Line | Csv => writeln!(output, "{},{:#}", input, sol)?,
                    }
                }
            }
        } else if count_solutions {
            let count = board.count_solutions(multiple_limit);
            match format {
                Pretty => writeln!(output, "{}:\n\t{} solutions", input, count)?,
                Line | Csv => writeln!(output, "{},{}", input, count)?,
            }
        } else {
            if board.solve() {
                match format {
                    Pretty => writeln!(output, "{}:\n{}", input, board)?,
                    Line | Csv => writeln!(output, "{},{:#}", input, board)?,
                }
            } else {
                match format {
                    Pretty => writeln!(output, "{}:\n\tNo solution", input)?,
                    Line | Csv => writeln!(output, "{},no_solution", input)?,
                }
            }
        }
    }

    Ok(())
}

fn handle_generate(
    matches: &ArgMatches,
    output: &mut BufWriter<Box<dyn Write>>,
    format: OutputFormat,
    multiple_limit: usize,
) -> Result<(), Error> {
    let amount: usize = matches
        .value_of("amount")
        .expect("No amount of sudokus to generate, not even default.")
        .parse()
        .expect("Invalid amount, however it pass the validator");

    let mut builder = SudokuPuzzle::prepare()
        .with_difficulty(
            matches
                .value_of("difficulty")
                .expect("No difficulty, not even default.")
                .try_into()
                .expect("Invalid difficulty, however it is included in possible values."),
        )
        .unique_solution(!matches.is_present("allow_multiple"))
        .show_solution(matches.is_present("show_solution"))
        .count_solutions(matches.is_present("count_solutions"))
        .max_count_solutions(multiple_limit);
    if let Some(seed) = matches.value_of("from_seed") {
        builder = builder.with_seed(seed);
    }

    eprintln!("Generating puzzles...");

    if matches!(format, OutputFormat::Csv) {
        writeln!(output, "{}", SudokuPuzzle::csv_head())?;
    }

    for _ in 0..amount {
        let puzzle = builder.generate();
        match format {
            OutputFormat::Pretty => writeln!(output, "{}", puzzle)?,
            OutputFormat::Csv => writeln!(output, "{:#}", puzzle)?,
            OutputFormat::Line => {
                write!(output, "{:#}", puzzle.puzzle)?;
                if let Some(solution) = puzzle.solution {
                    write!(output, ",{:#}", solution)?;
                }
                writeln!(output)?;
            }
        }
    }

    Ok(())
}
