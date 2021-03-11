use super::{N, N2};

#[inline]
pub const fn to_pos(row: usize, col: usize) -> usize {
	row * N2 + col
}

#[inline]
pub const fn to_row_col(pos: usize) -> (usize, usize) {
	(pos / N2, pos % N2)
}

pub trait PosIteratorKind {}

pub struct RowKind(usize);
impl PosIteratorKind for RowKind {}

pub struct ColKind(usize);
impl PosIteratorKind for ColKind {}

pub struct GroupKind {
	group_row: usize,
	group_col: usize,
}
impl PosIteratorKind for GroupKind {}

pub struct AdjacentKind {
	row: usize,
	col: usize,
	group_row: usize,
	group_col: usize,
	idx: usize,
	count: usize,
}
impl PosIteratorKind for AdjacentKind {}

pub struct PosIterator<K: PosIteratorKind> {
	kind: K,
	i: usize,
}

pub fn row_positions(row: usize) -> PosIterator<RowKind> {
	PosIterator {
		kind: RowKind(row),
		i: 0,
	}
}

pub fn col_positions(col: usize) -> PosIterator<ColKind> {
	PosIterator {
		kind: ColKind(col),
		i: 0,
	}
}

pub fn group_positions(row: usize, col: usize) -> PosIterator<GroupKind> {
	let group_row = row - row % N;
	let group_col = col - col % N;

	PosIterator {
		kind: GroupKind {
			group_row,
			group_col,
		},
		i: 0,
	}
}

pub fn adjacent_positions(pos: usize) -> PosIterator<AdjacentKind> {
	let (row, col) = to_row_col(pos);
	let group_row = row - row % N;
	let group_col = col - col % N;

	PosIterator {
		kind: AdjacentKind {
			row,
			col,
			group_row,
			group_col,
			idx: pos,
			count: 0,
		},
		i: 0,
	}
}

impl Iterator for PosIterator<RowKind> {
	type Item = usize;
	fn next(&mut self) -> Option<Self::Item> {
		if self.i < N2 {
			let current_index = to_pos(self.kind.0, self.i);
			self.i += 1;
			Some(current_index)
		} else {
			None
		}
	}
}

impl ExactSizeIterator for PosIterator<RowKind> {
	fn len(&self) -> usize {
		N2 - self.i
	}
}

impl Iterator for PosIterator<ColKind> {
	type Item = usize;
	fn next(&mut self) -> Option<Self::Item> {
		if self.i < N2 {
			let current_index = to_pos(self.i, self.kind.0);
			self.i += 1;
			Some(current_index)
		} else {
			None
		}
	}
}

impl ExactSizeIterator for PosIterator<ColKind> {
	fn len(&self) -> usize {
		N2 - self.i
	}
}

impl Iterator for PosIterator<GroupKind> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < N2 {
			let (i, j) = (self.i / N, self.i % N);
			let current_index = to_pos(self.kind.group_row + i, self.kind.group_col + j);
			self.i += 1;
			Some(current_index)
		} else {
			None
		}
	}
}

impl ExactSizeIterator for PosIterator<GroupKind> {
	fn len(&self) -> usize {
		N2 - self.i
	}
}

impl Iterator for PosIterator<AdjacentKind> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		let mut r = None;
		if self.i < N2 {
			if self.kind.col != self.i {
				r = Some(to_pos(self.kind.row, self.i));
				self.i += 1;
				self.kind.count += 1;
			} else {
				self.i += 1;

				r = self.next();
			}
		} else if self.i < 2 * N2 {
			let i = self.i - N2;
			if self.kind.row != i {
				r = Some(to_pos(i, self.kind.col));
				self.i += 1;
				self.kind.count += 1;
			} else {
				self.i += 1;

				r = self.next();
			}
		} else if self.i < 3 * N2 {
			let i = self.i - 2 * N2;
			let (i, j) = (i / N, i % N);

			let (crow, ccol) = (self.kind.group_row + i, self.kind.group_col + j);

			let current_index = to_pos(crow, ccol);
			self.i += 1;

			if self.kind.row == crow || self.kind.col == ccol || current_index == self.kind.idx {
				r = self.next();
			} else {
				r = Some(current_index);
				self.kind.count += 1;
			}
		}

		r
	}
}

impl ExactSizeIterator for PosIterator<AdjacentKind> {
	fn len(&self) -> usize {
		(N2 - 1 + N * (N - 1) * 2) - self.kind.count
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_row() {
		let mut it = row_positions(2);
		for i in 2 * N2..3 * N2 {
			assert_eq!(it.next(), Some(i));
		}
		assert_eq!(it.next(), None)
	}

	#[test]
	fn test_col() {
		let mut it = col_positions(2);
		for i in 0..N2 {
			assert_eq!(it.next(), Some(N2 * i + 2))
		}
		assert_eq!(it.next(), None)
	}

	#[test]
	fn test_group() {
		let mut it = group_positions(0, 0);
		for i in 0..N {
			for j in 0..N {
				assert_eq!(it.next(), Some(i * N2 + j))
			}
		}
		assert_eq!(it.next(), None)
	}

	#[test]
	fn test_adjacent() {
		if N == 3 {
			let mut it = adjacent_positions(10);
			assert_eq!(it.len(), 20);
			assert_eq!(it.next(), Some(9));
			assert_eq!(it.next(), Some(11));
			assert_eq!(it.next(), Some(12));
			assert_eq!(it.next(), Some(13));
			assert_eq!(it.next(), Some(14));
			assert_eq!(it.next(), Some(15));
			assert_eq!(it.next(), Some(16));
			assert_eq!(it.next(), Some(17));
			assert_eq!(it.next(), Some(1));
			assert_eq!(it.next(), Some(19));
			assert_eq!(it.next(), Some(28));
			assert_eq!(it.next(), Some(37));
			assert_eq!(it.next(), Some(46));
			assert_eq!(it.next(), Some(55));
			assert_eq!(it.next(), Some(64));
			assert_eq!(it.next(), Some(73));
			assert_eq!(it.next(), Some(0));
			assert_eq!(it.next(), Some(2));
			assert_eq!(it.next(), Some(18));
			assert_eq!(it.next(), Some(20));
			assert_eq!(it.next(), None);
			assert_eq!(it.len(), 0);
		}
	}
}
