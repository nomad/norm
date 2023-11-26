use core::ops::Range;

use super::{query::*, slab::*, *};
use crate::*;

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Clone, Default)]
pub struct FzfV2 {
    /// TODO: docs
    candidate_slab: CandidateSlab,

    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    normalization: bool,

    /// TODO: docs
    scheme: Scheme,

    /// TODO: docs
    slab: V2Slab,
}

impl core::fmt::Debug for FzfV2 {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FzfV2")
            .field("case_sensitivity", &self.case_sensitivity)
            .field("normalization", &self.normalization)
            .field("scheme", &FzfScheme::from_inner(&self.scheme).unwrap())
            .finish_non_exhaustive()
    }
}

impl FzfV2 {
    /// TODO: docs
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    /// TODO: docs
    #[cfg(feature = "tests")]
    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    /// TODO: docs
    #[inline(always)]
    pub fn with_case_sensitivity(
        &mut self,
        case_sensitivity: CaseSensitivity,
    ) -> &mut Self {
        self.case_sensitivity = case_sensitivity;
        self
    }

    /// TODO: docs
    #[inline(always)]
    pub fn with_normalization(&mut self, normalization: bool) -> &mut Self {
        self.normalization = normalization;
        self
    }

    /// TODO: docs
    #[inline(always)]
    pub fn with_scoring_scheme(&mut self, scheme: FzfScheme) -> &mut Self {
        self.scheme = scheme.into_inner();
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

        utils::char_eq(is_sensitive, self.normalization)
    }

    #[inline(always)]
    fn scheme(&self) -> &Scheme {
        &self.scheme
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

        let opts = CandidateOpts::new(is_sensitive, self.normalization);

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
            self.scheme.initial_char_class
        } else {
            char_class(candidate.char(first_offset - 1), &self.scheme)
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
            &self.scheme,
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
