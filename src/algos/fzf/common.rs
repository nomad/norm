use core::ops::Range;

use super::{query::*, *};
use crate::*;

/// TODO: docs
#[inline]
pub(super) fn calculate_score(
    pattern: Pattern,
    candidate: &str,
    range: Range<usize>,
    scheme: &Scheme,
    case_matcher: CaseMatcher,
    with_matched_ranges: bool,
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

    let mut score: Score = 0;

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

            if with_matched_ranges {
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

/// TODO: docs
#[inline]
pub(super) fn prefix_match(
    pattern: Pattern,
    candidate: &str,
    scheme: &Scheme,
    is_case_sensitive: bool,
    with_matched_ranges: bool,
) -> Option<(Score, MatchedRanges)> {
    let char_eq = utils::char_eq(is_case_sensitive);

    let mut pattern_chars = pattern.chars();

    let ignored_whitespaces = if pattern.char(0).is_ascii_whitespace() {
        0
    } else {
        utils::leading_spaces(candidate)
    };

    for (pattern_ch, candidate_ch) in
        pattern_chars.by_ref().zip(candidate[ignored_whitespaces..].chars())
    {
        if !char_eq(pattern_ch, candidate_ch) {
            break;
        }
    }

    if pattern_chars.next().is_some() {
        return None;
    }

    let matched_range =
        ignored_whitespaces..ignored_whitespaces + pattern.byte_len;

    let (score, _) = calculate_score(
        pattern,
        candidate,
        matched_range.clone(),
        scheme,
        char_eq,
        false,
    );

    let mut ranges = MatchedRanges::default();

    if with_matched_ranges {
        ranges.push(matched_range);
    }

    Some((score, ranges))
}

/// TODO: docs
#[inline]
pub(super) fn suffix_match(
    pattern: Pattern,
    candidate: &str,
    scheme: &Scheme,
    is_case_sensitive: bool,
    with_matched_ranges: bool,
) -> Option<(Score, MatchedRanges)> {
    let char_eq = utils::char_eq(is_case_sensitive);

    let mut pattern_chars = pattern.chars().rev();

    let up_to_ignored_whitespaces = candidate.len()
        - if pattern.last_char().is_ascii_whitespace() {
            0
        } else {
            utils::trailing_spaces(candidate)
        };

    for (pattern_ch, candidate_ch) in pattern_chars
        .by_ref()
        .zip(candidate[..up_to_ignored_whitespaces].chars())
    {
        if !char_eq(pattern_ch, candidate_ch) {
            break;
        }
    }

    if pattern_chars.next().is_some() {
        return None;
    }

    let matched_range = up_to_ignored_whitespaces - pattern.byte_len
        ..up_to_ignored_whitespaces;

    let (score, _) = calculate_score(
        pattern,
        candidate,
        matched_range.clone(),
        scheme,
        char_eq,
        false,
    );

    let mut ranges = MatchedRanges::default();

    if with_matched_ranges {
        ranges.push(matched_range);
    }

    Some((score, ranges))
}
