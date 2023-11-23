use core::ops::Range;

use super::{query::*, *};
use crate::utils::CharEq;
use crate::*;

/// TODO: docs
pub(super) trait Fzf {
    /// TODO: docs
    fn alloc_chars<'a>(&mut self, candidate: &str) -> &'a [char];

    /// TODO: docs
    fn char_eq(&self, pattern: Pattern) -> CharEq;

    /// TODO: docs
    fn scheme(&self) -> &Scheme;

    /// TODO: docs
    fn fuzzy<const RANGES: bool>(
        &mut self,
        pattern: Pattern,
        candidate: Candidate,
        ranges: &mut MatchedRanges,
    ) -> Option<Score>;

    /// TODO: docs
    fn score<const RANGES: bool>(
        &mut self,
        pattern: Pattern,
        candidate: Candidate,
        ranges: &mut MatchedRanges,
    ) -> Option<Score> {
        let score = match pattern.match_type {
            MatchType::Fuzzy => {
                if pattern.is_inverse {
                    self.fuzzy::<false>(pattern, candidate, ranges)
                } else {
                    self.fuzzy::<RANGES>(pattern, candidate, ranges)
                }
            },

            MatchType::Exact => {
                let char_eq = self.char_eq(pattern);

                if pattern.is_inverse {
                    exact_match::<false>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                } else {
                    exact_match::<RANGES>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                }
            },

            MatchType::PrefixExact => {
                let char_eq = self.char_eq(pattern);

                if pattern.is_inverse {
                    prefix_match::<false>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                } else {
                    prefix_match::<RANGES>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                }
            },

            MatchType::SuffixExact => {
                let char_eq = self.char_eq(pattern);

                if pattern.is_inverse {
                    suffix_match::<false>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                } else {
                    suffix_match::<RANGES>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                }
            },

            MatchType::EqualExact => {
                let char_eq = self.char_eq(pattern);

                if pattern.is_inverse {
                    equal_match::<false>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                } else {
                    equal_match::<RANGES>(
                        pattern,
                        candidate,
                        char_eq,
                        self.scheme(),
                        ranges,
                    )
                }
            },
        };

        match (score.is_some(), pattern.is_inverse) {
            (true, false) => score,
            (false, true) => Some(0),
            _ => None,
        }
    }

    /// TODO: docs
    #[inline(always)]
    fn distance<const RANGES: bool>(
        &mut self,
        query: FzfQuery,
        candidate: &str,
        ranges: &mut MatchedRanges,
    ) -> Option<FzfDistance> {
        if query.is_empty() {
            return Some(FzfDistance::from_score(0));
        }

        let candidate = if candidate.is_ascii() {
            Candidate::Ascii(candidate.as_bytes())
        } else {
            Candidate::Unicode(self.alloc_chars(candidate))
        };

        match query.search_mode {
            SearchMode::NotExtended(pattern) => self
                .fuzzy::<RANGES>(pattern, candidate, ranges)
                .map(FzfDistance::from_score),

            SearchMode::Extended(conditions) => {
                let mut total_score: Score = 0;
                for condition in conditions {
                    total_score += condition.iter().find_map(|pattern| {
                        self.score::<RANGES>(pattern, candidate, ranges)
                    })?;
                }
                Some(FzfDistance::from_score(total_score))
            },
        }
    }
}

/// TODO: docs
#[inline]
pub(super) fn exact_match<const RANGES: bool>(
    pattern: Pattern,
    candidate: Candidate,
    char_eq: CharEq,
    scheme: &Scheme,
    ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    // TODO: docs
    let mut best_bonus: i64 = -1;

    // TODO: docs
    let mut best_bonus_start = 0;

    // TODO: docs
    let mut best_bonus_end = 0;

    // TODO: docs
    let mut matched = false;

    let mut prev_class = scheme.initial_char_class;

    let mut start_offset = 0;

    'outer: loop {
        let current_start_offset = start_offset;
        let mut bonus_start = 0;
        let mut current_bonus: Score = 0;
        let mut pattern_char_idx = 0;

        let mut chars = candidate.chars_from(start_offset).enumerate();

        for (char_offset, candidate_ch) in chars.by_ref() {
            let pattern_ch = pattern.char(pattern_char_idx);

            let char_class = char_class(candidate_ch, scheme);

            if (char_eq)(pattern_ch, candidate_ch) {
                if pattern_char_idx == 0 {
                    bonus_start = current_start_offset + char_offset;
                    start_offset += char_offset + 1;
                    current_bonus =
                        compute_bonus(prev_class, char_class, scheme);
                }

                pattern_char_idx += 1;

                if pattern_char_idx == pattern.char_len() {
                    matched = true;

                    if current_bonus as i64 > best_bonus {
                        best_bonus = current_bonus as _;

                        best_bonus_start = bonus_start;

                        best_bonus_end =
                            current_start_offset + char_offset + 1;
                    }

                    if current_bonus >= bonus::BOUNDARY {
                        break 'outer;
                    }

                    break;
                }
            } else if pattern_char_idx > 0 {
                break;
            }

            prev_class = char_class;
        }

        if chars.next().is_none() {
            break;
        }
    }

    if !matched {
        return None;
    }

    let matched_range = best_bonus_start..best_bonus_end;

    // let score = calculate_score(
    //     pattern,
    //     candidate,
    //     matched_range.clone(),
    //     opts,
    //     scheme,
    //     None,
    // );

    if RANGES {
        ranges.insert(matched_range);
    }

    todo!();

    // Some(score)
}

