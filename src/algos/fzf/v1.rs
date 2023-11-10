use core::ops::Range;

use super::{query::*, *};
use crate::*;

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v1")))]
#[derive(Clone, Default)]
pub struct FzfV1 {
    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    normalization: bool,

    /// TODO: docs
    scheme: Scheme,

    /// TODO: docs
    with_matched_ranges: bool,
}

impl core::fmt::Debug for FzfV1 {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FzfV1")
            .field("case_sensitivity", &self.case_sensitivity)
            .field("matched_ranges", &self.with_matched_ranges)
            .field("normalization", &self.normalization)
            .field("scheme", &FzfScheme::from_inner(&self.scheme).unwrap())
            .finish_non_exhaustive()
    }
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
    pub fn with_normalization(mut self, normalization: bool) -> Self {
        self.normalization = normalization;
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
            return Some(Match::default());
        }

        let is_candidate_ascii = candidate.is_ascii();

        let conditions = match query.search_mode {
            SearchMode::NotExtended(pattern) => {
                let is_case_sensitive = match self.case_sensitivity {
                    CaseSensitivity::Sensitive => true,
                    CaseSensitivity::Insensitive => false,
                    CaseSensitivity::Smart => pattern.has_uppercase,
                };

                let char_eq =
                    utils::char_eq(is_case_sensitive, self.normalization);

                let (score, matched_ranges) = fzf_v1(
                    pattern,
                    candidate,
                    &self.scheme,
                    char_eq,
                    is_case_sensitive,
                    self.with_matched_ranges,
                    is_candidate_ascii,
                )?;

                let distance = FzfDistance::from_score(score);

                return Some(Match::new(distance, matched_ranges));
            },

            SearchMode::Extended(conditions) => conditions,
        };

        let mut total_score = 0;

        let mut matched_ranges = MatchedRanges::default();

        for condition in conditions {
            let (score, ranges) =
                condition.or_patterns().find_map(|pattern| {
                    let is_case_sensitive = match self.case_sensitivity {
                        CaseSensitivity::Sensitive => true,
                        CaseSensitivity::Insensitive => false,
                        CaseSensitivity::Smart => pattern.has_uppercase,
                    };

                    let char_eq =
                        utils::char_eq(is_case_sensitive, self.normalization);

                    pattern.score(
                        candidate,
                        &self.scheme,
                        char_eq,
                        is_case_sensitive,
                        self.with_matched_ranges,
                        is_candidate_ascii,
                        fzf_v1,
                    )
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
pub(super) fn fzf_v1(
    pattern: Pattern,
    candidate: &str,
    scheme: &Scheme,
    char_eq: CharEq,
    is_case_sensitive: bool,
    with_matched_ranges: bool,
    is_candidate_ascii: bool,
) -> Option<(Score, MatchedRanges)> {
    let range_forward = forward_pass(
        pattern,
        candidate,
        is_candidate_ascii,
        is_case_sensitive,
        char_eq,
    )?;

    let start_backward = backward_pass(
        pattern,
        &candidate[range_forward.clone()],
        is_candidate_ascii,
        is_case_sensitive,
        char_eq,
    );

    let range = range_forward.start + start_backward..range_forward.end;

    let (score, matched_ranges) = calculate_score(
        pattern,
        candidate,
        range,
        scheme,
        char_eq,
        with_matched_ranges,
    );

    Some((score, matched_ranges))
}

/// TODO: docs
#[inline]
fn forward_pass(
    pattern: Pattern,
    mut candidate: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    char_eq: CharEq,
) -> Option<Range<usize>> {
    let mut pattern_chars = pattern.chars();

    let mut pattern_char = pattern_chars.next()?;

    let (start_offset, matched_char) = utils::find_first(
        pattern_char,
        candidate,
        is_candidate_ascii,
        is_case_sensitive,
        char_eq,
    )?;

    let matched_char_byte_len = matched_char.len_utf8();

    let mut end_offset = start_offset + matched_char_byte_len;

    if let Some(next) = pattern_chars.next() {
        pattern_char = next;
    } else {
        return Some(start_offset..end_offset);
    }

    // SAFETY: todo.
    candidate = unsafe { candidate.get_unchecked(end_offset..) };

    loop {
        let (byte_offset, matched_char) = utils::find_first(
            pattern_char,
            candidate,
            is_candidate_ascii,
            is_case_sensitive,
            char_eq,
        )?;

        let matched_char_byte_len = matched_char.len_utf8();

        end_offset += byte_offset + matched_char_byte_len;

        if let Some(next) = pattern_chars.next() {
            pattern_char = next;
        } else {
            return Some(start_offset..end_offset);
        }

        // SAFETY: todo.
        candidate = unsafe {
            candidate.get_unchecked(byte_offset + matched_char_byte_len..)
        };
    }
}

/// TODO: docs
#[inline]
fn backward_pass(
    pattern: Pattern,
    mut candidate: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    char_eq: CharEq,
) -> usize {
    // The candidate must start with the first character of the query.
    debug_assert!(char_eq(
        candidate.chars().next().unwrap(),
        pattern.chars().next().unwrap()
    ));

    // The candidate must end with the last character of the query.
    debug_assert!(char_eq(
        candidate.chars().next_back().unwrap(),
        pattern.chars().next_back().unwrap()
    ));

    let mut pattern_chars = pattern.chars().rev();

    let mut pattern_char = pattern_chars.next().expect("pattern is not empty");

    loop {
        let (byte_offset, _) = utils::find_last(
            pattern_char,
            candidate,
            is_candidate_ascii,
            is_case_sensitive,
            char_eq,
        )
        .unwrap();

        if let Some(next) = pattern_chars.next() {
            pattern_char = next;
        } else {
            return byte_offset;
        }

        // SAFETY: todo.
        candidate = unsafe { candidate.get_unchecked(..byte_offset) };
    }
}
