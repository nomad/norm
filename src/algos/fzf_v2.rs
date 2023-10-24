use core::ops::{Index, IndexMut, Range};

use super::fzf_v1::*;
use crate::{CaseMatcher, CaseSensitivity, Match, Metric};

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Clone, Default)]
pub struct FzfV2 {
    /// TODO: docs
    bonus_vector_slab: BonusVectorSlab,

    /// TODO: docs
    candidate_slab: CandidateSlab,

    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    matched_indices_slab: MatchedIndicesSlab,

    /// TODO: docs
    scoring_matrix_slab: ScoringMatrixSlab,

    /// TODO: docs
    scheme: scheme::Scheme,

    /// TODO: docs
    with_matched_ranges: bool,
}

impl FzfV2 {
    /// TODO: docs
    #[inline]
    fn fuzzy_match(&self, query: &str, candidate: &str) -> Option<Score> {
        // Phase 1.

        let m = query.len();

        let n = candidate.len();

        let mut idx = 0;

        let mut h0 = vec![0; n];

        let mut c0 = vec![0; n];

        let mut b = vec![0; n];

        let mut f = vec![0; m];

        let t = candidate.chars().collect::<Vec<_>>();

        // Phase 2.

        let mut max_score = 0;

        let mut max_score_pos = 0;

        let mut query_chars = query.chars();

        let mut query_char = query_chars.next().expect("query is not empty");

        let mut last_idx = 0;

        let first_query_char = query_char;

        let mut prev_h0: Score = 0;

        let mut prev_class = self.scheme.initial_char_class;

        let mut is_in_gap = false;

        let h0_sub = &mut h0[..];

        let c0_sub = &mut c0[..];

        let b_sub = &mut b[..];

        let case_matcher = self.case_sensitivity.matcher(query);

        let mut pidx = 0;

        for (offset, candidate_char) in candidate.char_indices() {
            let char_class = char_class(candidate_char, &self.scheme);

            let bonus = bonus(prev_class, char_class, &self.scheme);

            b_sub[offset] = bonus;

            prev_class = char_class;

            if case_matcher.eq(query_char, candidate_char) {
                last_idx = offset;
                f[pidx] = offset;
                pidx += 1;

                if let Some(next_char) = query_chars.next() {
                    query_char = next_char;
                }
            }

            if case_matcher.eq(first_query_char, candidate_char) {
                let score =
                    bonus::MATCH + bonus * bonus::FIRST_QUERY_CHAR_MULTIPLIER;

                h0_sub[offset] = score;
                c0_sub[offset] = 1;

                if m == 1 && score > max_score {
                    max_score = score;
                    max_score_pos = offset;
                    if bonus >= bonus::BOUNDARY {
                        break;
                    }
                }

                is_in_gap = false;
            } else {
                let penalty = if is_in_gap {
                    penalty::GAP_EXTENSION
                } else {
                    penalty::GAP_START
                };
                h0_sub[offset] = prev_h0.saturating_sub(penalty);
                c0_sub[offset] = 0;
                is_in_gap = true;
            }

            prev_h0 = h0_sub[offset];
        }

        if pidx != m {
            return None;
        }

        if m == 1 {
            // TODO: return score.
            return Some(max_score);
        }

        println!("f: {f:?}");
        println!("t: {t:?}");
        println!("b: {b:?}");
        println!("h0: {h0:?}");
        println!("c0: {c0:?}");
        println!("last_idx: {last_idx:?}");

        // panic!();

        // Phase 3.
        //
        // Fill in score matrix H.

        let score = phase_3(
            &f,
            &t,
            &b,
            &h0,
            &c0,
            query,
            last_idx,
            self.case_sensitivity,
        );

        Some(score)
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// TODO: docs
    #[inline]
    pub fn with_case_sensitivity(
        mut self,
        case_sensitivity: CaseSensitivity,
    ) -> Self {
        self.case_sensitivity = case_sensitivity;
        self
    }

    /// TODO: docs
    #[inline]
    pub fn with_matched_ranges(mut self, matched_ranges: bool) -> Self {
        self.with_matched_ranges = matched_ranges;
        self
    }

    /// TODO: docs
    #[inline]
    pub fn with_scoring_scheme(mut self, scheme: FzfScheme) -> Self {
        self.scheme = match scheme {
            FzfScheme::Default => scheme::DEFAULT,
            FzfScheme::Path => scheme::PATH,
            FzfScheme::History => scheme::HISTORY,
        };
        self
    }
}

impl Metric for FzfV2 {
    type Query<'a> = FzfQuery<'a>;

