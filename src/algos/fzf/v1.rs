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

        let pattern = match query.search_mode {
            SearchMode::NotExtended(pattern) => pattern,
            SearchMode::Extended(_) => todo!(),
        };

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
