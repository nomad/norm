use core::ops::Range;

use super::{query::*, slab::*, *};
use crate::*;

/// A metric that implements fzf's v2 algorithm.
///
/// The [`Metric`] implementation of this struct produces the same results that
/// `fzf` would produce when run with the `--algo=v2` flag.
///
/// The algorithm used in the [`distance`](Metric::distance) calculation is a
/// modified version of the [Smith-Waterman][sw] algorithm, which was
/// originally designed for finding the best alignment between two DNA or
/// protein sequences.
///
/// Unlike [`FzfV1`], this metric is able to find the best occurrence of a
/// query within a candidate by considering all possible alignments between the
/// two. For example, given the query `"foo"` and the candidate `"f_o_o_foo"`,
/// `FzfV1` would stop at the first match it finds, i.e. `"f_o_o"`. `FzfV2` on
/// the other hand returns the distance and range of the best alignment
/// according to its scoring criteria, which in this case would be `"foo"`.
///
/// ```rust
/// # use norm::fzf::{FzfV1, FzfV2, FzfParser};
/// # use norm::Metric;
/// let mut v1 = FzfV1::new();
/// let mut v2 = FzfV2::new();
/// let mut parser = FzfParser::new();
/// let mut ranges = Vec::new();
///
/// let query = parser.parse("foo");
///
/// let candidate = "f_o_o_foo";
///
/// let distance_v1 =
///     v1.distance_and_ranges(query, candidate, &mut ranges).unwrap();
///
/// assert_eq!(ranges, [0..1, 2..3, 4..5]);
///
/// ranges.clear();
///
/// let distance_v2 =
///     v2.distance_and_ranges(query, candidate, &mut ranges).unwrap();
///
/// assert_eq!(ranges, [6..9]);
///
/// // The alignment found by FzfV2 has a lower distance than the one
/// // found by FzfV1.
/// assert!(distance_v2 < distance_v1);
/// ```
///
/// Of course, this increase in accuracy comes at the cost of a higher time
/// complexity for the distance calculation, namely `O(len(query) *
/// len(candidate))` instead of `O(len(candidate))` for `FzfV1`.
///
/// However, filtering out non-matches is still done in `O(len(candidate))`, so
/// for queries with decent selectivity the performance difference between the
/// two metrics is usually negligible even when dealing with a large number of
/// candidates.
///
///
/// [sw]: https://en.wikipedia.org/wiki/Smith%E2%80%93Waterman_algorithm
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Clone, Default)]
pub struct FzfV2 {
    /// TODO: docs
    candidate_slab: CandidateSlab,

    /// TODO: docs
    candidate_normalization: bool,

    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    scoring_scheme: Scheme,

    /// TODO: docs
    slab: V2Slab,
}

impl core::fmt::Debug for FzfV2 {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Some(scoring_scheme) = FzfScheme::from_inner(&self.scoring_scheme)
        else {
            return Ok(());
        };

        f.debug_struct("FzfV2")
            .field("candidate_normalization", &self.candidate_normalization)
            .field("case_sensitivity", &self.case_sensitivity)
            .field("scoring_scheme", &scoring_scheme)
            .finish_non_exhaustive()
    }
}

impl FzfV2 {
    /// Creates a new `FzfV2`.
    ///
    /// This will immediately allocate around 5kb of heap memory, so it's
    /// recommended to call this once and reuse the same instance for multiple
    /// distance calculations.
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current scoring scheme. This is only used for testing.
    #[cfg(feature = "tests")]
    pub fn scheme(&self) -> &Scheme {
        &self.scoring_scheme
    }

