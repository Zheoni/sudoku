use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;

const N: usize = 3;
const N2: usize = N * N;
pub const SIZE: usize = N2 * N2;

#[inline]
const fn to_pos(row: usize, col: usize) -> usize {
	row * N2 + col
}

pub struct SudokuBoard([u16; SIZE]);

impl SudokuBoard {
	pub fn solve(&mut self) -> bool {
		// get the first empty postion
		let pos = self.get_empty_position();
		if pos.is_none() {
			return true;
		}
		let pos = pos.unwrap();

		for n in 0..=N2 as u16 {
			if self.is_valid(n, pos) {
				self.0[pos] = n;
				if self.solve() {
					return true;
				} else {
					self.0[pos] = 0;
				}
			}
		}

		false
	}

	fn get_empty_position(&self) -> Option<usize> {
		for (i, &n) in self.0.iter().enumerate() {
			if n == 0 {
				return Some(i);
			}
		}
		None
	}

	fn is_valid(&self, n: u16, pos: usize) -> bool {
		let (row, col) = (pos / N2, pos % N2);
		self.is_valid_row(n, row) && self.is_valid_col(n, col) && self.is_valid_group(n, row, col)
	}

	fn is_valid_row(&self, n: u16, row: usize) -> bool {
		for i in 0..N2 {
			if n == self.0[to_pos(row, i)] {
				return false;
			}
		}
		true
	}

	fn is_valid_col(&self, n: u16, col: usize) -> bool {
		for i in 0..N2 {
			if n == self.0[to_pos(i, col)] {
				return false;
			}
		}
		true
	}

	fn is_valid_group(&self, n: u16, row: usize, col: usize) -> bool {
		let group_row = row - row % N;
		let group_col = col - col % N;

		for i in 0..N {
			for j in 0..N {
				if n == self.0[to_pos(group_row + i, group_col + j)] {
					return false;
				}
			}
		}
		true
	}
}

impl fmt::Display for SudokuBoard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fn fmt_row(f: &mut fmt::Formatter<'_>, row: &[u16]) -> fmt::Result {
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

	/// Tries to converts a sudoku string into a sudoku board.
	///
	/// **May not work correctly for SIZE > 3**
	fn try_from(s: &str) -> Result<SudokuBoard, Self::Error> {
		if s.len() != SIZE {
			return Err("Invalid str len, must be SIZE");
		}

		s.chars()
			.map(|c| match c {
				'.' => Ok(0),
				c => c.to_digit(10).ok_or("Invalid character").map(|d| d as u16),
			})
			.collect::<Result<Vec<u16>, Self::Error>>()
			.and_then(|v| v.try_into())
	}
}

impl TryFrom<[u16; SIZE]> for SudokuBoard {
	type Error = &'static str;

	fn try_from(arr: [u16; SIZE]) -> Result<SudokuBoard, Self::Error> {
		if arr.iter().all(|&d| d <= N2 as u16) {
			Ok(SudokuBoard(arr))
		} else {
			Err("Values must be between 0 and sqrt(SIZE)")
		}
	}
}

impl TryFrom<Vec<u16>> for SudokuBoard {
	type Error = &'static str;
	fn try_from(vec: Vec<u16>) -> Result<SudokuBoard, Self::Error> {
		vec.as_slice().try_into()
	}
}

impl TryFrom<&[u16]> for SudokuBoard {
	type Error = &'static str;
	fn try_from(slice: &[u16]) -> Result<SudokuBoard, Self::Error> {
		<[u16; SIZE]>::try_from(slice)
			.map_err(|_| "Could not convert slice to array")
			.and_then(SudokuBoard::try_from)
	}
}
