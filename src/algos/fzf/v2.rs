use core::ops::Range;

use super::{slab::*, *};
use crate::{CaseMatcher, CaseSensitivity, Match, Metric};

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Clone, Default)]
pub struct FzfV2 {
    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    scheme: Scheme,

    /// TODO: docs
    slab: V2Slab,

    /// TODO: docs
    with_matched_ranges: bool,
}

impl FzfV2 {
    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// TODO: docs
    #[cfg(feature = "tests")]
    pub fn scheme(&self) -> &Scheme {
        &self.scheme
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
        self.scheme = scheme.into_inner();
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

        let candidate = self.slab.candidate.alloc(candidate);

        let case_matcher = self.case_sensitivity.matcher(query);

        let (matched_indices, last_matched_idx, bonus_vector) =
            matched_indices(
                &mut self.slab.matched_indices,
                &mut self.slab.bonus_vector,
                query,
                candidate,
                &case_matcher,
                &self.scheme,
            )?;

        let (scores, consecutive, score, score_cell) = score(
            &mut self.slab.scoring_matrix,
            &mut self.slab.consecutive_matrix,
            query,
            candidate,
            matched_indices,
            last_matched_idx,
            bonus_vector,
            case_matcher,
        );

        let matched_ranges = if self.with_matched_ranges {
            matched_ranges(scores, consecutive, score_cell, candidate)
        } else {
            Vec::new()
        };

        let distance = FzfDistance::from_score(score);

        Some(Match::new(distance, matched_ranges))
    }
}

/// TODO: docs
#[inline]
fn matched_indices<'idx, 'bonus>(
    indices_slab: &'idx mut MatchedIndicesSlab,
    bonuses_slab: &'bonus mut BonusVectorSlab,
    query: FzfQuery,
    candidate: Candidate,
    case_matcher: &CaseMatcher,
    scheme: &Scheme,
) -> Option<(MatchedIndices<'idx>, CandidateCharIdx, BonusVector<'bonus>)> {
    let mut query_chars = query.chars();

    let mut query_char = query_chars.next().expect("query is not empty");

    let mut prev_class = scheme.initial_char_class;

    let mut matched_idxs = indices_slab.alloc(query);

    let mut last_matched_idx = CandidateCharIdx(0);

    let mut bonuses = bonuses_slab.alloc(candidate);

    for (char_idx, candidate_char) in candidate.char_idxs() {
        let char_class = char_class(candidate_char, scheme);
        let bonus = bonus(prev_class, char_class, scheme);
        prev_class = char_class;

        bonuses[char_idx] = bonus;

        if case_matcher.eq(query_char, candidate_char) {
            if !matched_idxs.is_full() {
                matched_idxs.push(char_idx);
            }

            last_matched_idx = char_idx;

            if let Some(next_char) = query_chars.next() {
                query_char = next_char;
            }
        }
    }

    if matched_idxs.is_full() {
        Some((matched_idxs, last_matched_idx, bonuses))
    } else {
        None
    }
}

/// TODO: docs
#[allow(clippy::too_many_arguments)]
#[inline]
fn score<'scoring, 'consecutive>(
    scoring_slab: &'scoring mut ScoringMatrixSlab,
    consecutive_slab: &'consecutive mut ConsecutiveMatrixSlab,
    query: FzfQuery,
    candidate: Candidate,
    matched_indices: MatchedIndices,
    last_matched_idx: CandidateCharIdx,
    bonus_vector: BonusVector,
    case_matcher: CaseMatcher,
) -> (Matrix<'scoring, Score>, Matrix<'consecutive, usize>, Score, MatrixCell)
{
    let mut scoring_matrix = scoring_slab.alloc(query, candidate);

    let mut consecutive_matrix = consecutive_slab.alloc(query, candidate);

    let mut chars_idxs_rows = query
        .chars()
        .zip(matched_indices.into_iter())
        .zip(scoring_matrix.rows(scoring_matrix.top_left()))
        .map(|((query_char, matched_idx), row)| {
            (query_char, matched_idx, row)
        });

    let (first_query_char, first_matched_idx, _) =
        chars_idxs_rows.next().expect("the query is not empty");

    let (max_score, max_score_cell) = score_first_row(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        first_query_char,
        first_matched_idx,
        last_matched_idx,
        candidate,
        &bonus_vector,
        &case_matcher,
    );

    let (max_score, max_score_cell) = score_remaining_rows(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        chars_idxs_rows,
        last_matched_idx,
        max_score,
        max_score_cell,
        candidate,
        bonus_vector,
        case_matcher,
    );

    (scoring_matrix, consecutive_matrix, max_score, max_score_cell)
}