/// TODO: docs
#[inline]
pub(super) fn prefix_match<const RANGES: bool>(
    pattern: Pattern,
    candidate: Candidate,
    char_eq: CharEq,
    scheme: &Scheme,
    ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    let mut pattern_chars = pattern.chars();

    let ignored_leading_spaces =
        ignored_candidate_leading_spaces(pattern, candidate)?;

    let mut match_byte_len = 0;

    for (candidate_ch, pattern_ch) in candidate
        .chars_from(ignored_leading_spaces)
        .zip(pattern_chars.by_ref())
    {
        if !char_eq(pattern_ch, candidate_ch) {
            return None;
        }
        if RANGES {
            match_byte_len += candidate_ch.len_utf8();
        }
    }

    if pattern_chars.next().is_some() {
        return None;
    }

    // let score = calculate_score(
    //     pattern,
    //     candidate,
    //     matched_range.clone(),
    //     opts,
    //     scheme,
    //     None,
    // );

    if RANGES {
        let start = ignored_leading_spaces;
        let end = start + match_byte_len;
        ranges.insert(start..end);
    }

    todo!()

    // Some(score)
}

/// TODO: docs
#[inline]
pub(super) fn suffix_match<const RANGES: bool>(
    pattern: Pattern,
    candidate: Candidate,
    char_eq: CharEq,
    scheme: &Scheme,
    ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    let mut pattern_chars = pattern.chars().rev();

    let chars_up_to_ignored_spaces = candidate.char_len()
        - ignored_candidate_trailing_spaces(pattern, candidate)?;

    for (candidate_ch, pattern_ch) in candidate
        .slice(0..chars_up_to_ignored_spaces)
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

    // let score = calculate_score(
    //     pattern,
    //     candidate,
    //     matched_range.clone(),
    //     opts,
    //     scheme,
    //     None,
    // );

    if RANGES {
        let end = chars_up_to_ignored_spaces;
        let start = end - pattern.char_len();
        ranges.insert(candidate.to_byte_range(start..end));
    }

    todo!()

    // Some(score)
}

/// TODO: docs
#[inline]
pub(super) fn equal_match<const RANGES: bool>(
    pattern: Pattern,
    candidate: Candidate,
    char_eq: CharEq,
    scheme: &Scheme,
    ranges: &mut MatchedRanges,
) -> Option<Score> {
    if pattern.is_empty() {
        return Some(0);
    }

    let ignored_leading_spaces =
        ignored_candidate_leading_spaces(pattern, candidate)?;

    // The candidate contains only spaces.
    if ignored_leading_spaces == candidate.char_len() {
        return None;
    }

    let ignored_trailing_spaces =
        ignored_candidate_trailing_spaces(pattern, candidate)?;

    let matched_char_range =
        ignored_leading_spaces..candidate.char_len() - ignored_trailing_spaces;

    if matched_char_range.len() < pattern.char_len() {
        return None;
    }

    let mut pattern_chars = pattern.chars();

    let mut candidate_chars =
        candidate.slice(matched_char_range.clone()).chars();

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

    // let score = calculate_score(
    //     pattern,
    //     candidate,
    //     matched_char_range.clone(),
    //     opts,
    //     scheme,
    //     None,
    // );

    if RANGES {
        ranges.insert(candidate.to_byte_range(matched_char_range));
    }

    todo!();

    // Some(score)
}

