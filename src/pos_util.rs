use super::{N, N2};

#[inline]
pub const fn to_pos(row: usize, col: usize) -> usize {
    row * N2 + col
}

#[inline]
pub const fn to_row_col(pos: usize) -> (usize, usize) {
    (pos / N2, pos % N2)
}

pub fn row_positions(row: usize) -> impl Iterator<Item = usize> + ExactSizeIterator {
    (0..N2).map(move |i| to_pos(row, i))
}

pub fn col_positions(col: usize) -> impl Iterator<Item = usize> + ExactSizeIterator {
    (0..N2).map(move |i| to_pos(i, col))
}

pub fn group_positions(row: usize, col: usize) -> impl Iterator<Item = usize> + ExactSizeIterator {
    let group_row = row - row % N;
    let group_col = col - col % N;

    (0..N2).map(move |group_i| {
        let (i, j) = (group_i / N, group_i % N);
        to_pos(group_row + i, group_col + j)
    })
}

pub fn adjacent_positions(pos: usize) -> impl Iterator<Item = usize> + ExactSizeIterator {
    let (row, col) = to_row_col(pos);
    AdjacentPositionsIterator::new(row, col)
}

struct AdjacentPositionsIterator {
    i: usize,
    row: usize,
    col: usize,
    group_row: usize,
    group_col: usize,
    phase: Phase,
    count: usize,
}

enum Phase {
    Row,
    Column,
    Group,
    End,
}

impl AdjacentPositionsIterator {
    fn new(row: usize, col: usize) -> Self {
        Self {
            i: 0,
            row,
            col,
            group_row: row - row % N,
            group_col: col - col % N,
            phase: Phase::Row,
            count: 0,
        }
    }
}

impl Iterator for AdjacentPositionsIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= N2 {
            self.phase = match self.phase {
                Phase::Row => Phase::Column,
                Phase::Column => Phase::Group,
                Phase::Group => Phase::End,
                Phase::End => Phase::End,
            };
            self.i = 0;
        }
        match self.phase {
            Phase::Row => {
                // skip self col
                if self.i == self.col {
                    self.i += 1;
                    self.next()
                } else {
                    let r = to_pos(self.row, self.i);
                    self.i += 1;
                    self.count += 1;
                    Some(r)
                }
            }
            Phase::Column => {
                // skip self row
                if self.i == self.row {
                    self.i += 1;
                    self.next()
                } else {
                    let r = to_pos(self.i, self.col);
                    self.i += 1;
                    self.count += 1;
                    Some(r)
                }
            }
            Phase::Group => {
                let (g_row, g_col) = (self.i / N, self.i % N);
                let row = self.group_row + g_row;
                let col = self.group_col + g_col;
                // skip self row and col
                if row == self.row || col == self.col {
                    self.i += 1;
                    self.next()
                } else {
                    let r = to_pos(row, col);
                    self.i += 1;
                    self.count += 1;
                    Some(r)
                }
            }
            Phase::End => None,
        }
    }
}

impl ExactSizeIterator for AdjacentPositionsIterator {
    fn len(&self) -> usize {
        const TOTAL: usize = (N2 - 1) * 2 + (N2 - (N * 2) + 1);
        TOTAL - self.count
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
            let expected_length = 20;
            #[rustfmt::skip]
            let expected = vec![
                9,     11, 12, 13, 14, 15, 16, 17,
                1,     19, 28, 37, 46, 55, 64, 73,
                0,     2,              18,     20
            ];

            let mut it = adjacent_positions(10);
            for (i, val) in expected.into_iter().enumerate() {
                assert_eq!(it.len(), expected_length - i);
                assert_eq!(it.next(), Some(val));
            }

            assert_eq!(it.len(), 0);
            assert_eq!(it.next(), None);
            assert_eq!(it.len(), 0);
        }
    }

    #[test]
    fn test_adjacent_edge() {
        if N == 3 {
            let expected_length = 20;
            #[rustfmt::skip]
            let expected = vec![
                36, 37, 38, 39, 40, 41, 42, 43,
                 8, 17, 26, 35,     53, 62, 71, 80, 
                33, 34,                 51, 52
            ];

            let mut it = adjacent_positions(44);
            for (i, val) in expected.into_iter().enumerate() {
                assert_eq!(it.len(), expected_length - i);
                assert_eq!(it.next(), Some(val));
            }

            assert_eq!(it.len(), 0);
            assert_eq!(it.next(), None);
            assert_eq!(it.len(), 0);
        }
    }
}
