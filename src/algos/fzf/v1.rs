use core::ops::Range;

use super::{query::*, *};
use crate::*;

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(any(feature = "fzf-v1", feature = "fzf-v2"))))]
#[derive(Default)]
pub struct FzfV1 {
    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    scheme: Scheme,

    /// TODO: docs
    with_matched_ranges: bool,
}

impl FzfV1 {
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

impl Metric for FzfV1 {
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

        let pattern = query.conditions()[0].or_patterns().next().unwrap();

        let case_matcher =
            self.case_sensitivity.matcher(pattern.has_uppercase);

        let range_forward = forward_pass(pattern, candidate, case_matcher)?;

        let start_backward = backward_pass(
            pattern,
            &candidate[range_forward.clone()],
            case_matcher,
        );

        let range = range_forward.start + start_backward..range_forward.end;

        let (score, matched_ranges) = calculate_score(
            pattern,
            candidate,
            range,
            &self.scheme,
            case_matcher,
            self.with_matched_ranges,
        );

        let distance = FzfDistance::from_score(score);

        Some(Match::new(distance, matched_ranges))
    }
}

/// TODO: docs
#[inline]
fn forward_pass(
    pattern: Pattern,
    candidate: &str,
    case_matcher: CaseMatcher,
) -> Option<Range<usize>> {
    let mut start_offset = None;

    let mut end_offset = None;

    let mut pattern_chars = pattern.chars();

    let mut pattern_char = pattern_chars.next().expect("pattern is not empty");

    for (offset, candidate_char) in candidate.char_indices() {
        if !case_matcher(pattern_char, candidate_char) {
            continue;
        }

        if start_offset.is_none() {
            start_offset = Some(offset);
        }

        let Some(next_target_char) = pattern_chars.next() else {
            end_offset = Some(offset + candidate_char.len_utf8());
            break;
        };

        pattern_char = next_target_char;
    }

    let (Some(start), Some(end)) = (start_offset, end_offset) else {
        return None;
    };

    Some(start..end)
}

/// TODO: docs
#[inline]
fn backward_pass(
    pattern: Pattern,
    candidate: &str,
    case_matcher: CaseMatcher,
) -> usize {
    // The candidate must start with the first character of the query.
    debug_assert!(case_matcher(
        candidate.chars().next().unwrap(),
        pattern.chars().next().unwrap()
    ));

    // The candidate must end with the last character of the query.
    debug_assert!(case_matcher(
        candidate.chars().next_back().unwrap(),
        pattern.chars().next_back().unwrap()
    ));

    let mut start_offset = 0;

    let mut query_chars = pattern.chars().rev();

    let mut query_char = query_chars.next().expect("query is not empty");

    for (offset, candidate_char) in candidate.char_indices().rev() {
        if !case_matcher(query_char, candidate_char) {
            continue;
        }

        let Some(next_query_char) = query_chars.next() else {
            start_offset = offset;
            break;
        };

        query_char = next_query_char;
    }

    start_offset
}

/// TODO: docs
#[inline]
fn calculate_score(
    pattern: Pattern,
    candidate: &str,
    range: Range<usize>,
    scheme: &Scheme,
    case_matcher: CaseMatcher,
    track_matched_ranges: bool,
) -> (Score, MatchedRanges) {
    // TODO: docs
    let mut is_in_gap = false;

    // TODO: docs
    let mut is_first_pattern_char = true;

    // TODO: docs
    let mut first_bonus = 0u32;

    // TODO: docs
    let mut consecutive = 0u32;

    let range_start = range.start;

    let mut prev_class = candidate[..range.start]
        .chars()
        .next_back()
        .map(|ch| char_class(ch, scheme))
        .unwrap_or(scheme.initial_char_class);

    let mut pattern_chars = pattern.chars();

    let mut pattern_char = pattern_chars.next().expect("pattern is not empty");

    let mut score = 0u32;

    let mut matched_ranges = MatchedRanges::default();

    for (offset, candidate_ch) in candidate[range].char_indices() {
        let ch_class = char_class(candidate_ch, scheme);

        if case_matcher(pattern_char, candidate_ch) {
            score += bonus::MATCH;

            let mut bonus = bonus(prev_class, ch_class, scheme);

            if consecutive == 0 {
                first_bonus = bonus;
            } else {
                if bonus >= bonus::BOUNDARY && bonus > first_bonus {
                    first_bonus = bonus
                }
                bonus = bonus.max(first_bonus).max(bonus::CONSECUTIVE);
            }

            score += if is_first_pattern_char {
                bonus * bonus::FIRST_QUERY_CHAR_MULTIPLIER
            } else {
                bonus
            };

            if track_matched_ranges {
                if consecutive == 0 {
                    let start = range_start + offset;
                    let end = start + candidate_ch.len_utf8();
                    matched_ranges.push(start..end);
                } else if let Some(last_range) = matched_ranges.last_mut() {
                    last_range.end += candidate_ch.len_utf8();
                } else {
                    unreachable!(
                        "if consecutive is > 0 we've already pushed a range"
                    );
                }
            }

            is_in_gap = false;

            is_first_pattern_char = false;

            consecutive += 1;

            if let Some(next_char) = pattern_chars.next() {
                pattern_char = next_char;
            } else {
                break;
            };
        } else {
            score -= if is_in_gap {
                penalty::GAP_EXTENSION
            } else {
                penalty::GAP_START
            };

            is_in_gap = true;

            consecutive = 0;

            first_bonus = 0;
        }

        prev_class = ch_class;
    }

    (score, matched_ranges)
}