    type Distance = FzfDistance;

    #[inline]
    fn distance(
        &mut self,
        query: FzfQuery<'_>,
        candidate: &str,
    ) -> Option<Match<Self::Distance>> {
        if query.is_empty() {
            return None;
        }

        let query = query.raw();

        let case_matcher = self.case_sensitivity.matcher(query);

        let candidate = self.candidate_slab.alloc(candidate);

        let (matched_indices, bonus_vector) = matched_indices(
            &mut self.matched_indices_slab,
            &mut self.bonus_vector_slab,
            query,
            candidate,
            &case_matcher,
            &self.scheme,
        )?;

        println!("matched_indices: {:?}", matched_indices);

        println!("bonus_vector: {:?}", bonus_vector);

        let (scoring_matrix, score, score_cell) = score(
            &mut self.scoring_matrix_slab,
            query,
            candidate,
            matched_indices,
            bonus_vector,
            &case_matcher,
            &self.scheme,
        );

        todo!()

        // let distance = FzfDistance::from_score(score);

        // Some(Match::new(distance, Vec::new()))
    }
}

/// TODO: docs
#[inline]
fn matched_indices<'idx, 'bonus>(
    indices_slab: &'idx mut MatchedIndicesSlab,
    bonuses_slab: &'bonus mut BonusVectorSlab,
    query: &str,
    candidate: Candidate<'_>,
    case_matcher: &CaseMatcher,
    scheme: &scheme::Scheme,
) -> Option<(MatchedIndices<'idx>, BonusVector<'bonus>)> {
    let mut query_chars = query.chars();

    let mut query_char = query_chars.next().expect("query is not empty");

    let mut prev_class = scheme.initial_char_class;

    let mut bonuses = bonuses_slab.alloc(candidate);

    let mut matched_idxs = indices_slab.alloc(query);

    for (char_idx, candidate_char) in candidate.char_idxs() {
        let char_class = char_class(candidate_char, scheme);
        let bonus = bonus(prev_class, char_class, scheme);
        prev_class = char_class;

        bonuses[char_idx] = bonus;

        if case_matcher.eq(query_char, candidate_char) {
            matched_idxs.push(char_idx);

            if let Some(next_char) = query_chars.next() {
                query_char = next_char;
            }
        }
    }

    // TODO: use query.char_len()
    if matched_idxs.len() == query.chars().count() {
        Some((matched_idxs, bonuses))
    } else {
        None
    }
}

/// TODO: docs
#[inline]
fn score<'scoring>(
    matrix_slab: &'scoring mut ScoringMatrixSlab,
    query: &str,
    candidate: Candidate,
    matched_indices: MatchedIndices,
    bonus_vector: BonusVector,
    case_matcher: &CaseMatcher,
    scheme: &scheme::Scheme,
) -> (ScoringMatrix<'scoring>, Score, MatrixCell) {
    let mut matrix = matrix_slab.alloc(query, candidate);

    // The char index in the candidate string of the character that matched the
    // last character in the query string.
    let last_matched_idx = matched_indices.last();

    let mut chars_idxs_rows = query
        .chars()
        .zip(matched_indices.into_iter())
        .zip(matrix.rows(matrix.top_left()))
        .map(|((query_char, matched_idx), row)| {
            (query_char, matched_idx, row)
        });

    let (first_query_char, first_matched_idx, _) =
        chars_idxs_rows.next().expect("the query is not empty");

    let (max_score, max_score_cell) = score_first_row(
        &mut matrix,
        first_query_char,
        first_matched_idx,
        last_matched_idx,
        candidate,
        &bonus_vector,
        case_matcher,
    );

    let (max_score, max_score_cell) = score_remaining_rows(
        &mut matrix,
        chars_idxs_rows,
        last_matched_idx,
        max_score,
        max_score_cell,
        candidate,
        bonus_vector,
        case_matcher,
    );

    (matrix, max_score, max_score_cell)
}

