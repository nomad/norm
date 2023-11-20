use core::ops::Range;

use super::{query::*, scoring::*, slab::*, *};
use crate::Opts;
use crate::*;

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Clone, Default)]
pub struct FzfV2 {
    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    normalization: bool,

    /// TODO: docs
    scheme: Scheme,

    /// TODO: docs
    slab: V2Slab,

    /// TODO: docs
    with_matched_ranges: bool,
}

impl core::fmt::Debug for FzfV2 {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FzfV2")
            .field("case_sensitivity", &self.case_sensitivity)
            .field("matched_ranges", &self.with_matched_ranges)
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
    fn score(
        &mut self,
        pattern: Pattern,
        candidate: &str,
        is_candidate_ascii: bool,
        buf: Option<&mut MatchedRanges>,
    ) -> Option<Score> {
        let is_sensitive = match self.case_sensitivity {
            CaseSensitivity::Sensitive => true,
            CaseSensitivity::Insensitive => false,
            CaseSensitivity::Smart => pattern.has_uppercase,
        };

        if is_candidate_ascii {
            fzf_v2(
                pattern,
                candidate,
                AsciiCandidateOpts::new(is_sensitive),
                &self.scheme,
                buf,
                &mut self.slab,
            )
        } else {
            fzf_v2(
                pattern,
                candidate,
                UnicodeCandidateOpts::new(is_sensitive, self.normalization),
                &self.scheme,
                buf,
                &mut self.slab,
            )
        }
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
    pub fn with_matched_ranges(&mut self, matched_ranges: bool) -> &mut Self {
        self.with_matched_ranges = matched_ranges;
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
    ) -> Option<Match<Self::Distance>> {
        if query.is_empty() {
            return Some(Match::default());
        }

        let is_candidate_ascii = candidate.is_ascii();

        let mut buf = if self.with_matched_ranges {
            Some(MatchedRanges::default())
        } else {
            None
        };

        let conditions = match query.search_mode {
            SearchMode::Extended(conditions) => conditions,

            SearchMode::NotExtended(pattern) => {
                return self
                    .score(
                        pattern,
                        candidate,
                        is_candidate_ascii,
                        buf.as_mut(),
                    )
                    .map(FzfDistance::from_score)
                    .map(|distance| {
                        Match::new(distance, buf.unwrap_or_default())
                    })
            },
        };

        let mut total_score = 0;

        for condition in conditions {
            let score = condition.iter().find_map(|pattern| {
                let is_sensitive = match self.case_sensitivity {
                    CaseSensitivity::Sensitive => true,
                    CaseSensitivity::Insensitive => false,
                    CaseSensitivity::Smart => pattern.has_uppercase,
                };

                if is_candidate_ascii {
                    pattern.score(
                        candidate,
                        AsciiCandidateOpts::new(is_sensitive),
                        &self.scheme,
                        buf.as_mut(),
                        &mut self.slab,
                        fzf_v2,
                    )
                } else {
                    pattern.score(
                        candidate,
                        UnicodeCandidateOpts::new(
                            is_sensitive,
                            self.normalization,
                        ),
                        &self.scheme,
                        buf.as_mut(),
                        &mut self.slab,
                        fzf_v2,
                    )
                }
            })?;

            total_score += score;
        }

        let distance = FzfDistance::from_score(total_score);

        Some(Match::new(distance, buf.unwrap_or_default()))
    }

    #[inline]
    fn distance_and_ranges(
        &mut self,
        _query: FzfQuery<'_>,
        _candidate: &str,
        _ranges_buf: &mut Vec<Range<usize>>,
    ) -> Option<Self::Distance> {
        todo!();
    }
}

/// TODO: docs
#[inline]
pub(super) fn fzf_v2(
    pattern: Pattern,
    candidate: &str,
    opts: impl Opts,
    scheme: &Scheme,
    ranges_buf: Option<&mut MatchedRanges>,
    slab: &mut V2Slab,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    let (matches, last_match_offset) =
        matches(&mut slab.matched_indices, pattern, candidate, opts)?;

    let first_match = matches[0];

    let initial_char_class = candidate[..first_match.byte_offset]
        .chars()
        .next_back()
        .map(|ch| char_class(ch, scheme))
        .unwrap_or(scheme.initial_char_class);

    let candidate = &candidate[first_match.byte_offset..last_match_offset];

    // After slicing the candidate we need to move all the offsets back
    // by the offsets of the first match so that they still refer to the
    // characters.
    matches.iter_mut().for_each(|mach| *mach -= first_match);

    let bonus_vector = compute_bonuses(
        &mut slab.bonus_vector,
        candidate,
        initial_char_class,
        scheme,
    );

    let (scores, consecutive, score, score_cell) = score(
        &mut slab.scoring_matrix,
        &mut slab.consecutive_matrix,
        pattern,
        candidate,
        matches,
        bonus_vector,
        opts,
    );

    if let Some(buf) = ranges_buf {
        matched_ranges(
            scores,
            consecutive,
            score_cell,
            candidate,
            first_match.byte_offset,
            buf,
        );
    };

    Some(score)
}

/// TODO: docs
#[inline]
fn matches<'idx>(
    indices_slab: &'idx mut MatchedIndicesSlab,
    pattern: Pattern,
    mut candidate: &str,
    opts: impl Opts,
) -> Option<(&'idx mut [MatchedIdx], usize)> {
    let matched_idxs = indices_slab.alloc(pattern.char_len());

    let mut pattern_char_idx = 0;

    let mut last_matched_idx = MatchedIdx::default();

    loop {
        let pattern_char = pattern.char(pattern_char_idx);

        let (byte_offset, matched_char_byte_len) =
            opts.find_first(pattern_char, candidate)?;

        let char_offset = opts.to_char_offset(candidate, byte_offset);

        last_matched_idx += MatchedIdx { byte_offset, char_offset };

        matched_idxs[pattern_char_idx] = last_matched_idx;

        // SAFETY: the start of the range is within the byte length of the
        // candidate and it's a valid char boundary.
        candidate = unsafe {
            candidate.get_unchecked(byte_offset + matched_char_byte_len..)
        };

        last_matched_idx +=
            MatchedIdx { byte_offset: matched_char_byte_len, char_offset: 1 };

        if pattern_char_idx + 1 < pattern.char_len() {
            pattern_char_idx += 1;
        } else {
            break;
        }
    }

    let last_char_offset_inclusive = last_matched_idx.byte_offset
        + if let Some((byte_offset, matched_char_byte_len)) =
            opts.find_last(pattern.char(pattern_char_idx), candidate)
        {
            byte_offset + matched_char_byte_len
        } else {
            0
        };

    Some((matched_idxs, last_char_offset_inclusive))
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
    matches: &[MatchedIdx],
    bonus_vector: &[Score],
    opts: impl Opts,
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
        opts,
    );

    let (max_score, max_score_cell) = score_remaining_rows(
        &mut scoring_matrix,
        &mut consecutive_matrix,
        pattern,
        matches,
        candidate,
        bonus_vector,
        opts,
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
    first_pattern_char: char,
    mut candidate: &str,
    opts: impl Opts,
) -> (Score, MatrixCell) {
    let mut max_score: Score = 0;

    let mut prev_score: Score = 0;

    let mut max_score_col: usize = 0;

    // TODO: docs
    let mut col = 0;

    // TODO: explain what this does.
    let mut penalty = penalty::GAP_START;

    while !candidate.is_empty() {
        let Some((byte_offset, matched_char_byte_len)) =
            opts.find_first(first_pattern_char, candidate)
        else {
            for col in col..scores_first_row.len() {
                let score = prev_score.saturating_sub(penalty);
                scores_first_row[col] = score;
                prev_score = score;
                penalty = penalty::GAP_EXTENSION;
            }

            break;
        };

        let char_offset = opts.to_char_offset(candidate, byte_offset);

        // TODO: explain what this does.
        {
            for col in col..col + char_offset {
                let score = prev_score.saturating_sub(penalty);
                scores_first_row[col] = score;
                prev_score = score;
                penalty = penalty::GAP_EXTENSION;
            }
        }

        col += char_offset;

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

        candidate = &candidate[byte_offset + matched_char_byte_len..];
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
    opts: impl Opts,
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

        let matched_idx = matches[row_idx];

        let mut column = matched_idx.char_offset;

        let mut candidate = &candidate[matched_idx.byte_offset..];

        // TODO: explain what this does.
        let mut penalty = penalty::GAP_START;

        while !candidate.is_empty() {
            let Some((byte_offset, matched_char_byte_len)) =
                opts.find_first(pattern_char, candidate)
            else {
                for col in column..matrix_width {
                    let score_left = scores_row[col - 1];
                    let score = score_left.saturating_sub(penalty);
                    scores_row[col] = score;
                    penalty = penalty::GAP_EXTENSION;
                }

                break;
            };

            let char_offset = opts.to_char_offset(candidate, byte_offset);

            // TODO: explain what this does.
            penalty = penalty::GAP_START;

            {
                for col in column..column + char_offset {
                    let score_left = scores_row[col - 1];
                    let score = score_left.saturating_sub(penalty);
                    scores_row[col] = score;
                    penalty = penalty::GAP_EXTENSION;
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

            candidate = &candidate[byte_offset + matched_char_byte_len..];
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
    candidate: &str,
    start_offset: usize,
    ranges: &mut MatchedRanges,
) {
    let mut prefer_match = true;

    let mut cell = max_score_cell;

    let mut char_indices = candidate.char_indices().rev().enumerate();

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

            let (mut offset, ch) = char_indices
                .by_ref()
                .find_map(|(back_idx, ch)| {
                    let idx = scores.width() - back_idx - 1;
                    (idx == col).then_some(ch)
                })
                .unwrap();

            offset += start_offset;

            ranges.insert(offset..offset + ch.len_utf8());

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
}