/// TODO: docs
#[allow(clippy::too_many_arguments)]
#[inline]
fn score_first_row(
    scoring_matrix: &mut Matrix<'_, Score>,
    consecutive_matrix: &mut Matrix<'_, usize>,
    first_query_char: char,
    first_matched_idx: CandidateCharIdx,
    last_matched_idx: CandidateCharIdx,
    candidate: Candidate,
    bonus_vector: &BonusVector,
    case_matcher: &CaseMatcher,
) -> (Score, MatrixCell) {
    let mut max_score: Score = 0;

    let mut max_score_cell = scoring_matrix.top_left();

    let mut prev_score: Score = 0;

    let mut is_in_gap = false;

    let candidate = candidate.slice(first_matched_idx..last_matched_idx);

    let starting_col = scoring_matrix
        .right_n(scoring_matrix.top_left(), first_matched_idx.into_usize())
        .expect(
            "the index of the first matched character is within the scoring \
             matrix's width",
        );

    let mut cols = scoring_matrix.cols(starting_col);

    for (char_idx, candidate_char) in candidate.char_idxs() {
        let cell = cols.next().expect(
            "the scoring matrix's width >= than the sliced candidate's char \
             length",
        );

        let bonus = bonus_vector[char_idx];

        let chars_match = case_matcher.eq(first_query_char, candidate_char);

        consecutive_matrix[cell] = chars_match as usize;

        let score = if chars_match {
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

        scoring_matrix[cell] = score;

        prev_score = score;
    }

    (max_score, max_score_cell)
}

/// TODO: docs
#[allow(clippy::too_many_arguments)]
#[inline]
fn score_remaining_rows<I>(
    scoring_matrix: &mut Matrix<'_, Score>,
    consecutive_matrix: &mut Matrix<'_, usize>,
    chars_idxs_rows: I,
    last_matched_idx: CandidateCharIdx,
    mut max_score: Score,
    mut max_score_cell: MatrixCell,
    candidate: Candidate,
    bonus_vector: BonusVector,
    case_matcher: CaseMatcher,
) -> (Score, MatrixCell)
where
    I: Iterator<Item = (char, CandidateCharIdx, MatrixCell)>,
{
    for (query_char, matched_idx, first_col_cell) in chars_idxs_rows {
        // TODO: docs
        let starting_col = {
            let skipped_cols = matched_idx.into_usize();
            scoring_matrix.right_n(first_col_cell, skipped_cols).unwrap()
        };

        // TODO: docs
        let left_of_starting_col = scoring_matrix.left(starting_col).unwrap();

        // TODO: docs
        let up_left_of_starting_col =
            scoring_matrix.up(left_of_starting_col).unwrap();

        // TODO: docs
        let mut cols = scoring_matrix
            .cols(starting_col)
            .zip(scoring_matrix.cols(left_of_starting_col))
            .zip(scoring_matrix.cols(up_left_of_starting_col));

        let mut is_in_gap = false;

        for (char_idx, candidate_char) in
            candidate.slice(matched_idx..last_matched_idx).char_idxs()
        {
            let ((cell, left_cell), up_left_cell) = cols.next().unwrap();

            let score_left =
                scoring_matrix[left_cell].saturating_sub(if is_in_gap {
                    penalty::GAP_EXTENSION
                } else {
                    penalty::GAP_START
                });

            let mut consecutive = 0;

            let score_up_left = if case_matcher.eq(query_char, candidate_char)
            {
                let score = scoring_matrix[up_left_cell] + bonus::MATCH;

                let mut bonus = bonus_vector[char_idx];

                consecutive = consecutive_matrix[up_left_cell] + 1;

                if consecutive > 1 {
                    let fb = bonus_vector[CandidateCharIdx(
                        char_idx.into_usize() + 1 - consecutive,
                    )];

                    if bonus >= bonus::BOUNDARY && bonus > fb {
                        consecutive = 1;
                    } else {
                        bonus = bonus::CONSECUTIVE.max(fb).max(bonus);
                    }
                }

                if score + bonus < score_left {
                    consecutive = 0;
                    score + bonus_vector[char_idx]
                } else {
                    score + bonus
                }
            } else {
                0
            };

            is_in_gap = score_up_left < score_left;

            let score = score_up_left.max(score_left).max(0);

            if score > max_score {
                max_score = score;
                max_score_cell = cell;
            }

            consecutive_matrix[cell] = consecutive;

            scoring_matrix[cell] = score;
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
) -> Vec<Range<usize>> {
    let mut ranges = Vec::<Range<usize>>::new();

    let mut prefer_match = true;

    let mut cell = max_score_cell;

    loop {
        let score = scores[cell];

        let cell_left = scores.left(cell);

        let cell_up_left = cell_left.and_then(|left| scores.up(left));

        let score_left = cell_left.map(|c| scores[c]).unwrap_or(0);

        let score_up_left = cell_up_left.map(|c| scores[c]).unwrap_or(0);

        let this_prefer_match = prefer_match;

        prefer_match = consecutives[cell] > 1
            || consecutives
                .right(cell)
                .and_then(|right| consecutives.down(right))
                .map(|down_right| consecutives[down_right] > 0)
                .unwrap_or(false);

        if score > score_up_left
            && (score > score_left || score == score_left && this_prefer_match)
        {
            let char_idx = CandidateCharIdx(scores.col_of(cell));

            let char = candidate.char(char_idx);
            let offset = candidate.char_offset(char_idx);

            let char_len_utf8 = char.len_utf8();

            match ranges.last_mut() {
                Some(last) if last.start == offset + char_len_utf8 => {
                    last.start = offset;
                },
                _ => {
                    ranges.push(offset..offset + char_len_utf8);
                },
            }

            if let Some(up_left) = cell_up_left {
                cell = up_left;
            } else {
                break;
            }
        } else if let Some(left) = scores.left(cell) {
            cell = left;
        } else {
            break;
        }
    }

    ranges
}