/// TODO: docs
#[inline]
fn score_first_row(
    matrix: &mut ScoringMatrix,
    first_query_char: char,
    first_matched_idx: CandidateCharIdx,
    last_matched_idx: CandidateCharIdx,
    candidate: Candidate,
    bonus_vector: &BonusVector,
    case_matcher: &CaseMatcher,
) -> (Score, MatrixCell) {
    let mut max_score: Score = 0;

    let mut max_score_cell = matrix.top_left();

    let mut prev_score: Score = 0;

    let mut is_in_gap = false;

    let candidate = candidate.slice(first_matched_idx..last_matched_idx);

    let mut candidate_chars = candidate.char_idxs();

    let starting_col = matrix
        .right_n(matrix.top_left(), first_matched_idx.into_usize())
        .expect("TODO");

    for cell in matrix.cols(starting_col) {
        let (char_idx, candidate_char) = candidate_chars.next().expect(
            "the scoring matrix's width is equal to the candidate's char \
             length",
        );

        let bonus = bonus_vector[char_idx];

        let score = if case_matcher.eq(first_query_char, candidate_char) {
            is_in_gap = false;

            let score =
                bonus::MATCH + (bonus * bonus::FIRST_QUERY_CHAR_MULTIPLIER);

            if score > max_score {
                max_score = score;
                max_score_cell = cell;
            }

            score
        } else {
            let penalty = if is_in_gap {
                penalty::GAP_EXTENSION
            } else {
                penalty::GAP_START
            };

            is_in_gap = true;

            prev_score.saturating_sub(penalty)
        };

        matrix[cell] = score;

        prev_score = score;
    }

    (max_score, max_score_cell)
}

/// TODO: docs
#[inline]
fn score_remaining_rows<I>(
    matrix: &mut ScoringMatrix,
    chars_idxs_rows: I,
    last_matched_idx: CandidateCharIdx,
    mut max_score: Score,
    mut max_score_cell: MatrixCell,
    candidate: Candidate,
    bonus_vector: BonusVector,
    case_matcher: &CaseMatcher,
) -> (Score, MatrixCell)
where
    I: Iterator<Item = (char, CandidateCharIdx, MatrixCell)>,
{
    for (query_char, matched_idx, first_col_cell) in chars_idxs_rows {
        // TODO: docs
        let starting_col = {
            let skipped_cols = matched_idx.into_usize();
            matrix.right_n(first_col_cell, skipped_cols).unwrap()
        };

        // TODO: docs
        let left_of_starting_col = matrix.left(starting_col).unwrap();

        // TODO: docs
        let up_left_of_starting_col = matrix.up(left_of_starting_col).unwrap();

        // TODO: docs
        let mut cols = matrix
            .cols(starting_col)
            .zip(matrix.cols(left_of_starting_col))
            .zip(matrix.cols(up_left_of_starting_col));

        let candidate = candidate.slice(matched_idx..last_matched_idx);

        let mut is_in_gap = false;

        for (char_idx, candidate_char) in candidate.char_idxs() {
            let ((cell, left_cell), up_left_cell) = cols.next().unwrap();

            let score_left = matrix[left_cell] as i32
                - if is_in_gap {
                    penalty::GAP_EXTENSION
                } else {
                    penalty::GAP_START
                } as i32;

            let score_up_left = if case_matcher.eq(query_char, candidate_char)
            {
                let score = matrix[up_left_cell] + bonus::MATCH;

                let bonus = bonus_vector[char_idx];

                score
            } else {
                0
            };

            let score = score_up_left.max(score_left as Score).max(0);

            if score > max_score {
                max_score = score;
                max_score_cell = cell;
            }

            matrix[cell] = score;
        }
    }

    (max_score, max_score_cell)
}