    /// Sets the case sensitivity to use when comparing the characters of the
    /// Sets whether multi-byte latin characters in the candidate should be
    /// normalized to ASCII before comparing them to the query. The default is
    /// `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use norm::fzf::{FzfV2, FzfParser};
    /// # use norm::{Metric, CaseSensitivity};
    /// let mut fzf = FzfV2::new();
    /// let mut parser = FzfParser::new();
    ///
    /// // FzfV2 doesn't normalize candidates by default.
    /// assert!(fzf.distance(parser.parse("foo"), "ƒöö").is_none());
    ///
    /// fzf.set_candidate_normalization(true);
    ///
    /// // With normalization enabled, we get a match.
    /// assert!(fzf.distance(parser.parse("foo"), "ƒöö").is_some());
    ///
    /// // Note that normalization is only applied to the candidate, the query
    /// // is left untouched.
    /// assert!(fzf.distance(parser.parse("ƒöö"), "foo").is_none());
    /// ```
    #[inline(always)]
    pub fn set_candidate_normalization(
        &mut self,
        normalization: bool,
    ) -> &mut Self {
        self.candidate_normalization = normalization;
        self
    }

    /// Sets the case sensitivity to use when comparing the characters of the
    /// query and the candidate. The default is [`CaseSensitivity::Smart`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use norm::fzf::{FzfV2, FzfParser};
    /// # use norm::{Metric, CaseSensitivity};
    /// let mut fzf = FzfV2::new();
    /// let mut parser = FzfParser::new();
    ///
    /// // FzfV2 uses smart case sensitivity by default.
    /// assert!(fzf.distance(parser.parse("abc"), "ABC").is_some());
    ///
    /// fzf.set_case_sensitivity(CaseSensitivity::Sensitive);
    ///
    /// // Now it's case sensitive, so the query won't match the candidate.
    /// assert!(fzf.distance(parser.parse("abc"), "ABC").is_none());
    /// ```
    #[inline(always)]
    pub fn set_case_sensitivity(
        &mut self,
        case_sensitivity: CaseSensitivity,
    ) -> &mut Self {
        self.case_sensitivity = case_sensitivity;
        self
    }

    /// Sets the scoring scheme to use when calculating the distance between
    /// the query and the candidate. The default is [`FzfScheme::Default`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use norm::fzf::{FzfV2, FzfParser, FzfScheme};
    /// # use norm::{Metric};
    /// let mut fzf = FzfV2::new();
    /// let mut parser = FzfParser::new();
    ///
    /// let query = parser.parse("foo");
    ///
    /// // With the default scoring scheme, "f o o" is considered a better
    /// // match than "f/o/o" when searching for "foo".
    /// let distance_spaces = fzf.distance(query, "f o o").unwrap();
    /// let distance_path_separator = fzf.distance(query, "f/o/o").unwrap();
    /// assert!(distance_spaces < distance_path_separator);
    ///
    /// // When searching for a file path we want to use a scoring scheme that
    /// // considers "f/o/o" a better match than "f o o".
    /// fzf.set_scoring_scheme(FzfScheme::Path);
    ///
    /// // Now "f/o/o" is considered a better match than "f o o".
    /// let distance_spaces = fzf.distance(query, "f o o").unwrap();
    /// let distance_path_separator = fzf.distance(query, "f/o/o").unwrap();
    /// assert!(distance_path_separator < distance_spaces);
    /// ```
    #[inline(always)]
    pub fn set_scoring_scheme(&mut self, scheme: FzfScheme) -> &mut Self {
        self.scoring_scheme = scheme.into_inner();
        self
    }
}

impl Metric for FzfV2 {
    type Query<'a> = FzfQuery<'a>;

    type Distance = FzfDistance;

    #[inline(always)]
    fn distance(
        &mut self,
        query: FzfQuery<'_>,
        candidate: &str,
    ) -> Option<Self::Distance> {
        let ranges = &mut Vec::new();
        <Self as Fzf>::distance::<false>(self, query, candidate, ranges)
    }

    #[inline(always)]
    fn distance_and_ranges(
        &mut self,
        query: FzfQuery<'_>,
        candidate: &str,
        ranges: &mut Vec<Range<usize>>,
    ) -> Option<Self::Distance> {
        <Self as Fzf>::distance::<true>(self, query, candidate, ranges)
    }
}

