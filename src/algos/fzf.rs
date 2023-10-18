use core::ops::Range;

use crate::{CaseSensitivity, Match, Metric};

type Distance = u32;

type Score = Distance;

/// TODO: docs.
#[derive(Debug)]
pub struct FzfQuery<'a> {
    /// TODO: docs.
    raw: &'a str,
}

impl<'a> FzfQuery<'a> {
    /// TODO: docs
    #[inline]
    pub fn from_str(s: &'a str) -> Self {
        Self { raw: s }
    }

    /// TODO: docs
    #[inline]
    fn is_empty(&self) -> bool {
        self.raw().is_empty()
    }

    /// TODO: docs
    #[inline]
    fn raw(&self) -> &'a str {
        self.raw
    }
}

/// TODO: docs
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct FzfDistance(Distance);

impl FzfDistance {
    /// TODO: docs
    fn from_score(score: Score) -> Self {
        // The higher the score the lower the distance.
        Self(Distance::MAX - score)
    }

    /// TODO: docs
    fn into_score(self) -> Score {
        // The higher the score the lower the distance.
        Distance::MAX - self.0
    }
}

#[cfg(feature = "fzf-v1")]
pub use v1::FzfV1;

#[cfg(feature = "fzf-v1")]
mod v1 {
    use super::*;

    /// TODO: docs
    #[derive(Default)]
    pub struct FzfV1 {
        /// TODO: docs
        case_sensitivity: CaseSensitivity,

        /// TODO: docs
        with_matched_ranges: bool,
    }

    impl FzfV1 {
        /// TODO: docs
        #[inline]
        fn fuzzy_match(
            &self,
            query: &str,
            candidate: &str,
        ) -> Option<Range<usize>> {
            debug_assert!(!query.is_empty());

            let range_forward =
                forward_pass(query, candidate, self.case_sensitivity)?;

            let candidate = &candidate[range_forward.clone()];

            let start_backward =
                backward_pass(query, candidate, self.case_sensitivity);

            Some(range_forward.start + start_backward..range_forward.end)
        }

        /// TODO: docs
        #[inline]
        pub fn new() -> Self {
            Self::default()
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
        #[doc(hidden)]
        pub fn score(
            &self,
            query: FzfQuery<'_>,
            candidate: &str,
        ) -> Option<Score> {
            self.distance(query, candidate)
                .map(|m| m.distance())
                .map(FzfDistance::into_score)
        }
    }

    impl Metric for FzfV1 {
        type Query<'a> = FzfQuery<'a>;

        type Distance = FzfDistance;

        #[inline]
        fn distance(
            &self,
            query: FzfQuery<'_>, // helwo
            candidate: &str,     // Hello World!
        ) -> Option<Match<Self::Distance>> {
            if query.is_empty() {
                return None;
            }

            let range = self.fuzzy_match(query.raw(), candidate)?;

            let (score, matched_ranges) = calculate_score(
                query.raw(),
                candidate,
                range,
                self.case_sensitivity,
                self.with_matched_ranges,
            );

            let distance = FzfDistance::from_score(score);

            Some(Match::new(distance, matched_ranges))
        }
    }

    /// TODO: docs
    #[inline]
    fn forward_pass(
        query: &str,
        candidate: &str,
        case_sensitivity: CaseSensitivity,
    ) -> Option<Range<usize>> {
        let mut start_offset = None;

        let mut end_offset = None;

        let mut query_chars = query.chars();

        let mut query_char = query_chars.next().expect("query is not empty");

        for (offset, mut candidate_char) in candidate.char_indices() {
            if case_sensitivity.is_insensitive() {
                candidate_char.make_ascii_lowercase();
            }

            if query_char != candidate_char {
                continue;
            }

            if start_offset.is_none() {
                start_offset = Some(offset);
            }

            let Some(next_target_char) = query_chars.next() else {
                end_offset = Some(offset + candidate_char.len_utf8());
                break;
            };

            query_char = next_target_char;
        }

        let (Some(start), Some(end)) = (start_offset, end_offset) else {
            return None;
        };

        Some(start..end)
    }

