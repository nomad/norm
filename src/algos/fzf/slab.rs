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
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars.iter().copied()
    }

    /// TODO: docs
    #[inline]
    pub fn char_idxs(
        &self,
    ) -> impl Iterator<Item = (CandidateCharIdx, char)> + '_ {
        self.chars.iter().enumerate().map(|(idx, &char)| {
            (CandidateCharIdx(idx + self.char_offset), char)
        })
    }

    /// TODO: docs
    #[inline]
    pub fn char_len(&self) -> usize {
        self.chars.len()
    }

    /// TODO: docs
    #[inline]
    pub fn char_offset(&self, idx: CandidateCharIdx) -> usize {
        self.char_offsets[idx.0 - self.char_offset]
    }

    /// TODO: docs
    #[inline]
    pub fn char_offsets(&self) -> impl Iterator<Item = (usize, char)> + '_ {
        self.char_offsets
            .iter()
            .zip(self.chars)
            .map(|(&offset, &char)| (offset, char))
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
pub(super) struct MatchedIndices<'a> {
    indices: &'a mut [CandidateCharIdx],
    len: usize,
}

impl<'a> MatchedIndices<'a> {
    /// TODO: docs
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = CandidateCharIdx> + 'a {
        self.indices[..self.len].into_iter().copied()
    }

    /// TODO: docs
    #[inline]
    pub fn last(&self) -> CandidateCharIdx {
        self.indices[self.len - 1]
    }

    /// TODO: docs
    #[inline]
    pub fn len(&self) -> usize {
        self.len
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

impl core::fmt::Debug for MatchedIndices<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.indices[..self.len].fmt(f)
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

trait MatrixItem: Copy + Ord + core::fmt::Display {
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
    pub fn alloc<'a>(
        &'a mut self,
        query: FzfQuery,
        candidate: Candidate,
    ) -> Matrix<'a, Score> {
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
    fn alloc<'a>(&'a mut self, width: usize, height: usize) -> Matrix<'a, T> {
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
    #[inline]
    pub fn cols(&self, starting_from: MatrixCell) -> Cols {
        Cols::new(starting_from, self.width)
    }

    #[inline]
    pub fn col_of(&self, cell: MatrixCell) -> usize {
        cell.0 % self.width
    }

    #[inline]
    pub fn down(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.down(self.width, self.height)
    }

    /// TODO: docs
    #[inline]
    pub fn is_first_row(&self, cell: MatrixCell) -> bool {
        self.up(cell).is_none()
    }

    /// TODO: docs
    #[inline]
    pub fn is_in_last_col(&self, cell: MatrixCell) -> bool {
        self.right(cell).is_none()
    }

    /// TODO: docs
    #[inline]
    pub fn is_last_row(&self, cell: MatrixCell) -> bool {
        self.down(cell).is_none()
    }

    #[inline]
    pub fn left(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.left(self.width)
    }

    #[inline]
    pub fn right(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.right(self.width)
    }

    #[inline]
    pub fn right_n(&self, cell: MatrixCell, n: usize) -> Option<MatrixCell> {
        if n == 0 {
            Some(cell)
        } else {
            (MatrixCell(cell.0 + n - 1)).right(self.width)
        }
    }

    #[inline]
    pub fn rows(&self, starting_from: MatrixCell) -> Rows {
        Rows::new(starting_from, self.width, self.height)
    }

    /// TODO: docs
    #[inline]
    pub fn top_left(&self) -> MatrixCell {
        MatrixCell(0)
    }

    #[inline]
    pub fn up(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.up(self.width)
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

impl MatrixCell {
    /// TODO: docs
    #[inline]
    fn up(&self, matrix_width: usize) -> Option<Self> {
        if self.0 < matrix_width {
            None
        } else {
            Some(Self(self.0 - matrix_width))
        }
    }

    /// TODO: docs
    #[inline]
    fn down(&self, matrix_width: usize, matrix_height: usize) -> Option<Self> {
        if self.0 + matrix_width >= matrix_height * matrix_width {
            None
        } else {
            Some(Self(self.0 + matrix_width))
        }
    }

    /// TODO: docs
    #[inline]
    fn right(&self, matrix_width: usize) -> Option<Self> {
        let out = if (self.0 + 1) % matrix_width == 0 {
            None
        } else {
            Some(Self(self.0 + 1))
        };

        out
    }

    /// TODO: docs
    #[inline]
    fn left(&self, matrix_width: usize) -> Option<Self> {
        if self.0 % matrix_width == 0 {
            None
        } else {
            Some(Self(self.0 - 1))
        }
    }
}

/// TODO: docs
pub(super) struct Cols {
    /// TODO: docs
    next: Option<MatrixCell>,

    /// TODO: docs
    matrix_width: usize,

    /// TODO: docs
    next_col: ColNext,
}

impl Cols {
    #[inline]
    fn new(start_from: MatrixCell, matrix_width: usize) -> Self {
        Self {
            next: Some(start_from),
            matrix_width,
            next_col: ColNext::default(),
        }
    }

    #[inline]
    pub fn reverse(mut self) -> Self {
        self.next_col.switch();
        self
    }
}

impl Iterator for Cols {
    type Item = MatrixCell;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let this = self.next.take();
        let next =
            this.and_then(|cell| self.next_col.call(cell, self.matrix_width));
        self.next = next;
        this
    }
}

