use core::ops::{Index, IndexMut};

use super::Score;

/// TODO: docs
#[derive(Clone, Default)]
pub(super) struct V2Slab {
    /// TODO: docs
    pub(super) bonus: BonusSlab,

    /// TODO: docs
    pub(super) consecutive_matrix: MatrixSlab<usize>,

    /// TODO: docs
    pub(super) matched_indices: MatchedIndicesSlab,

    /// TODO: docs
    pub(super) scoring_matrix: MatrixSlab<Score>,
}

// #[repr(align(8))]
/// TODO: docs
#[derive(Clone, Default)]
pub(super) struct Bonus {
    value: u8,
    is_set: bool,
}

impl Bonus {
    #[inline(always)]
    pub fn is_set(&self) -> bool {
        self.is_set
    }

    #[inline(always)]
    pub fn set(&mut self, value: Score) {
        self.value = value as _;
        self.is_set = true;
    }

    #[inline(always)]
    pub fn value(&self) -> Score {
        self.value as _
    }
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct BonusSlab {
    vec: Vec<Bonus>,
}

impl Default for BonusSlab {
    #[inline(always)]
    fn default() -> Self {
        Self { vec: vec![Bonus::default(); 128] }
    }
}

impl BonusSlab {
    /// TODO: docs
    #[inline]
    pub fn alloc<'a>(&'a mut self, len: usize) -> &'a mut [Bonus] {
        if len > self.vec.len() {
            self.vec.resize(len, Bonus::default());
        }

        let slice = &mut self.vec[..len];

        for bonus in slice.iter_mut() {
            bonus.is_set = false;
        }

        slice
    }
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct CandidateSlab {
    chars: Vec<char>,
}

impl Default for CandidateSlab {
    #[inline(always)]
    fn default() -> Self {
        Self { chars: vec![char::default(); 128] }
    }
}

impl CandidateSlab {
    #[inline(always)]
    pub fn alloc<'a>(&'a mut self, text: &str) -> &'a [char] {
        if text.len() > self.chars.len() {
            self.chars.resize(text.len(), char::default());
        }

        let mut char_len = 0;

        for ch in text.chars() {
            self.chars[char_len] = ch;
            char_len += 1;
        }

        &self.chars[..char_len]
    }
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct MatchedIndicesSlab {
    vec: Vec<usize>,
}

impl Default for MatchedIndicesSlab {
    #[inline]
    fn default() -> Self {
        Self { vec: vec![0; 128] }
    }
}

impl MatchedIndicesSlab {
    #[inline]
    /// TODO: docs
    pub fn alloc(&mut self, len: usize) -> &mut [usize] {
        if len > self.vec.len() {
            self.vec.resize(len, 0);
        }

        &mut self.vec[..len]
    }
}

pub(super) trait MatrixItem: Copy + Ord + core::fmt::Display {
    /// TODO: docs
    fn fill() -> Self;

    /// TODO: docs
    fn printed_width(&self) -> usize;
}

impl MatrixItem for Score {
    #[inline]
    fn fill() -> Self {
        0
    }

    fn printed_width(&self) -> usize {
        if *self == 0 {
            1
        } else {
            (self.ilog10() + 1) as usize
        }
    }
}

impl MatrixItem for usize {
    #[inline]
    fn fill() -> Self {
        0
    }

    fn printed_width(&self) -> usize {
        if *self == 0 {
            1
        } else {
            (self.ilog10() + 1) as usize
        }
    }
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct MatrixSlab<T: MatrixItem> {
    vec: Vec<T>,
}

impl<T: Default + MatrixItem> Default for MatrixSlab<T> {
    #[inline]
    fn default() -> MatrixSlab<T> {
        // We allocate a 256 cell matrix slab by default to minimize the
        // need to re-allocate for long `query * candidate` pairs.
        Self { vec: vec![T::default(); 256] }
    }
}

impl<T: MatrixItem> MatrixSlab<T> {
    /// TODO: docs
    #[inline]
    pub fn alloc(&mut self, width: usize, height: usize) -> Matrix<'_, T> {
        debug_assert!(height * width > 0);

        if height * width > self.vec.len() {
            self.vec.resize(height * width, T::fill());
        }

        let slice = &mut self.vec[..height * width];

        slice.fill(T::fill());

        Matrix { slice, height, width }
    }
}

/// TODO: docs
pub(super) struct Matrix<'a, T: MatrixItem> {
    /// TODO: docs
    ///
    /// <width><width>...<width>
    /// \---- height times ----/
    slice: &'a mut [T],
    height: usize,
    width: usize,
}

