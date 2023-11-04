use super::{query::*, scoring::*, slab::*, *};
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

        let mut total_score = 0;

        let mut matched_ranges = MatchedRanges::default();

        for condition in query.conditions() {
            let (score, ranges) =
                condition.or_patterns().find_map(|pattern| {
                    match pattern.match_type {
                        MatchType::Fuzzy => {
                            fzf_v2(self, pattern, candidate, false)
                        },
                        _ => todo!(),
                    }
                })?;

            total_score += score;

            if self.with_matched_ranges {
                matched_ranges.join(ranges);
            }
        }

        let distance = FzfDistance::from_score(total_score);

        Some(Match::new(distance, matched_ranges))
    }
}

/// TODO: docs
#[inline]
fn fzf_v2(
    fzf: &mut FzfV2,
    pattern: Pattern,
    candidate: &str,
    with_matched_ranges: bool,
) -> Option<(Score, MatchedRanges)> {
    let is_candidate_ascii = candidate.is_ascii();

    let is_case_sensitive = match fzf.case_sensitivity {
        CaseSensitivity::Sensitive => true,
        CaseSensitivity::Insensitive => false,
        CaseSensitivity::Smart => pattern.has_uppercase,
    };

    let (matches, last_match_offset) = matches(
        &mut fzf.slab.matched_indices,
        pattern,
        candidate,
        is_candidate_ascii,
        is_case_sensitive,
    )?;

    let first_match = matches[0];

    let initial_char_class = candidate[..first_match.byte_offset]
        .chars()
        .next_back()
        .map(|ch| char_class(ch, &fzf.scheme))
        .unwrap_or(fzf.scheme.initial_char_class);

    let candidate = &candidate[first_match.byte_offset..last_match_offset];

    // After slicing the candidate we need to move all the offsets back
    // by the offsets of the first match so that they still refer to the
    // characters.
    matches.iter_mut().for_each(|idx| *idx -= first_match);

    let bonus_vector = compute_bonuses(
        &mut fzf.slab.bonus_vector,
        candidate,
        initial_char_class,
        &fzf.scheme,
    );

    let (scores, consecutive, score, score_cell) = score(
        &mut fzf.slab.scoring_matrix,
        &mut fzf.slab.consecutive_matrix,
        pattern,
        candidate,
        is_candidate_ascii,
        is_case_sensitive,
        matches,
        bonus_vector,
    );

    let matched_ranges = if with_matched_ranges {
        let candidate = fzf.slab.candidate.alloc(candidate);
        let mut ranges =
            matched_ranges(scores, consecutive, score_cell, candidate);
        ranges.iter_mut().for_each(|range| {
            range.start += first_match.byte_offset;
            range.end += first_match.byte_offset;
        });
        ranges
    } else {
        MatchedRanges::default()
    };

    Some((score, matched_ranges))
}

/// TODO: docs
#[inline]
fn matches<'idx>(
    indices_slab: &'idx mut MatchedIndicesSlab,
    pattern: Pattern,
    mut candidate: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
) -> Option<(&'idx mut [MatchedIdx], usize)> {
    let matched_idxs = indices_slab.alloc(pattern.char_len());

    let mut query_char_idx = 0;

    let mut last_matched_idx = MatchedIdx::default();

    loop {
        let query_char = pattern.char(query_char_idx);

        let byte_offset =
            utils::find_first(query_char, candidate, is_case_sensitive)?;

        let char_offset = if is_candidate_ascii {
            byte_offset
        } else {
            utils::char_len(&candidate[..byte_offset])
        };

        last_matched_idx += MatchedIdx { byte_offset, char_offset };

        matched_idxs[query_char_idx] = last_matched_idx;

        let query_char_byte_len = query_char.len_utf8();

        // SAFETY: the start of the range is within the byte length of the
        // candidate and it's a valid char boundary.
        candidate = unsafe {
            candidate.get_unchecked(byte_offset + query_char_byte_len..)
        };

        if query_char_idx + 1 < pattern.char_len() {
            last_matched_idx += MatchedIdx {
                byte_offset: query_char_byte_len,
                char_offset: 1,
            };
            query_char_idx += 1;
        } else {
            break;
        }
    }

    let last_query_char = pattern.char(query_char_idx);

    let byte_offset =
        utils::find_last(last_query_char, candidate, is_case_sensitive)
            .unwrap_or(0);

    Some((
        matched_idxs,
        last_matched_idx.byte_offset
            + byte_offset
            + last_query_char.len_utf8(),
    ))
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

    for ch in candidate.chars() {
        let char_class = char_class(ch, scheme);
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
    pattern: Pattern,
    candidate: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    matches: &[MatchedIdx],
    bonus_vector: &[Score],
) -> (Matrix<'scoring, Score>, Matrix<'consecutive, usize>, Score, MatrixCell)
{
    // The length of the bonus slice is the same as the character length of the
    // candidate.
    let matrix_width = bonus_vector.len();

    let matrix_height = pattern.char_len();

    let mut scoring_matrix = scoring_slab.alloc(matrix_width, matrix_height);

    let mut consecutive_matrix =
        consecutive_slab.alloc(matrix_width, matrix_height);

    let (max_score, max_score_cell) = score_first_row(
        scoring_matrix.row_mut(0),
        consecutive_matrix.row_mut(0),
        bonus_vector,
        pattern.char(0),
        candidate,
        is_candidate_ascii,
        is_case_sensitive,
    );

    let (max_score, max_score_cell) = score_remaining_rows(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        pattern,
        matches,
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

    // TODO: explain what this does.
    let mut penalty = penalty::GAP_START;

    while !candidate.is_empty() {
        let Some(byte_idx) =
            utils::find_first(query_first_char, candidate, is_case_sensitive)
        else {
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
            utils::char_len(&candidate[..byte_idx])
        };

        // TODO: explain what this does.
        {
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
    pattern: Pattern,
    matches: &[MatchedIdx],
    candidate: &str,
    bonus_vector: &[Score],
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    mut max_score: Score,
    mut max_score_cell: MatrixCell,
) -> (Score, MatrixCell) {
    let matrix_width = scores.width();

    for row_idx in 1..scores.height() {
        let query_char = pattern.char(row_idx);

        let (prev_scores_row, scores_row) =
            scores.two_rows_mut(row_idx - 1, row_idx);

        let (prev_consecutives_row, consecutives_row) =
            consecutives.two_rows_mut(row_idx - 1, row_idx);

        let matched_idx = matches[row_idx];

        let mut column = matched_idx.char_offset;

        let mut candidate = &candidate[matched_idx.byte_offset..];

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
                utils::char_len(&candidate[..byte_offset])
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
) -> MatchedRanges {
    let mut ranges = MatchedRanges::default();

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