/// TODO: docs
pub(super) struct Rows {
    next: Option<MatrixCell>,
    matrix_height: usize,
    matrix_width: usize,
    next_row: RowNext,
}

impl Rows {
    #[inline]
    fn new(
        start_from: MatrixCell,
        matrix_width: usize,
        matrix_height: usize,
    ) -> Self {
        Self {
            next: Some(start_from),
            matrix_width,
            matrix_height,
            next_row: RowNext::default(),
        }
    }

    #[inline]
    pub fn reverse(mut self) -> Self {
        self.next_row.switch();
        self
    }
}

impl Iterator for Rows {
    type Item = MatrixCell;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let this = self.next.take();

        let next = this.and_then(|cell| {
            self.next_row.call(cell, self.matrix_width, self.matrix_height)
        });

        self.next = next;

        this
    }
}

struct RowNext {
    fun: fn(MatrixCell, usize, usize) -> Option<MatrixCell>,
    is_down: bool,
}

impl Default for RowNext {
    #[inline]
    fn default() -> Self {
        Self { fun: down, is_down: true }
    }
}

impl RowNext {
    fn call(
        &self,
        cell: MatrixCell,
        matrix_width: usize,
        matrix_height: usize,
    ) -> Option<MatrixCell> {
        (self.fun)(cell, matrix_width, matrix_height)
    }

    fn switch(&mut self) {
        self.fun = if self.is_down { up } else { down };
        self.is_down = !self.is_down;
    }
}

struct ColNext {
    fun: fn(MatrixCell, usize) -> Option<MatrixCell>,
    is_right: bool,
}

impl Default for ColNext {
    #[inline]
    fn default() -> Self {
        Self { fun: right, is_right: true }
    }
}

impl ColNext {
    fn call(
        &self,
        cell: MatrixCell,
        matrix_width: usize,
    ) -> Option<MatrixCell> {
        (self.fun)(cell, matrix_width)
    }

    fn switch(&mut self) {
        self.fun = if self.is_right { left } else { right };
        self.is_right = !self.is_right;
    }
}

#[inline]
fn up(cell: MatrixCell, width: usize, _height: usize) -> Option<MatrixCell> {
    cell.up(width)
}

#[inline]
fn down(cell: MatrixCell, width: usize, height: usize) -> Option<MatrixCell> {
    cell.down(width, height)
}

#[inline]
fn left(cell: MatrixCell, width: usize) -> Option<MatrixCell> {
    cell.left(width)
}

#[inline]
fn right(cell: MatrixCell, width: usize) -> Option<MatrixCell> {
    cell.right(width)
}
