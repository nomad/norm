use core::ops::Range;

use super::{slab::*, *};
use crate::*;

/// TODO: docs
type CandidateCharIdx = usize;

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

        let is_candidate_ascii = candidate.is_ascii();

        let is_case_sensitive = match self.case_sensitivity {
            CaseSensitivity::Sensitive => true,
            CaseSensitivity::Insensitive => false,
            CaseSensitivity::Smart => query.has_uppercase(),
        };

        let (matched_indices, last_matched_idx) = matched_indices(
            &mut self.slab.matched_indices,
            query,
            candidate,
            is_candidate_ascii,
            is_case_sensitive,
        )?;

        let first_matched_idx = matched_indices[0];

        let initial_char_class = candidate[..first_matched_idx]
            .chars()
            .next_back()
            .map(|ch| char_class(ch, &self.scheme))
            .unwrap_or(self.scheme.initial_char_class);

        // TODO: we're slicing with a char range.
        let candidate = &candidate[first_matched_idx..last_matched_idx + 1];

        // After slicing the candidate we need to move all the indices back by
        // `first_matched_idx` so that they still refer to the characters.
        matched_indices.iter_mut().for_each(|idx| *idx -= first_matched_idx);

        let bonus_vector = compute_bonuses(
            &mut self.slab.bonus_vector,
            candidate,
            initial_char_class,
            &self.scheme,
        );

        let (scores, consecutive, score, score_cell) = score(
            &mut self.slab.scoring_matrix,
            &mut self.slab.consecutive_matrix,
            query,
            candidate,
            is_candidate_ascii,
            is_case_sensitive,
            matched_indices,
            bonus_vector,
        );

        let matched_ranges = if self.with_matched_ranges {
            let candidate = self.slab.candidate.alloc(candidate);
            let mut ranges =
                matched_ranges(scores, consecutive, score_cell, candidate);
            ranges.iter_mut().for_each(|range| {
                range.start += first_matched_idx;
                range.end += first_matched_idx;
            });
            ranges
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
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
) -> Option<(&'idx mut [CandidateCharIdx], CandidateCharIdx)> {
    let mut query_chars = query.chars();

    let mut query_char = query_chars.next()?;

    let mut matched_idxs = indices_slab.alloc(query);

    let mut char_offset = 0;

    let mut last_matched_idx;

    loop {
        let byte_offset =
            utils::find_first(query_char, candidate, is_case_sensitive)?;

        let char_idx = if is_candidate_ascii {
            byte_offset
        } else {
            candidate[..byte_offset].chars().count()
        };

        last_matched_idx = char_idx + char_offset;

        matched_idxs.push(last_matched_idx);

        // SAFETY: the start of the range is within the byte length of the
        // candidate and it's a valid char boundary.
        candidate = unsafe {
            candidate.get_unchecked(byte_offset + query_char.len_utf8()..)
        };

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

    last_matched_idx += if is_candidate_ascii {
        byte_offset
    } else {
        candidate[..byte_offset].chars().count()
    };

    Some((matched_idxs.into_slice(), last_matched_idx))
}

/// TODO: docs
#[inline]
fn compute_bonuses<'bonus>(
    bonus_slab: &'bonus mut BonusVectorSlab,
    candidate: &str,
    initial_char_class: CharClass,
    scheme: &Scheme,
) -> &'bonus [Score] {
    let mut prev_class = initial_char_class;

    let mut bonuses = bonus_slab.alloc(candidate);

    for char in candidate.chars() {
        let char_class = char_class(char, scheme);
        bonuses.push(bonus(prev_class, char_class, scheme));
        prev_class = char_class;
    }

    bonuses.into_slice()
}

