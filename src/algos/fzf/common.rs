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
    char_eq: CharEq,
    with_matched_ranges: bool,
    matched_ranges: &mut MatchedRanges,
) -> Score {
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

    for (offset, candidate_ch) in candidate[range].char_indices() {
        let ch_class = char_class(candidate_ch, scheme);

        if char_eq(pattern_char, candidate_ch) {
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
                let start = range_start + offset;
                let end = start + candidate_ch.len_utf8();
                matched_ranges.insert(start..end);
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

    score
}

/// TODO: docs
#[inline]
pub(super) fn exact_match(
    pattern: Pattern,
    candidate: &str,
    scheme: &Scheme,
    char_eq: CharEq,
    with_matched_ranges: bool,
    matched_ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    // TODO: docs
    let mut best_bonus: i64 = -1;

    // TODO: docs
    let mut best_bonus_byte_offset = 0;

    // TODO: docs
    let mut matched = false;

    let mut prev_char_class = scheme.initial_char_class;

    let mut start_offset = 0;

    'outer: loop {
        let current_start_offset = start_offset;
        let candidate = &candidate[start_offset..];
        let mut current_bonus: Score = 0;
        let mut pattern_char_idx = 0;

        let mut char_indices = candidate.char_indices();

        for (byte_offset, candidate_ch) in char_indices.by_ref() {
            let pattern_ch = pattern.char(pattern_char_idx);

            let char_class = char_class(candidate_ch, scheme);

            if char_eq(pattern_ch, candidate_ch) {
                if pattern_char_idx == 0 {
                    start_offset += byte_offset + candidate_ch.len_utf8();
                    current_bonus = bonus(prev_char_class, char_class, scheme);
                }

                pattern_char_idx += 1;

                if pattern_char_idx == pattern.char_len() {
                    matched = true;

                    if current_bonus as i64 > best_bonus {
                        best_bonus = current_bonus as _;

                        best_bonus_byte_offset = current_start_offset
                            + byte_offset
                            + candidate_ch.len_utf8();
                    }

                    if current_bonus >= bonus::BOUNDARY {
                        break 'outer;
                    }

                    break;
                }
            } else if pattern_char_idx > 0 {
                break;
            }

            prev_char_class = char_class;
        }

        if char_indices.next().is_none() {
            break;
        }
    }

    if !matched {
        return None;
    }

    let matched_range = {
        let end = best_bonus_byte_offset;
        end - pattern.byte_len..end
    };

    let score = calculate_score(
        pattern,
        candidate,
        matched_range.clone(),
        scheme,
        char_eq,
        false,
        matched_ranges,
    );

    if with_matched_ranges {
        matched_ranges.insert(matched_range);
    }

    Some(score)
}

/// TODO: docs
#[inline]
pub(super) fn prefix_match(
    pattern: Pattern,
    candidate: &str,
    scheme: &Scheme,
    char_eq: CharEq,
    with_matched_ranges: bool,
    matched_ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    let mut pattern_chars = pattern.chars();

    let ignored_leading_spaces =
        ignored_candidate_leading_spaces(pattern, candidate)?;

    for (candidate_ch, pattern_ch) in
        candidate[ignored_leading_spaces..].chars().zip(pattern_chars.by_ref())
    {
        if !char_eq(pattern_ch, candidate_ch) {
            return None;
        }
    }

    if pattern_chars.next().is_some() {
        return None;
    }

    let matched_range =
        ignored_leading_spaces..ignored_leading_spaces + pattern.byte_len;

    let score = calculate_score(
        pattern,
        candidate,
        matched_range.clone(),
        scheme,
        char_eq,
        false,
        matched_ranges,
    );

    if with_matched_ranges {
        matched_ranges.insert(matched_range);
    }

    Some(score)
}

/// TODO: docs
#[inline]
pub(super) fn suffix_match(
    pattern: Pattern,
    candidate: &str,
    scheme: &Scheme,
    char_eq: CharEq,
    with_matched_ranges: bool,
    matched_ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    let mut pattern_chars = pattern.chars().rev();

    let up_to_ignored_spaces = candidate.len()
        - ignored_candidate_trailing_spaces(pattern, candidate)?;

    for (candidate_ch, pattern_ch) in candidate[..up_to_ignored_spaces]
        .chars()
        .rev()
        .zip(pattern_chars.by_ref())
    {
        if !char_eq(pattern_ch, candidate_ch) {
            return None;
        }
    }

    if pattern_chars.next().is_some() {
        return None;
    }

    let matched_range =
        up_to_ignored_spaces - pattern.byte_len..up_to_ignored_spaces;

    let score = calculate_score(
        pattern,
        candidate,
        matched_range.clone(),
        scheme,
        char_eq,
        false,
        matched_ranges,
    );

    if with_matched_ranges {
        matched_ranges.insert(matched_range);
    }

    Some(score)
}

/// TODO: docs
#[inline]
pub(super) fn equal_match(
    pattern: Pattern,
    candidate: &str,
    scheme: &Scheme,
    char_eq: CharEq,
    with_matched_ranges: bool,
    matched_ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    let ignored_leading_spaces =
        ignored_candidate_leading_spaces(pattern, candidate)?;

    // The candidate contains only spaces.
    if ignored_leading_spaces == candidate.len() {
        return None;
    }

    let ignored_trailing_spaces =
        ignored_candidate_trailing_spaces(pattern, candidate)?;

    let matched_range =
        ignored_leading_spaces..candidate.len() - ignored_trailing_spaces;

    let relevant_candidate = &candidate[matched_range.clone()];

    if relevant_candidate.len() < pattern.char_len() {
        return None;
    }

    let mut pattern_chars = pattern.chars();

    let mut candidate_chars = relevant_candidate.chars();

    for (pattern_ch, candidate_ch) in
        pattern_chars.by_ref().zip(candidate_chars.by_ref())
    {
        if !char_eq(pattern_ch, candidate_ch) {
            return None;
        }
    }

    if pattern_chars.next().is_some() || candidate_chars.next().is_some() {
        return None;
    }

    let score = calculate_score(
        pattern,
        candidate,
        matched_range.clone(),
        scheme,
        char_eq,
        false,
        matched_ranges,
    );

    if with_matched_ranges {
        matched_ranges.insert(matched_range);
    }

    Some(score)
}

/// TODO: docs
#[inline(always)]
fn ignored_candidate_leading_spaces(
    pattern: Pattern,
    candidate: &str,
) -> Option<usize> {
    let candidate_leading_spaces = utils::leading_spaces(candidate);

    if pattern.leading_spaces() > candidate_leading_spaces {
        None
    } else {
        Some(candidate_leading_spaces - pattern.leading_spaces())
    }
}

/// TODO: docs
#[inline(always)]
fn ignored_candidate_trailing_spaces(
    pattern: Pattern,
    candidate: &str,
) -> Option<usize> {
    let candidate_trailing_spaces = utils::trailing_spaces(candidate);

    if pattern.trailing_spaces() > candidate_trailing_spaces {
        None
    } else {
        Some(candidate_trailing_spaces - pattern.trailing_spaces())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::single_range_in_vec_init)]

    use super::*;

    #[test]
    fn equal_match_1() {
        let pattern =
            Pattern::parse("^AbC$".chars().collect::<Vec<_>>().leak());

        let mut matched_ranges = MatchedRanges::default();

        assert!(exact_match(
            pattern,
            "ABC",
            &Scheme::default(),
            utils::char_eq(true, false),
            true,
            &mut matched_ranges
        )
        .is_none());

        {
            matched_ranges = MatchedRanges::default();

            assert!(exact_match(
                pattern,
                "AbC",
                &Scheme::default(),
                utils::char_eq(true, false),
                true,
                &mut matched_ranges
            )
            .is_some());

            assert_eq!(matched_ranges.as_slice(), [0..3]);
        }

        {
            matched_ranges = MatchedRanges::default();

            assert!(exact_match(
                pattern,
                "AbC ",
                &Scheme::default(),
                utils::char_eq(true, false),
                true,
                &mut matched_ranges
            )
            .is_some());

            assert_eq!(matched_ranges.as_slice(), [0..3]);
        }

        {
            matched_ranges = MatchedRanges::default();

            assert!(exact_match(
                pattern,
                " AbC ",
                &Scheme::default(),
                utils::char_eq(true, false),
                true,
                &mut matched_ranges
            )
            .is_some());

            assert_eq!(matched_ranges.as_slice(), [1..4]);
        }

        {
            matched_ranges = MatchedRanges::default();

            assert!(exact_match(
                pattern,
                "  AbC",
                &Scheme::default(),
                utils::char_eq(true, false),
                true,
                &mut matched_ranges
            )
            .is_some());

            assert_eq!(matched_ranges.as_slice(), [2..5]);
        }
    }

    #[test]
    fn exact_match_1() {
        let pattern = Pattern::parse("abc".chars().collect::<Vec<_>>().leak());

        let mut matched_ranges = MatchedRanges::default();

        assert!(exact_match(
            pattern,
            "aabbcc abc",
            &Scheme::default(),
            utils::char_eq(true, false),
            true,
            &mut matched_ranges
        )
        .is_some());

        assert_eq!(matched_ranges.as_slice(), [7..10]);
    }
}
