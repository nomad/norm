use core::ops::{AddAssign, Index, IndexMut, SubAssign};

use super::{FzfQuery, Score};

/// TODO: docs
#[derive(Clone, Default)]
pub(super) struct V2Slab {
    /// TODO: docs
    pub(super) bonus_vector: BonusVectorSlab,

    /// TODO: docs
    pub(super) candidate: CandidateSlab,

    /// TODO: docs
    pub(super) consecutive_matrix: MatrixSlab<usize>,

    /// TODO: docs
    pub(super) matched_indices: MatchedIndicesSlab,

    /// TODO: docs
    pub(super) scoring_matrix: MatrixSlab<Score>,
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct CandidateSlab {
    chars: Vec<char>,
    char_offsets: Vec<usize>,
}

impl Default for CandidateSlab {
    #[inline]
    fn default() -> Self {
        let chars = vec!['\0'; 64];
        let char_indices = vec![0; 64];
        Self { chars, char_offsets: char_indices }
    }
}

impl CandidateSlab {
    /// TODO: docs
    #[inline]
    pub fn alloc<'a>(&'a mut self, candidate: &str) -> Candidate<'a> {
        // Here we compare the byte length of the candidate string with the
        // current char length of the slab. This is fine since the byte length
        // is always greater than or equal to the char length.
        //
        // The worst case scenario is that we allocate more space than we need
        // to, but that's fine since we'll reuse the space later.
        if candidate.len() > self.chars.len() {
            self.chars.resize(candidate.len(), '\0');
            self.char_offsets.resize(candidate.len(), 0);
        }

        let mut len = 0;

        for (offset, char) in candidate.char_indices() {
            self.chars[len] = char;
            self.char_offsets[len] = offset;
            len += 1;
        }

        Candidate {
            chars: &self.chars[..len],
            char_offsets: &self.char_offsets[..len],
            byte_len: candidate.len(),
        }
    }
}

/// TODO: docs
#[derive(Clone, Copy)]
pub(super) struct Candidate<'a> {
    chars: &'a [char],
    char_offsets: &'a [usize],
    byte_len: usize,
}

impl core::fmt::Debug for Candidate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.chars.iter().collect::<String>().fmt(f)
    }
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CandidateCharIdx(pub usize);

impl AddAssign<Self> for CandidateCharIdx {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<'a> Candidate<'a> {
    /// TODO: docs
    #[inline]
    pub fn nth_char_offset(&self, n: usize) -> usize {
        if n == self.char_offsets.len() {
            self.byte_len
        } else {
            self.char_offsets[n]
        }
    }
}

/// TODO: docs
pub(super) struct CandidateCharIdxs<'a> {
    chars: &'a [char],
    next_idx: CandidateCharIdx,
}

impl Iterator for CandidateCharIdxs<'_> {
    type Item = (CandidateCharIdx, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.chars.is_empty() {
            return None;
        }
        let char = self.chars[0];
        let idx = self.next_idx;
        self.chars = &self.chars[1..];
        self.next_idx.0 += 1;
        Some((idx, char))
    }
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct MatchedIndicesSlab {
    vec: Vec<MatchedIdx>,
}

impl Default for MatchedIndicesSlab {
    #[inline]
    fn default() -> Self {
        Self { vec: vec![MatchedIdx::default(); 16] }
    }
}

impl MatchedIndicesSlab {
    #[inline]
    /// TODO: docs
    pub fn alloc<'a>(&'a mut self, query: FzfQuery) -> &'a mut [MatchedIdx] {
        let char_len = query.char_len();

        if char_len > self.vec.len() {
            self.vec.resize(char_len, MatchedIdx::default());
        }

        &mut self.vec[..char_len]
    }
}

/// TODO: docs
#[derive(Copy, Clone, Debug, Default)]
pub(super) struct MatchedIdx {
    /// TODO: docs
    pub(super) byte_offset: usize,

    /// TODO: docs
    pub(super) char_offset: usize,
}

impl AddAssign<Self> for MatchedIdx {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.byte_offset += rhs.byte_offset;
        self.char_offset += rhs.char_offset;
    }
}

impl SubAssign<Self> for MatchedIdx {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.byte_offset -= rhs.byte_offset;
        self.char_offset -= rhs.char_offset;
    }
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct BonusVectorSlab {
    vec: Vec<Score>,
}

impl Default for BonusVectorSlab {
    #[inline]
    fn default() -> Self {
        Self { vec: vec![0; 16] }
    }
}

impl BonusVectorSlab {
    /// TODO: docs
    #[inline]
    pub fn alloc<'a>(&'a mut self, candidate: &str) -> BonusVector<'a> {
        let byte_len = candidate.len();

        if byte_len > self.vec.len() {
            self.vec.resize(byte_len, 0);
        }

        BonusVector { indices: &mut self.vec[..byte_len], len: 0 }
    }
}

/// TODO: docs
pub(super) struct BonusVector<'a> {
    indices: &'a mut [Score],
    len: usize,
}

impl core::fmt::Debug for BonusVector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.indices[..self.len].fmt(f)
    }
}

impl<'a> BonusVector<'a> {
    /// TODO: docs
    #[inline]
    pub fn into_slice(self) -> &'a [Score] {
        &self.indices[..self.len]
    }

    /// TODO: docs
    #[inline]
    pub fn push(&mut self, score: Score) {
        self.indices[self.len] = score;
        self.len += 1;
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