fn phase_3(
    f: &[usize],
    t: &[char],
    b: &[Score],
    h0: &[Score],
    c0: &[usize],
    query: &str,
    last_idx: usize,
    case_sensitivity: CaseSensitivity,
) -> Score {
    let m = query.len();

    let mut max_score = 0;

    let mut max_score_pos = 0;

    let f0 = f[0];

    let width = last_idx - f0 + 1;

    let mut h = vec![0; m * width];

    h[..last_idx + 1 - f0].copy_from_slice(&h0[f0..last_idx + 1]);

    // Possible length of consecutive chunk at each position.
    let mut c = vec![0; m * width];

    c[..last_idx + 1 - f0].copy_from_slice(&c0[f0..last_idx + 1]);

    let f_sub = &f[1..];

    let p_sub = &query[1..][..f_sub.len()];

    let case_matcher = case_sensitivity.matcher(query);

    for (offset, f) in f_sub.iter().enumerate() {
        let c_len = c.len();
        let c_ptr = c.as_mut_ptr();

        let h_len = h.len();
        let h_ptr = h.as_mut_ptr();

        let pchar = p_sub.chars().nth(offset).unwrap();
        let pidx = offset + 1;
        let row = pidx * width;
        let mut is_in_gap = false;
        let t_sub = &t[*f..last_idx + 1];
        let b_sub = &b[*f..][..t_sub.len()];
        let c_sub = &mut c[row + f - f0..][..t_sub.len()];
        let h_sub = &mut h[row + f - f0..][..t_sub.len()];

        let c_diag = {
            let c = unsafe { core::slice::from_raw_parts_mut(c_ptr, c_len) };
            &mut c[row + f - f0 - 1 - width..][..t_sub.len()]
        };

        let h_diag = {
            let h = unsafe { core::slice::from_raw_parts_mut(h_ptr, h_len) };
            &mut h[row + f - f0 - 1 - width..][..t_sub.len()]
        };

        let h_left = {
            let h = unsafe { core::slice::from_raw_parts_mut(h_ptr, h_len) };
            &mut h[row + f - f0 - 1..][..t_sub.len()]
        };

        // println!("-------------");
        // println!("pchar: {:?}", pchar);
        // println!("pidx: {:?}", pidx);
        // println!("row: {:?}", row);
        // println!("inGap: {:?}", is_in_gap);
        // println!("Tsub: {:?}", t_sub);
        // println!("Bsub: {:?}", b_sub);
        // println!("Csub: {:?}", c_sub);
        // println!("Cdiag: {:?}", c_diag);
        // println!("Hsub: {:?}", h_sub);
        // println!("Hdiag: {:?}", h_diag);
        // println!("Hleft: {:?}", h_left);
        // println!("-------------");

        h_left[0] = 0;

        for (offset, &char) in t_sub.iter().enumerate() {
            let col = offset + f;

            let mut s1 = 0;

            let mut consecutive = 0;

            let s2 = if is_in_gap {
                h_left[offset] as i32 - penalty::GAP_EXTENSION as i32
            } else {
                h_left[offset] as i32 - penalty::GAP_START as i32
            };

            if case_matcher.eq(pchar, char) {
                s1 = h_diag[offset] + bonus::MATCH;
                let mut small_b = b_sub[offset];
                consecutive = c_diag[offset] + 1;

                if consecutive > 1 {
                    let fb = b[col - consecutive + 1];
                    if small_b >= bonus::BOUNDARY && small_b > fb {
                        consecutive = 1;
                    } else {
                        small_b = bonus::CONSECUTIVE.max(fb).max(small_b);
                    }
                }

                if (s1 as i32) + (small_b as i32) < s2 {
                    s1 += b_sub[offset];
                    consecutive = 0;
                } else {
                    s1 += small_b;
                }
            }

            // println!("-------------");
            // println!("s1: {:?}", s1);
            // println!("s2: {:?}", s2);
            // println!("offset: {:?}", offset);

            c_sub[offset] = consecutive;

            is_in_gap = (s1 as i32) < s2;

            let score = (s1 as i32).max(s2).max(0) as Score;

            if pidx == m - 1 && score > max_score {
                max_score = score;
                max_score_pos = col;
            }

            // println!("-------------");
            // println!("score: {:?}", score);
            // println!("offset: {:?}", offset);

            h_sub[offset] = score;
        }
    }

    // println!("f0: {:?}", f0);
    // println!("width: {:?}", width);
    // println!("h: {:?}", h);
    // println!("c: {:?}", c);
    // println!("f_sub: {:?}", f_sub);
    // println!("p_bub: {:?}", p_sub);

    let j = f0;

    max_score
}

