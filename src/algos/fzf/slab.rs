use core::ops::{Index, IndexMut, Range};

use super::{FzfQuery, Score};

/// TODO: docs
#[derive(Clone, Default)]
pub(super) struct V2Slab {
    /// TODO: docs
    pub(super) bonus_vector: BonusVectorSlab,

    /// TODO: docs
    pub(super) candidate: CandidateSlab,

    /// TODO: docs
    pub(super) consecutive_matrix: ConsecutiveMatrixSlab,

    /// TODO: docs
    pub(super) matched_indices: MatchedIndicesSlab,

    /// TODO: docs
    pub(super) scoring_matrix: ScoringMatrixSlab,
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct CandidateSlab {
    chars: Vec<char>,
    char_indices: Vec<usize>,
}

impl Default for CandidateSlab {
    #[inline]
    fn default() -> Self {
        let chars = vec!['\0'; 16];
        let char_indices = vec![0; 16];
        Self { chars, char_indices }
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
            self.char_indices.resize(candidate.len(), 0);
        }

        let mut len = 0;

        for (offset, char) in candidate.char_indices() {
            self.chars[len] = char;
            self.char_indices[len] = offset;
            len += 1;
        }

        Candidate {
            chars: &self.chars[..len],
            char_offsets: &self.char_indices[..len],
            char_offset: 0,
        }
    }
}

/// TODO: docs
#[derive(Clone, Copy)]
pub(super) struct Candidate<'a> {
    chars: &'a [char],
    char_offsets: &'a [usize],
    char_offset: usize,
}

impl core::fmt::Debug for Candidate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.chars.iter().collect::<String>().fmt(f)
    }
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CandidateCharIdx(pub usize);

impl CandidateCharIdx {
    #[inline]
    pub fn into_usize(self) -> usize {
        self.0
    }
}

impl<'a> Candidate<'a> {
    /// TODO: docs
    #[inline]
    pub fn char(&self, idx: CandidateCharIdx) -> char {
        self.chars[idx.0 - self.char_offset]
    }

    /// TODO: docs
    #[inline]
    pub fn char_offset(&self, idx: CandidateCharIdx) -> usize {
        self.char_offsets[idx.0 - self.char_offset]
    }

    /// TODO: docs
    #[inline]
    pub fn char_idxs(&self) -> CandidateCharIdxs<'_> {
        CandidateCharIdxs {
            chars: self.chars,
            next_idx: CandidateCharIdx(self.char_offset),
        }
    }

    /// TODO: docs
    #[inline]
    pub fn char_len(&self) -> usize {
        self.chars.len()
    }

    /// TODO: docs
    #[inline]
    pub fn slice(self, range: Range<CandidateCharIdx>) -> Self {
        let range = range.start.0..range.end.0 + 1;
        let chars = &self.chars[range.clone()];
        let char_offsets = &self.char_offsets[range.clone()];
        let char_offset = self.char_offset + range.start;
        Self { chars, char_offsets, char_offset }
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
    vec: Vec<CandidateCharIdx>,
}

impl Default for MatchedIndicesSlab {
    #[inline]
    fn default() -> Self {
        Self { vec: vec![CandidateCharIdx(0); 16] }
    }
}

impl MatchedIndicesSlab {
    #[inline]
    /// TODO: docs
    pub fn alloc<'a>(&'a mut self, query: FzfQuery) -> MatchedIndices<'a> {
        let char_len = query.char_len();

        if char_len > self.vec.len() {
            self.vec.resize(char_len, CandidateCharIdx(0));
        }

        MatchedIndices::new(&mut self.vec[..char_len])
    }
}

/// TODO: docs
#[derive(Debug)]
pub(super) struct MatchedIndices<'a> {
    indices: &'a mut [CandidateCharIdx],

    /// The number of indices in [`Self::indices`] that have been pushed so far.
    ///
    /// The remaining indices are uninitialized (i.e. set to zero) and should
    /// not be read from.
    len: usize,
}

impl<'a> MatchedIndices<'a> {
    /// TODO: docs
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = CandidateCharIdx> + 'a {
        self.indices[..self.len].iter().copied()
    }

    /// TODO: docs
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == self.indices.len()
    }

    #[inline]
    pub fn new(indices: &'a mut [CandidateCharIdx]) -> Self {
        Self { indices, len: 0 }
    }

    /// TODO: docs
    #[inline]
    pub fn push(&mut self, idx: CandidateCharIdx) {
        self.indices[self.len] = idx;
        self.len += 1;
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
    pub fn alloc<'a>(&'a mut self, candidate: Candidate) -> BonusVector<'a> {
        let char_len = candidate.char_len();

        if char_len > self.vec.len() {
            self.vec.resize(char_len, 0);
        }

        BonusVector { indices: &mut self.vec[..char_len] }
    }
}

/// TODO: docs
pub(super) struct BonusVector<'a> {
    indices: &'a mut [Score],
}

impl core::fmt::Debug for BonusVector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.indices.fmt(f)
    }
}

impl Index<CandidateCharIdx> for BonusVector<'_> {
    type Output = Score;

    #[inline]
    fn index(&self, index: CandidateCharIdx) -> &Self::Output {
        &self.indices[index.0]
    }
}

impl IndexMut<CandidateCharIdx> for BonusVector<'_> {
    #[inline]
    fn index_mut(&mut self, index: CandidateCharIdx) -> &mut Self::Output {
        &mut self.indices[index.0]
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
#[derive(Default, Clone)]
pub(super) struct ConsecutiveMatrixSlab {
    slab: MatrixSlab<usize>,
}

impl ConsecutiveMatrixSlab {
    /// TODO: docs
    #[inline]
    pub fn alloc<'a>(
        &'a mut self,
        query: FzfQuery,
        candidate: Candidate,
    ) -> Matrix<'a, usize> {
        let height = query.char_len();
        let width = candidate.char_len();
        self.slab.alloc(width, height)
    }
}

/// TODO: docs
#[derive(Default, Clone)]
pub(super) struct ScoringMatrixSlab {
    slab: MatrixSlab<Score>,
}

impl ScoringMatrixSlab {
    /// TODO: docs
    #[inline]
    pub fn alloc(
        &mut self,
        query: FzfQuery,
        candidate: Candidate,
    ) -> Matrix<'_, Score> {
        let height = query.char_len();
        let width = candidate.char_len();
        self.slab.alloc(width, height)
    }
}

/// TODO: docs
#[derive(Default, Clone)]
pub(super) struct MatrixSlab<T: MatrixItem> {
    vec: Vec<T>,
}

impl<T: MatrixItem> MatrixSlab<T> {
    /// TODO: docs
    #[inline]
    fn alloc(&mut self, width: usize, height: usize) -> Matrix<'_, T> {
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
    pub fn right_n(&self, cell: MatrixCell, n: usize) -> MatrixCell {
        MatrixCell(cell.0 + n)
    }

    /// TODO: docs
    #[inline]
    pub fn row_of(&self, cell: MatrixCell) -> usize {
        cell.0 / self.width
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

    #[inline]
    pub fn up(&self, cell: MatrixCell) -> MatrixCell {
        MatrixCell(cell.0 - self.width)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MatrixCell(usize);

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