/// TODO: docs
#[inline(always)]
fn ignored_candidate_leading_spaces(
    pattern: Pattern,
    candidate: Candidate,
) -> Option<usize> {
    let candidate_leading_spaces = candidate.leading_spaces();

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
    candidate: Candidate,
) -> Option<usize> {
    let candidate_trailing_spaces = candidate.trailing_spaces();

    if pattern.trailing_spaces() > candidate_trailing_spaces {
        None
    } else {
        Some(candidate_trailing_spaces - pattern.trailing_spaces())
    }
}

/// TODO: docs
#[inline]
pub(super) fn calculate_score(
    pattern: Pattern,
    candidate: &str,
    candidate_range: Range<usize>,
    opts: impl Opts,
    scheme: &Scheme,
    mut ranges_buf: Option<&mut MatchedRanges>,
) -> Score {
    // TODO: docs
    let mut is_in_gap = false;

    // TODO: docs
    let mut is_first_pattern_char = true;

    // TODO: docs
    let mut first_bonus: Score = 0;

    // TODO: docs
    let mut consecutive = 0u32;

    let range_start = candidate_range.start;

    let mut prev_class = candidate[..candidate_range.start]
        .chars()
        .next_back()
        .map(|ch| char_class(ch, scheme))
        .unwrap_or(scheme.initial_char_class);

    let mut pattern_chars = pattern.chars();

    let mut pattern_char = pattern_chars.next().expect("pattern is not empty");

    let mut score: Score = 0;

    for (offset, candidate_ch) in candidate[candidate_range].char_indices() {
        let ch_class = char_class(candidate_ch, scheme);

        if opts.char_eq(pattern_char, candidate_ch) {
            score += bonus::MATCH;

            let mut bonus = compute_bonus(prev_class, ch_class, scheme);

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

            if let Some(ranges) = &mut ranges_buf {
                let start = range_start + offset;
                let end = start + candidate_ch.len_utf8();
                ranges.insert(start..end);
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

// #[cfg(test)]
// mod tests {
//     #![allow(clippy::single_range_in_vec_init)]
//
//     use super::*;
//
//     #[test]
//     fn equal_match_1() {
//         let pattern =
//             Pattern::parse("^AbC$".chars().collect::<Vec<_>>().leak());
//
//         let mut ranges_buf = MatchedRanges::default();
//
//         assert!(exact_match(
//             pattern,
//             "ABC",
//             todo!(),
//             &Scheme::default(),
//             Some(&mut ranges_buf)
//         )
//         .is_none());
//
//         {
//             ranges_buf = MatchedRanges::default();
//
//             assert!(exact_match(
//                 pattern,
//                 "AbC",
//                 todo!(),
//                 &Scheme::default(),
//                 Some(&mut ranges_buf)
//             )
//             .is_some());
//
//             assert_eq!(ranges_buf.as_slice(), [0..3]);
//         }
//
//         {
//             ranges_buf = MatchedRanges::default();
//
//             assert!(exact_match(
//                 pattern,
//                 "AbC ",
//                 todo!(),
//                 &Scheme::default(),
//                 Some(&mut ranges_buf)
//             )
//             .is_some());
//
//             assert_eq!(ranges_buf.as_slice(), [0..3]);
//         }
//
//         {
//             ranges_buf = MatchedRanges::default();
//
//             assert!(exact_match(
//                 pattern,
//                 " AbC ",
//                 todo!(),
//                 &Scheme::default(),
//                 Some(&mut ranges_buf)
//             )
//             .is_some());
//
//             assert_eq!(ranges_buf.as_slice(), [1..4]);
//         }
//
//         {
//             ranges_buf = MatchedRanges::default();
//
//             assert!(exact_match(
//                 pattern,
//                 "  AbC",
//                 todo!(),
//                 &Scheme::default(),
//                 Some(&mut ranges_buf)
//             )
//             .is_some());
//
//             assert_eq!(ranges_buf.as_slice(), [2..5]);
//         }
//     }
//
//     #[test]
//     fn exact_match_1() {
//         let pattern = Pattern::parse("abc".chars().collect::<Vec<_>>().leak());
//
//         let mut ranges_buf = MatchedRanges::default();
//
//         assert!(exact_match(
//             pattern,
//             "aabbcc abc",
//             todo!(),
//             &Scheme::default(),
//             Some(&mut ranges_buf)
//         )
//         .is_some());
//
//         assert_eq!(ranges_buf.as_slice(), [7..10]);
//     }
// }