impl Fzf for FzfV2 {
    #[inline(always)]
    fn alloc_chars<'a>(&mut self, s: &str) -> &'a [char] {
        unsafe { core::mem::transmute(self.candidate_slab.alloc(s)) }
    }

    #[inline(always)]
    fn char_eq(&self, pattern: Pattern) -> utils::CharEq {
        let is_sensitive = match self.case_sensitivity {
            CaseSensitivity::Sensitive => true,
            CaseSensitivity::Insensitive => false,
            CaseSensitivity::Smart => pattern.has_uppercase,
        };

        utils::char_eq(is_sensitive, self.candidate_normalization)
    }

    #[inline(always)]
    fn scheme(&self) -> &Scheme {
        &self.scoring_scheme
    }

    #[inline(always)]
    fn fuzzy<const RANGES: bool>(
        &mut self,
        pattern: Pattern,
        candidate: Candidate,
        ranges: &mut MatchedRanges,
    ) -> Option<Score> {
        // TODO: can we remove this?
        if pattern.is_empty() {
            return Some(0);
        }

        let is_sensitive = match self.case_sensitivity {
            CaseSensitivity::Sensitive => true,
            CaseSensitivity::Insensitive => false,
            CaseSensitivity::Smart => pattern.has_uppercase,
        };

        let opts =
            CandidateOpts::new(is_sensitive, self.candidate_normalization);

        if pattern.char_len() == 1 {
            return fuzzy_single_char::<RANGES>(
                pattern.char(0),
                candidate,
                opts,
                self.scheme(),
                ranges,
            );
        }

        let (match_offsets, last_match_offset) =
            matches(&mut self.slab.matched_indices, pattern, candidate, opts)?;

        let first_offset = match_offsets[0];

        let start_byte_offset =
            if RANGES { candidate.to_byte_offset(first_offset) } else { 0 };

        let initial_char_class = if first_offset == 0 {
            self.scoring_scheme.initial_char_class
        } else {
            char_class(candidate.char(first_offset - 1), &self.scoring_scheme)
        };

        let mut candidate = CandidateV2::new(
            candidate.slice(first_offset..last_match_offset),
            &mut self.slab.bonus,
            initial_char_class,
            opts,
        );

        // After slicing the candidate we move all the offsets back by the
        // first offset.
        match_offsets.iter_mut().for_each(|offset| *offset -= first_offset);

        let (scores, consecutive, score, score_cell) = score(
            &mut self.slab.scoring_matrix,
            &mut self.slab.consecutive_matrix,
            pattern,
            &mut candidate,
            match_offsets,
            &self.scoring_scheme,
        );

        if RANGES {
            matched_ranges(
                scores,
                consecutive,
                score_cell,
                candidate.into_base(),
                start_byte_offset,
                ranges,
            );
        };

        Some(score)
    }
}

/// TODO: docs
#[inline]
fn matches<'idx>(
    indices_slab: &'idx mut MatchedIndicesSlab,
    pattern: Pattern,
    candidate: Candidate,
    opts: CandidateOpts,
) -> Option<(&'idx mut [usize], usize)> {
    let match_offsets = indices_slab.alloc(pattern.char_len());

    let mut pattern_char_idx = 0;

    let mut last_match_offset = 0;

    loop {
        let pattern_char = pattern.char(pattern_char_idx);

        last_match_offset = candidate.find_first_from(
            last_match_offset,
            pattern_char,
            opts.is_case_sensitive,
            opts.char_eq,
        )?;

        match_offsets[pattern_char_idx] = last_match_offset;

        last_match_offset += 1;

        if pattern_char_idx + 1 < pattern.char_len() {
            pattern_char_idx += 1;
        } else {
            break;
        }
    }

    let last_char_offset_inclusive = candidate
        .find_last(
            pattern.char(pattern_char_idx),
            opts.is_case_sensitive,
            opts.char_eq,
        )
        .unwrap()
        + 1;

    Some((match_offsets, last_char_offset_inclusive))
}