/// Prints the matrix like this:
///
/// ```text
///   ┌                         ┐
///   │0  16 16 13 12 11 10 9  8│
///   │0  0  0  0  0  0  0  0  0│
///   │0  0  0  0  0  0  0  0  0│
///   └                         ┘
/// ```
impl<T: MatrixItem> core::fmt::Debug for Matrix<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;

        // The matrix should never be empty, but just in case.
        if self.slice.is_empty() {
            return f.write_str("[ ]");
        }

        // The character width of the biggest score in the whole matrix.
        let max_score_width = {
            let max_score = self.slice.iter().copied().max().unwrap();
            max_score.printed_width()
        };

        // The character width of the biggest score in the last column.
        let last_col_max_score_width = {
            // The cell in the last column of the first row.
            let first_row_last_col =
                self.cols(self.top_left()).last().unwrap();

            let last_col_max_score = self
                .rows(first_row_last_col)
                .map(|cell| self[cell])
                .max()
                .unwrap();

            last_col_max_score.printed_width()
        };

        let printed_matrix_inner_width = (self.width - 1)
            * (max_score_width + 1)
            + last_col_max_score_width;

        let opening_char: char;

        let closing_char: char;

        if self.height == 1 {
            opening_char = '[';
            closing_char = ']';
        } else {
            f.write_char('┌')?;
            f.write_str(&" ".repeat(printed_matrix_inner_width))?;
            f.write_char('┐')?;
            f.write_char('\n')?;
            opening_char = '│';
            closing_char = '│';
        }

        for cell in self.rows(self.top_left()) {
            f.write_char(opening_char)?;

            for cell in self.cols(cell) {
                let item = self[cell];

                write!(f, "{item}")?;

                let num_spaces = if self.is_in_last_col(cell) {
                    last_col_max_score_width - item.printed_width()
                } else {
                    max_score_width - item.printed_width() + 1
                };

                f.write_str(&" ".repeat(num_spaces))?;
            }

            f.write_char(closing_char)?;

            f.write_char('\n')?;
        }

        if self.height > 1 {
            f.write_char('└')?;
            f.write_str(&" ".repeat(printed_matrix_inner_width))?;
            f.write_char('┘')?;
        }

        Ok(())
    }
}

impl<'a, T: MatrixItem> Matrix<'a, T> {
    /// TODO: docs
    #[inline]
    pub fn col_of(&self, cell: MatrixCell) -> usize {
        cell.0 % self.width
    }

    /// TODO: docs
    #[inline]
    pub fn cols(&self, starting_from: MatrixCell) -> Cols {
        Cols {
            next: starting_from,
            remaining: self.width - self.col_of(starting_from),
        }
    }

    /// TODO: docs
    #[inline]
    pub fn down(&self, cell: MatrixCell) -> MatrixCell {
        MatrixCell(cell.0 + self.width)
    }

    /// TODO: docs
    #[inline(always)]
    pub fn height(&self) -> usize {
        self.height
    }

    /// TODO: docs
    #[inline]
    pub fn is_in_first_col(&self, cell: MatrixCell) -> bool {
        self.col_of(cell) == 0
    }

    /// TODO: docs
    #[inline]
    pub fn is_in_first_row(&self, cell: MatrixCell) -> bool {
        self.row_of(cell) == 0
    }

    /// TODO: docs
    #[inline]
    pub fn is_in_last_col(&self, cell: MatrixCell) -> bool {
        self.col_of(cell) == self.width - 1
    }

    /// TODO: docs
    #[inline]
    pub fn is_in_last_row(&self, cell: MatrixCell) -> bool {
        self.row_of(cell) == self.height - 1
    }

    /// TODO: docs
    #[inline]
    pub fn left(&self, cell: MatrixCell) -> MatrixCell {
        MatrixCell(cell.0 - 1)
    }

    /// TODO: docs
    #[inline]
    pub fn right(&self, cell: MatrixCell) -> MatrixCell {
        MatrixCell(cell.0 + 1)
    }

    /// TODO: docs
    #[inline]
    pub fn row_of(&self, cell: MatrixCell) -> usize {
        cell.0 / self.width
    }

    /// TODO: docs
    #[inline]
    pub fn row_mut(&mut self, row: usize) -> &mut [T] {
        let start = row * self.width;
        let end = start + self.width;
        &mut self.slice[start..end]
    }

    #[inline]
    pub fn rows(&self, starting_from: MatrixCell) -> Rows {
        Rows {
            next: starting_from,
            matrix_width: self.width,
            remaining: self.height - self.row_of(starting_from),
        }
    }

    /// TODO: docs
    #[inline]
    pub fn top_left(&self) -> MatrixCell {
        MatrixCell(0)
    }

    /// TODO: docs
    #[inline]
    pub fn two_rows_mut(
        &mut self,
        row_idx_a: usize,
        row_idx_b: usize,
    ) -> (&mut Row<T>, &mut Row<T>) {
        debug_assert!(row_idx_a < row_idx_b);

        let start_b = row_idx_b * self.width;

        let (part_a, part_b) = self.slice.split_at_mut(start_b);

        let start_a = row_idx_a * self.width;

        (&mut part_a[start_a..start_a + self.width], &mut part_b[..self.width])
    }

    #[inline]
    pub fn up(&self, cell: MatrixCell) -> MatrixCell {
        MatrixCell(cell.0 - self.width)
    }

    /// TODO: docs
    #[inline(always)]
    pub fn width(&self) -> usize {
        self.width
    }
}

pub(super) type Row<T> = [T];

#[derive(Debug, Clone, Copy)]
pub(super) struct MatrixCell(pub(super) usize);

impl<T: MatrixItem> Index<MatrixCell> for Matrix<'_, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: MatrixCell) -> &Self::Output {
        &self.slice[index.0]
    }
}

impl<T: MatrixItem> IndexMut<MatrixCell> for Matrix<'_, T> {
    #[inline]
    fn index_mut(&mut self, index: MatrixCell) -> &mut Self::Output {
        &mut self.slice[index.0]
    }
}

/// TODO: docs
pub(super) struct Cols {
    next: MatrixCell,
    remaining: usize,
}

impl Iterator for Cols {
    type Item = MatrixCell;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let this = self.next;
        self.next.0 += 1;
        self.remaining -= 1;
        Some(this)
    }
}

/// TODO: docs
pub(super) struct Rows {
    next: MatrixCell,
    matrix_width: usize,
    remaining: usize,
}

impl Iterator for Rows {
    type Item = MatrixCell;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let this = self.next;
        self.next.0 += self.matrix_width;
        self.remaining -= 1;
        Some(this)
    }
}