/// TODO: docs
#[derive(Clone)]
struct CandidateSlab {
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
    fn alloc<'a>(&'a mut self, candidate: &str) -> Candidate<'a> {
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
            byte_offset: 0,
            char_offset: 0,
        }
    }
}

/// TODO: docs
#[derive(Clone, Copy)]
struct Candidate<'a> {
    chars: &'a [char],
    char_offsets: &'a [usize],
    byte_offset: usize,
    char_offset: usize,
}

impl core::fmt::Debug for Candidate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.chars.iter().collect::<String>().fmt(f)
    }
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateCharIdx(usize);

impl CandidateCharIdx {
    #[inline]
    fn into_usize(self) -> usize {
        self.0
    }
}

impl<'a> Candidate<'a> {
    /// TODO: docs
    #[inline]
    fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars.iter().copied()
    }

    /// TODO: docs
    #[inline]
    fn char_idxs(
        &self,
    ) -> impl Iterator<Item = (CandidateCharIdx, char)> + '_ {
        self.chars.iter().enumerate().map(|(idx, &char)| {
            (CandidateCharIdx(idx + self.char_offset), char)
        })
    }

    /// TODO: docs
    #[inline]
    fn char_len(&self) -> usize {
        self.chars.len()
    }

    /// TODO: docs
    #[inline]
    fn char_offsets(&self) -> impl Iterator<Item = (usize, char)> + '_ {
        self.char_offsets
            .iter()
            .zip(self.chars)
            .map(|(&offset, &char)| (offset + self.byte_offset, char))
    }

    /// TODO: docs
    #[inline]
    fn nth_char(&self, idx: usize) -> char {
        self.chars[idx]
    }

    /// TODO: docs
    #[inline]
    fn nth_char_offset(&self, idx: usize) -> usize {
        self.char_offsets[idx] + self.byte_offset
    }

    /// TODO: docs
    #[inline]
    fn slice(self, range: Range<CandidateCharIdx>) -> Self {
        let range = range.start.0..range.end.0 + 1;
        let chars = &self.chars[range.clone()];
        let char_offsets = &self.char_offsets[range.clone()];
        let char_offset = self.char_offset + range.start;
        let byte_offset = self.byte_offset + self.char_offsets[range.start];
        Self { chars, char_offsets, char_offset, byte_offset }
    }
}

/// TODO: docs
#[derive(Clone)]
struct MatchedIndicesSlab {
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
    fn alloc<'a>(&'a mut self, query: &str) -> MatchedIndices<'a> {
        let char_len = query.chars().count();

        if char_len > self.vec.len() {
            self.vec.resize(char_len, CandidateCharIdx(0));
        }

        MatchedIndices::new(&mut self.vec[..char_len])
    }
}

/// TODO: docs
struct MatchedIndices<'a> {
    indices: &'a mut [CandidateCharIdx],
    len: usize,
}

impl<'a> MatchedIndices<'a> {
    /// TODO: docs
    #[inline]
    fn into_iter(self) -> impl Iterator<Item = CandidateCharIdx> + 'a {
        self.indices[..self.len].into_iter().copied()
    }

    /// TODO: docs
    #[inline]
    fn last(&self) -> CandidateCharIdx {
        self.indices[self.len - 1]
    }

    /// TODO: docs
    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn new(indices: &'a mut [CandidateCharIdx]) -> Self {
        Self { indices, len: 0 }
    }

    /// TODO: docs
    #[inline]
    fn push(&mut self, idx: CandidateCharIdx) {
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
struct BonusVectorSlab {
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
    fn alloc<'a>(&'a mut self, candidate: Candidate) -> BonusVector<'a> {
        let char_len = candidate.char_len();

        if char_len > self.vec.len() {
            self.vec.resize(char_len, 0);
        }

        BonusVector { indices: &mut self.vec[..char_len] }
    }
}

/// TODO: docs
struct BonusVector<'a> {
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

/// TODO: docs
#[derive(Default, Clone)]
struct ScoringMatrixSlab {
    vec: Vec<Score>,
}