/// TODO: docs
#[inline]
fn score<'scoring, 'consecutive>(
    scoring_slab: &'scoring mut MatrixSlab<Score>,
    consecutive_slab: &'consecutive mut MatrixSlab<usize>,
    pattern: Pattern,
    candidate: &mut CandidateV2,
    matches: &[usize],
    scheme: &Scheme,
) -> (Matrix<'scoring, Score>, Matrix<'consecutive, usize>, Score, MatrixCell)
{
    let matrix_width = candidate.char_len();

    let matrix_height = pattern.char_len();

    let mut scoring_matrix = scoring_slab.alloc(matrix_width, matrix_height);

    let mut consecutive_matrix =
        consecutive_slab.alloc(matrix_width, matrix_height);

    let (max_score, max_score_cell) = score_first_row(
        scoring_matrix.row_mut(0),
        consecutive_matrix.row_mut(0),
        pattern.char(0),
        candidate,
        scheme,
    );

    let (max_score, max_score_cell) = score_remaining_rows(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        pattern,
        candidate,
        scheme,
        matches,
        max_score,
        max_score_cell,
    );

    (scoring_matrix, consecutive_matrix, max_score, max_score_cell)
}

/// TODO: docs
#[inline]
fn score_first_row(
    scores_first_row: &mut Row<Score>,
    consecutives_first_row: &mut Row<usize>,
    first_pattern_char: char,
    candidate: &mut CandidateV2,
    scheme: &Scheme,
) -> (Score, MatrixCell) {
    let mut max_score: Score = 0;

    let mut prev_score: Score = 0;

    let mut max_score_col: usize = 0;

    let mut column = 0;

    let mut penalty = penalty::GAP_START;

    for char_offset in candidate.matches(first_pattern_char) {
        penalty = penalty::GAP_START;

        for col in column + 1..char_offset {
            let score = prev_score.saturating_sub(penalty);
            scores_first_row[col] = score;
            prev_score = score;
            penalty = penalty::GAP_EXTENSION;
        }

        column = char_offset;

        consecutives_first_row[column] = 1;

        let score = bonus::MATCH
            + (candidate.bonus_at(column, scheme)
                * bonus::FIRST_QUERY_CHAR_MULTIPLIER);

        scores_first_row[column] = score;

        if score > max_score {
            max_score = score;
            max_score_col = column;
        }

        prev_score = score;
    }

    for col in column + 1..scores_first_row.len() {
        let score = prev_score.saturating_sub(penalty);
        scores_first_row[col] = score;
        prev_score = score;
        penalty = penalty::GAP_EXTENSION;
    }

    (max_score, MatrixCell(max_score_col))
}

/// TODO: docs
#[inline]
fn score_remaining_rows(
    scores: &mut Matrix<'_, Score>,
    consecutives: &mut Matrix<'_, usize>,
    pattern: Pattern,
    candidate: &mut CandidateV2,
    scheme: &Scheme,
    matches: &[usize],
    mut max_score: Score,
    mut max_score_cell: MatrixCell,
) -> (Score, MatrixCell) {
    let matrix_width = scores.width();

    for row_idx in 1..scores.height() {
        let pattern_char = pattern.char(row_idx);

        let (prev_scores_row, scores_row) =
            scores.two_rows_mut(row_idx - 1, row_idx);

        let (prev_consecutives_row, consecutives_row) =
            consecutives.two_rows_mut(row_idx - 1, row_idx);

        let first_match_offset = matches[row_idx];

        let mut column = first_match_offset;

        let mut penalty = penalty::GAP_START;

        for char_offset in
            candidate.matches_from(first_match_offset, pattern_char)
        {
            penalty = penalty::GAP_START;

            for col in column + 1..char_offset {
                let score_left = scores_row[col - 1];
                let score = score_left.saturating_sub(penalty);
                scores_row[col] = score;
                penalty = penalty::GAP_EXTENSION;
            }

            column = char_offset;

            let score_left = scores_row[column - 1].saturating_sub(penalty);

            let mut score_up_left = prev_scores_row[column - 1] + bonus::MATCH;

            let mut bonus = candidate.bonus_at(column, scheme);

            let mut consecutive = prev_consecutives_row[column - 1] + 1;

            if consecutive > 1 {
                let fb = candidate.bonus_at(column + 1 - consecutive, scheme);

                if bonus >= bonus::BOUNDARY && bonus > fb {
                    consecutive = 1;
                } else {
                    bonus = bonus::CONSECUTIVE.max(fb).max(bonus);
                }
            }

            score_up_left += if score_up_left + bonus < score_left {
                consecutive = 0;
                candidate.bonus_at(column, scheme)
            } else {
                bonus
            };

            let score = score_left.max(score_up_left);

            if score > max_score {
                max_score = score;
                max_score_cell = MatrixCell(row_idx * matrix_width + column);
            }

            consecutives_row[column] = consecutive;

            scores_row[column] = score;
        }

        for col in column + 1..matrix_width {
            let score_left = scores_row[col - 1];
            let score = score_left.saturating_sub(penalty);
            scores_row[col] = score;
            penalty = penalty::GAP_EXTENSION;
        }
    }

    (max_score, max_score_cell)
}

