use core::ops::Range;

use super::{slab::*, *};
use crate::*;

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

        let is_case_sensitive = match self.case_sensitivity {
            CaseSensitivity::Sensitive => true,
            CaseSensitivity::Insensitive => false,
            CaseSensitivity::Smart => query.has_uppercase(),
        };

        let (matched_indices, last_matched_idx) = matched_indices(
            &mut self.slab.matched_indices,
            query,
            candidate,
            is_case_sensitive,
        )?;

        let candidate = self.slab.candidate.alloc(candidate);

        let bonus_vector = compute_bonuses(
            &mut self.slab.bonus_vector,
            candidate,
            &self.scheme,
        );

        let first_matched_idx = matched_indices.first();

        let candidate = candidate.slice(first_matched_idx..last_matched_idx);

        let case_matcher = if is_case_sensitive {
            utils::case_sensitive_eq
        } else {
            utils::case_insensitive_eq
        };

        let (scores, consecutive, score, score_cell) = score(
            &mut self.slab.scoring_matrix,
            &mut self.slab.consecutive_matrix,
            query,
            candidate,
            matched_indices,
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
fn matched_indices<'idx>(
    indices_slab: &'idx mut MatchedIndicesSlab,
    query: FzfQuery,
    mut candidate: &str,
    is_case_sensitive: bool,
) -> Option<(MatchedIndices<'idx>, CandidateCharIdx)> {
    let candidate_is_ascii = candidate.is_ascii();

    let mut query_chars = query.chars();

    let mut query_char = query_chars.next()?;

    let mut matched_idxs = indices_slab.alloc(query);

    let mut char_offset = 0;

    let mut last_matched_idx;

    loop {
        let byte_offset =
            utils::find_first(query_char, candidate, is_case_sensitive)?;

        let char_idx = if candidate_is_ascii {
            byte_offset
        } else {
            candidate[..byte_offset].chars().count()
        };

        last_matched_idx = CandidateCharIdx(char_idx + char_offset);

        matched_idxs.push(last_matched_idx);

        candidate = &candidate[byte_offset + query_char.len_utf8()..];

        char_offset += char_idx + 1;

        if let Some(next_char) = query_chars.next() {
            query_char = next_char;
        } else {
            break;
        }
    }

    let byte_offset =
        utils::find_last(query_char, candidate, is_case_sensitive)
            .unwrap_or(0);

    last_matched_idx += CandidateCharIdx(if candidate_is_ascii {
        byte_offset
    } else {
        candidate[..byte_offset].chars().count()
    });

    Some((matched_idxs, last_matched_idx))
}

/// TODO: docs
#[inline]
fn compute_bonuses<'bonus>(
    bonus_slab: &'bonus mut BonusVectorSlab,
    candidate: Candidate,
    scheme: &Scheme,
) -> BonusVector<'bonus> {
    let mut prev_class = scheme.initial_char_class;

    let mut bonuses = bonus_slab.alloc(candidate);

    for (char_idx, candidate_char) in candidate.char_idxs() {
        let char_class = char_class(candidate_char, scheme);
        bonuses[char_idx] = bonus(prev_class, char_class, scheme);
        prev_class = char_class;
    }

    bonuses
}

/// TODO: docs
#[inline]
fn score<'scoring, 'consecutive>(
    scoring_slab: &'scoring mut ScoringMatrixSlab,
    consecutive_slab: &'consecutive mut ConsecutiveMatrixSlab,
    query: FzfQuery,
    candidate: Candidate,
    matched_indices: MatchedIndices,
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

    let (first_query_char, _, _) =
        chars_idxs_rows.next().expect("the query is not empty");

    let (max_score, max_score_cell) = score_first_row(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        first_query_char,
        candidate,
        &bonus_vector,
        case_matcher,
    );

    let (max_score, max_score_cell) = score_remaining_rows(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        chars_idxs_rows,
        max_score,
        max_score_cell,
        candidate,
        bonus_vector,
        case_matcher,
    );

    (scoring_matrix, consecutive_matrix, max_score, max_score_cell)
}