    /// TODO: docs
    #[inline]
    fn backward_pass(
        query: &str,
        candidate: &str,
        case_sensitivity: CaseSensitivity,
    ) -> usize {
        // The candidate must start with the first character of the query.
        debug_assert!(candidate.starts_with(query.chars().next().unwrap()));

        // The candidate must end with the last character of the query.
        debug_assert!(candidate.ends_with(query.chars().next_back().unwrap()));

        let mut start_offset = 0;

        let mut query_chars = query.chars().rev();

        let mut query_char = query_chars.next().expect("query is not empty");

        for (offset, mut candidate_char) in candidate.char_indices().rev() {
            if case_sensitivity.is_insensitive() {
                candidate_char.make_ascii_lowercase();
            }

            if query_char != candidate_char {
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
}

/// TODO: docs
#[inline]
fn calculate_score(
    query: &str,
    candidate: &str,
    range: Range<usize>,
    case_sensitivity: CaseSensitivity,
    track_matched_ranges: bool,
) -> (Score, Vec<Range<usize>>) {
    // TODO: docs
    let mut is_in_gap = false;

    // TODO: docs
    let mut is_first_query_char = true;

    // TODO: docs
    let mut first_bonus = 0u32;

    // TODO: docs
    let mut consecutive = 0u32;

    let mut prev_class = candidate[..range.start]
        .chars()
        .next_back()
        .map(CharClass::from)
        .unwrap_or(CharClass::WhiteSpace);

    let mut query_chars = query.chars();

    let mut query_char = query_chars.next().expect("query is not empty");

    let mut score = 0u32;

    let mut matched_ranges = Vec::new();

    for (offset, mut candidate_ch) in candidate[range].char_indices() {
        let ch_class = CharClass::from(candidate_ch);

        if case_sensitivity.is_insensitive() {
            candidate_ch.make_ascii_lowercase();
        }

        if candidate_ch == query_char {
            score += bonus::MATCH;

            let mut bonus = bonus(prev_class, ch_class);

            if consecutive == 0 {
                first_bonus = bonus;
            } else {
                if bonus >= bonus::BOUNDARY && bonus > first_bonus {
                    first_bonus = bonus
                }
                bonus = bonus.max(first_bonus).max(bonus::CONSECUTIVE);
            }

            score += if is_first_query_char {
                bonus * bonus::FIRST_QUERY_CHAR_MULTIPLIER
            } else {
                bonus
            };

            if track_matched_ranges {
                if consecutive == 0 {
                    let range = offset..(offset + candidate_ch.len_utf8());
                    matched_ranges.push(range);
                } else if let Some(last_range) = matched_ranges.last_mut() {
                    last_range.end += candidate_ch.len_utf8();
                } else {
                    unreachable!(
                        "if consecutive is > 0 we've already pushed a range"
                    );
                }
            }

            is_in_gap = false;

            is_first_query_char = false;

            consecutive += 1;

            query_char =
                query_chars.next().expect("query chars not exhausted");
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
fn bonus(prev_class: CharClass, next_class: CharClass) -> Score {
    use CharClass::*;

    let boundary_white = bonus::default::BOUNDARY_WHITE;

    let boundary_delimiter = bonus::default::BOUNDARY_DELIMITER;

    match next_class {
        NonWord => bonus::NON_WORD,

        WhiteSpace => boundary_white,

        Upper if prev_class == Lower => bonus::CAMEL_123,

        Number if prev_class != Number => bonus::CAMEL_123,

        _ => {
            if prev_class == WhiteSpace {
                boundary_white
            } else if prev_class == Delimiter {
                boundary_delimiter
            } else if prev_class == NonWord {
                bonus::BOUNDARY
            } else {
                0
            }
        },
    }
}

/// TODO: docs
#[derive(Clone, Copy, PartialEq, Eq)]
enum CharClass {
    /// TODO: docs
    WhiteSpace,

    /// TODO: docs
    NonWord,

    /// TODO: docs
    Delimiter,

    /// TODO: docs
    Lower,

    /// TODO: docs
    Upper,

    /// TODO: docs
    Letter,

    /// TODO: docs
    Number,
}

impl From<char> for CharClass {
    #[inline]
    fn from(ch: char) -> Self {
        todo!();
    }
}

pub mod bonus {
    //! TODO: docs

    use super::*;

    /// TODO: docs
    pub const MATCH: Score = 16;

    /// TODO: docs
    pub const BOUNDARY: Score = MATCH / 2;

    /// TODO: docs
    pub const NON_WORD: Score = MATCH / 2;

    /// TODO: docs
    pub const CAMEL_123: Score = BOUNDARY - penalty::GAP_EXTENSION;

    /// TODO: docs
    pub const CONSECUTIVE: Score = penalty::GAP_START + penalty::GAP_EXTENSION;

    /// TODO: docs
    pub const FIRST_QUERY_CHAR_MULTIPLIER: Score = 2;

    pub mod default {
        //! TODO: docs

        use super::*;

        /// TODO: docs
        pub const BOUNDARY_WHITE: Score = BOUNDARY + 2;

        /// TODO: docs
        pub const BOUNDARY_DELIMITER: Score = BOUNDARY + 1;
    }

    pub mod path {
        //! TODO: docs

        use super::*;

        /// TODO: docs
        pub const BOUNDARY_WHITE: Score = BOUNDARY;

        /// TODO: docs
        pub const BOUNDARY_DELIMITER: Score = BOUNDARY + 1;
    }

    pub mod history {
        //! TODO: docs

        use super::*;

        /// TODO: docs
        pub const BOUNDARY_WHITE: Score = BOUNDARY;

        /// TODO: docs
        pub const BOUNDARY_DELIMITER: Score = BOUNDARY;
    }
}

pub mod penalty {
    //! TODO: docs

    use super::*;

    /// TODO: docs
    pub const GAP_START: Score = 3;

    /// TODO: docs
    pub const GAP_EXTENSION: Score = 1;
}