/// TODO: docs
#[inline]
fn matched_ranges(
    scores: Matrix<Score>,
    consecutives: Matrix<usize>,
    max_score_cell: MatrixCell,
    candidate: Candidate,
    start_byte_offset: usize,
    ranges: &mut MatchedRanges,
) {
    let mut prefer_match = true;

    let mut cell = max_score_cell;

    let mut col = scores.col_of(max_score_cell);

    let mut row = scores.row_of(max_score_cell);

    loop {
        let is_cell_in_first_col = col == 0;

        let is_cell_in_first_row = row == 0;

        let score_left =
            if is_cell_in_first_col { 0 } else { scores[scores.left(cell)] };

        let score_up_left = if is_cell_in_first_col || is_cell_in_first_row {
            0
        } else {
            scores[scores.up_left(cell)]
        };

        let prefer_this_match = prefer_match;

        prefer_match = consecutives[cell] > 1
            || consecutives
                .get_value(consecutives.down_right(cell))
                .map_or(false, |down_right| down_right > 0);

        let score = scores[cell];

        if score > score_up_left
            && (score > score_left || score == score_left && prefer_this_match)
        {
            let mut byte_offset = candidate.to_byte_offset(col);

            let ch = candidate.char(col);

            byte_offset += start_byte_offset;

            ranges.insert(byte_offset..byte_offset + ch.len_utf8());

            if is_cell_in_first_row || is_cell_in_first_col {
                break;
            } else {
                row -= 1;
                cell = scores.up_left(cell);
            }
        } else if is_cell_in_first_col {
            break;
        } else {
            cell = scores.left(cell);
        }

        col -= 1;
    }
}

/// TODO: docs
#[inline]
fn fuzzy_single_char<const RANGES: bool>(
    pattern_char: char,
    candidate: Candidate,
    opts: CandidateOpts,
    scheme: &Scheme,
    ranges: &mut MatchedRanges,
) -> Option<Score> {
    let mut max_score = 0;

    let mut max_score_pos = 0;

    for char_offset in
        candidate.matches(pattern_char, opts.is_case_sensitive, opts.char_eq)
    {
        let prev_class = if char_offset == 0 {
            scheme.initial_char_class
        } else {
            char_class(candidate.char(char_offset - 1), scheme)
        };

        let this_class = char_class(candidate.char(char_offset), scheme);

        let bonus = compute_bonus(prev_class, this_class, scheme);

        let score = bonus::MATCH + bonus * bonus::FIRST_QUERY_CHAR_MULTIPLIER;

        if score > max_score {
            max_score = score;
            max_score_pos = char_offset;
        }
    }

    if max_score == 0 {
        return None;
    }

    if RANGES {
        let start = candidate.to_byte_offset(max_score_pos);
        let byte_len = candidate.char(max_score_pos).len_utf8();
        ranges.insert(start..start + byte_len);
    }

    Some(max_score)
}