/// TODO: docs
#[inline]
fn score_first_row(
    scores: &mut Matrix<'_, Score>,
    consecutives: &mut Matrix<'_, usize>,
    first_query_char: char,
    candidate: Candidate,
    bonus_vector: &BonusVector,
    case_matcher: CaseMatcher,
) -> (Score, MatrixCell) {
    let mut max_score: Score = 0;

    let mut max_score_cell = scores.top_left();

    let mut prev_score: Score = 0;

    let mut is_in_gap = false;

    let mut cols = scores.cols(scores.top_left());

    for (char_idx, candidate_char) in candidate.char_idxs() {
        let cell = cols.next().expect(
            "the scoring matrix's width >= than the sliced candidate's char \
             length",
        );

        let bonus = bonus_vector[char_idx];

        let chars_match = case_matcher(first_query_char, candidate_char);

        consecutives[cell] = chars_match as usize;

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

        scores[cell] = score;

        prev_score = score;
    }

    (max_score, max_score_cell)
}

/// TODO: docs
#[inline]
fn score_remaining_rows<I>(
    scores: &mut Matrix<'_, Score>,
    consecutives: &mut Matrix<'_, usize>,
    chars_idxs_rows: I,
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
            let skipped_cols =
                matched_idx.into_usize() - candidate.first_idx().into_usize();
            scores.right_n(first_col_cell, skipped_cols)
        };

        // TODO: docs
        let left_of_starting_col = scores.left(starting_col);

        // TODO: docs
        let up_left_of_starting_col = scores.up(left_of_starting_col);

        // TODO: docs
        let mut cols = scores
            .cols(starting_col)
            .zip(scores.cols(left_of_starting_col))
            .zip(scores.cols(up_left_of_starting_col));

        let mut is_in_gap = false;

        for (char_idx, candidate_char) in
            candidate.slice_from(matched_idx).char_idxs()
        {
            let ((cell, left_cell), up_left_cell) = cols.next().unwrap();

            let score_left = scores[left_cell].saturating_sub(if is_in_gap {
                penalty::GAP_EXTENSION
            } else {
                penalty::GAP_START
            });

            let mut consecutive = 0;

            let score_up_left = if case_matcher(query_char, candidate_char) {
                let score = scores[up_left_cell] + bonus::MATCH;

                let mut bonus = bonus_vector[char_idx];

                consecutive = consecutives[up_left_cell] + 1;

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

            consecutives[cell] = consecutive;

            scores[cell] = score;
        }
    }

    (max_score, max_score_cell)
}

/// TODO: docs
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

        let cell_left = if scores.is_in_first_col(cell) {
            None
        } else {
            Some(scores.left(cell))
        };

        let cell_up_left =
            if scores.is_in_first_col(cell) || scores.is_in_first_row(cell) {
                None
            } else {
                Some(scores.up(scores.left(cell)))
            };

        let score_left = cell_left.map_or(0, |cell_left| scores[cell_left]);

        let score_up_left =
            cell_up_left.map_or(0, |cell_up_left| scores[cell_up_left]);

        let prefer_this_match = prefer_match;

        prefer_match = consecutives[cell] > 1
            || (!consecutives.is_in_last_col(cell)
                && !consecutives.is_in_last_row(cell)
                && {
                    let down_right =
                        consecutives.down(consecutives.right(cell));
                    consecutives[down_right] > 0
                });

        if score > score_up_left
            && (score > score_left || score == score_left && prefer_this_match)
        {
            let col = scores.col_of(cell);
            let char = candidate.nth_char(col);
            let offset = candidate.nth_char_offset(col);

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
        } else if let Some(left) = cell_left {
            cell = left;
        } else {
            break;
        }
    }

    ranges
}