/// TODO: docs
#[inline]
fn score<'scoring, 'consecutive>(
    scoring_slab: &'scoring mut MatrixSlab<Score>,
    consecutive_slab: &'consecutive mut MatrixSlab<usize>,
    query: FzfQuery,
    candidate: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    matched_indices: &[CandidateCharIdx],
    bonus_vector: &[Score],
) -> (Matrix<'scoring, Score>, Matrix<'consecutive, usize>, Score, MatrixCell)
{
    // The length of the bonus slice is the same as the character length of the
    // candidate.
    let matrix_width = bonus_vector.len();

    let matrix_height = query.char_len();

    let mut scoring_matrix = scoring_slab.alloc(matrix_width, matrix_height);

    let mut consecutive_matrix =
        consecutive_slab.alloc(matrix_width, matrix_height);

    let (max_score, max_score_cell) = score_first_row(
        scoring_matrix.row_mut(0),
        consecutive_matrix.row_mut(0),
        bonus_vector,
        query.char(0),
        candidate,
        is_candidate_ascii,
        is_case_sensitive,
    );

    let (max_score, max_score_cell) = score_remaining_rows(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        query,
        matched_indices,
        candidate,
        bonus_vector,
        is_candidate_ascii,
        is_case_sensitive,
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
    bonus_vector: &[Score],
    query_first_char: char,
    mut candidate: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
) -> (Score, MatrixCell) {
    let mut max_score: Score = 0;

    let mut prev_score: Score = 0;

    let mut max_score_col: usize = 0;

    // TODO: docs
    let mut col = 0;

    let char_len = query_first_char.len_utf8();

    while !candidate.is_empty() {
        let Some(byte_idx) =
            utils::find_first(query_first_char, candidate, is_case_sensitive)
        else {
            // TODO: explain what this does.
            let mut penalty = penalty::GAP_START;

            for col in col..scores_first_row.len() {
                let score = prev_score.saturating_sub(penalty);
                penalty = penalty::GAP_EXTENSION;
                scores_first_row[col] = score;
                prev_score = score;
            }

            break;
        };

        let char_idx = if is_candidate_ascii {
            byte_idx
        } else {
            candidate[..byte_idx].chars().count()
        };

        // TODO: explain what this does.
        {
            let mut penalty = penalty::GAP_START;

            for col in col..col + char_idx {
                let score = prev_score.saturating_sub(penalty);
                penalty = penalty::GAP_EXTENSION;
                scores_first_row[col] = score;
                prev_score = score;
            }
        }

        col += char_idx;

        consecutives_first_row[col] = 1;

        let score = bonus::MATCH
            + (bonus_vector[col] * bonus::FIRST_QUERY_CHAR_MULTIPLIER);

        if score > max_score {
            max_score = score;
            max_score_col = col;
        }

        scores_first_row[col] = score;

        prev_score = score;

        col += 1;

        candidate = &candidate[byte_idx + char_len..];
    }

    (max_score, MatrixCell(max_score_col))
}

/// TODO: docs
#[inline]
fn score_remaining_rows(
    scores: &mut Matrix<'_, Score>,
    consecutives: &mut Matrix<'_, usize>,
    query: FzfQuery,
    matched_offsets: &[usize],
    candidate: &str,
    bonus_vector: &[Score],
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    mut max_score: Score,
    mut max_score_cell: MatrixCell,
) -> (Score, MatrixCell) {
    let matrix_width = scores.width();

    for row_idx in 1..scores.height() {
        let query_char = query.char(row_idx);

        let (prev_scores_row, scores_row) =
            scores.two_rows_mut(row_idx - 1, row_idx);

        let (prev_consecutives_row, consecutives_row) =
            consecutives.two_rows_mut(row_idx - 1, row_idx);

        let matched_offset = matched_offsets[row_idx];

        let mut column = matched_offset;

        // TODO: matched_offset is a char offset, not a byte offset.
        let mut candidate = &candidate[matched_offset..];

        // TODO: explain what this does.
        let mut penalty = penalty::GAP_START;

        while !candidate.is_empty() {
            let Some(byte_offset) =
                utils::find_first(query_char, candidate, is_case_sensitive)
            else {
                for col in column..matrix_width {
                    let score_left = scores_row[col - 1];
                    let score = score_left.saturating_sub(penalty);
                    penalty = penalty::GAP_EXTENSION;
                    scores_row[col] = score;
                }

                break;
            };

            let char_offset = if is_candidate_ascii {
                byte_offset
            } else {
                candidate[..byte_offset].chars().count()
            };

            // TODO: explain what this does.
            penalty = penalty::GAP_START;

            {
                for col in column..column + char_offset {
                    let score_left = scores_row[col - 1];
                    let score = score_left.saturating_sub(penalty);
                    penalty = penalty::GAP_EXTENSION;
                    scores_row[col] = score;
                }
            }

            column += char_offset;

            // TODO: explain what this does.
            {
                let score_left =
                    scores_row[column - 1].saturating_sub(penalty);

                let mut score_up_left =
                    prev_scores_row[column - 1] + bonus::MATCH;

                let mut bonus = bonus_vector[column];

                let mut consecutive = prev_consecutives_row[column - 1] + 1;

                if consecutive > 1 {
                    let fb = bonus_vector[column + 1 - consecutive];

                    if bonus >= bonus::BOUNDARY && bonus > fb {
                        consecutive = 1;
                    } else {
                        bonus = bonus::CONSECUTIVE.max(fb).max(bonus);
                    }
                }

                score_up_left += if score_up_left + bonus < score_left {
                    consecutive = 0;
                    bonus_vector[column]
                } else {
                    bonus
                };

                let score = score_left.max(score_up_left);

                if score > max_score {
                    max_score = score;
                    max_score_cell =
                        MatrixCell(row_idx * matrix_width + column);
                }

                consecutives_row[column] = consecutive;

                scores_row[column] = score;
            }

            column += 1;

            candidate = &candidate[byte_offset + query_char.len_utf8()..];
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

            let offset = candidate.nth_char_offset(col);

            let char_len_utf8 = candidate.nth_char_offset(col + 1) - offset;

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