impl ScoringMatrixSlab {
    /// TODO: docs
    #[inline]
    fn alloc<'a>(
        &'a mut self,
        query: &str,
        candidate: Candidate,
    ) -> ScoringMatrix<'a> {
        let height = query.chars().count();

        let width = candidate.char_len();

        debug_assert!(height * width > 0);

        if height * width > self.vec.len() {
            self.vec.resize(height * width, 0);
        }

        let slice = &mut self.vec[..height * width];

        slice.fill(0);

        ScoringMatrix { slice, height, width }
    }
}

/// TODO: docs
#[derive(Default)]
struct ScoringMatrix<'a> {
    /// TODO: docs
    ///
    /// <width><width>...<width>
    /// \---- height times ----/
    slice: &'a mut [Score],
    height: usize,
    width: usize,
}

impl core::fmt::Debug for ScoringMatrix<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;

        // The matrix should never be empty, but just in case.
        if self.slice.is_empty() {
            return f.write_str("[ ]");
        }

        fn printed_width(score: Score) -> usize {
            if score == 0 {
                1
            } else {
                (score.ilog10() + 1) as usize
            }
        }

        // The character width of the biggest score in the whole matrix.
        let max_score_width = {
            let max_score = self.slice.iter().copied().max().unwrap();
            printed_width(max_score)
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

            printed_width(last_col_max_score)
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
                let score = self[cell];

                write!(f, "{score}", score = self[cell])?;

                let num_spaces = if self.is_in_last_col(cell) {
                    last_col_max_score_width - printed_width(score)
                } else {
                    max_score_width - printed_width(score) + 1
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

impl<'a> ScoringMatrix<'a> {
    #[inline]
    fn cols(&self, starting_from: MatrixCell) -> Cols {
        Cols { next: Some(starting_from), matrix_width: self.width }
    }

    #[inline]
    fn down(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.down(self.width, self.height)
    }

    /// TODO: docs
    #[inline]
    fn is_first_row(&self, cell: MatrixCell) -> bool {
        self.up(cell).is_none()
    }

    /// TODO: docs
    #[inline]
    fn is_in_last_col(&self, cell: MatrixCell) -> bool {
        self.right(cell).is_none()
    }

    /// TODO: docs
    #[inline]
    fn is_last_row(&self, cell: MatrixCell) -> bool {
        self.down(cell).is_none()
    }

    #[inline]
    fn left(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.left(self.width)
    }

    #[inline]
    fn right(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.right(self.width)
    }

    #[inline]
    fn right_n(&self, cell: MatrixCell, n: usize) -> Option<MatrixCell> {
        if n == 0 {
            Some(cell)
        } else {
            (MatrixCell(cell.0 + n - 1)).right(self.width)
        }
    }

    #[inline]
    fn rows(&self, starting_from: MatrixCell) -> Rows {
        Rows {
            next: Some(starting_from),
            matrix_width: self.width,
            matrix_height: self.height,
        }
    }

    /// TODO: docs
    #[inline]
    fn top_left(&self) -> MatrixCell {
        MatrixCell(0)
    }

    #[inline]
    fn up(&self, cell: MatrixCell) -> Option<MatrixCell> {
        cell.up(self.width)
    }
}

#[derive(Debug, Clone, Copy)]
struct MatrixCell(usize);

impl Index<MatrixCell> for ScoringMatrix<'_> {
    type Output = Score;

    #[inline]
    fn index(&self, index: MatrixCell) -> &Self::Output {
        &self.slice[index.0]
    }
}

impl IndexMut<MatrixCell> for ScoringMatrix<'_> {
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
struct Cols {
    next: Option<MatrixCell>,
    matrix_width: usize,
}

impl Iterator for Cols {
    type Item = MatrixCell;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let this = self.next.take();
        let next = this.and_then(|cell| cell.right(self.matrix_width));
        self.next = next;
        this
    }
}

/// TODO: docs
struct Rows {
    next: Option<MatrixCell>,
    matrix_height: usize,
    matrix_width: usize,
}

impl Iterator for Rows {
    type Item = MatrixCell;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let this = self.next.take();
        let next = this
            .and_then(|cell| cell.down(self.matrix_width, self.matrix_height));
        self.next = next;
        this
    }
}
