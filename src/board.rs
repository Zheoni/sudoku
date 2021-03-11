use super::{N, N2, SIZE};
use crate::pos_util::*;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;

pub struct SudokuBoard([u16; SIZE]);

type Domains = [[bool; N2]; SIZE];

impl SudokuBoard {
	pub fn solve(&mut self) -> bool {
		let mut domains = self.calculate_domains();
		// dbg!(self.still_possible(&domains));
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
			if self.is_valid(n, pos) {
				// apply the value and update the domains
				self.0[pos] = n;
				temp_domains = *domains;
				self.update_domains(domains, pos);
				// if sudoku can still be solved
				if self.still_possible(domains) {
					// contue searching
					if self.backtracking(domains) {
						// solution found
						return true;
					}
				}
				// backtrack: restore the position and the domains
				self.0[pos] = 0;
				*domains = temp_domains;
			}
		}

		false
	}

	fn calculate_domains(&self) -> Domains {
		let mut domains = [[true; N2]; SIZE];

		// for each cell
		for (pos, &n) in self.0.iter().enumerate() {
			// if the cell is assigned
			if n != 0 {
				// set all of its possible values to false
				domains[pos].fill(false);
				// update the domains as if the value was just assigned
				self.update_domains(&mut domains, pos);
			}
		}
		domains
	}

	fn update_domains(&self, domains: &mut Domains, updated_pos: usize) {
		let new_val = self.0[updated_pos] as usize;
		assert!(new_val > 0);
		let new_val = new_val - 1;

		// in all conflicting indexes (row, col, group) mark the new value as false
		for p in adjacent_positions(updated_pos) {
			domains[p][new_val] = false;
		}
	}

	fn get_empty_position(&self, domains: &Domains, min_tie_to_solve: usize) -> Option<usize> {
		// Calculate the number of available values for each empty position
		let mut values: Vec<(u32, usize)> = domains
			.iter()
			.enumerate()
			.filter(|(pos, _domain)| self.0[*pos] == 0)
			.map(|(pos, domain)| {
				(
					domain.iter().fold(0, |acc, &x| acc + if x { 1 } else { 0 }),
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

	fn get_possible(&self, pos: usize, domains: &Domains, min_possible_ordered: usize) -> Vec<u16> {
		let possible: Vec<_> = domains[pos]
			.iter()
			.enumerate()
			.filter(|(_, &possible)| possible)
			.map(|(value, _)| value as u16 + 1)
			.collect();

		if possible.len() > min_possible_ordered {
			let mut values = domains
				.iter()
				.enumerate()
				.filter(|(pos, _)| self.0[*pos] == 0)
				.fold([0; N2], |mut acc, (_, domain)| {
					acc.iter_mut()
						.zip(domain)
						.for_each(|(accref, x)| *accref += if *x { 0 } else { 1 });
					acc
				})
				.iter()
				.enumerate()
				.map(|(i, x)| (*x, i as u16 + 1))
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
			.iter()
			.enumerate()
			.filter(|(pos, _)| self.0[*pos] == 0)
			.map(|(_, domain)| domain.iter().fold(0, |acc, &x| acc + if x { 1 } else { 0 }))
			.any(|sum| sum == 0)
	}

	fn is_valid(&self, n: u16, pos: usize) -> bool {
		for p in adjacent_positions(pos) {
			if n == self.0[p] {
				return false;
			}
		}
		true
	}

	fn is_valid_row(&self, n: u16, row: usize) -> bool {
		for p in row_positions(row) {
			if n == self.0[p] {
				return false;
			}
		}
		true
	}

	fn is_valid_col(&self, n: u16, col: usize) -> bool {
		for p in col_positions(col) {
			if n == self.0[p] {
				return false;
			}
		}
		true
	}

	fn is_valid_group(&self, n: u16, row: usize, col: usize) -> bool {
		for p in group_positions(row, col) {
			if n == self.0[p] {
				return false;
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
