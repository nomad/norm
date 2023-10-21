use core::ops::Range;

use super::fzf_v1::*;
use crate::{CaseSensitivity, Match, Metric};

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Default)]
pub struct FzfV2 {
    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    scheme: scheme::Scheme,

    /// TODO: docs
    with_matched_ranges: bool,
}

impl FzfV2 {
    /// TODO: docs
    #[inline]
    fn fuzzy_match(
        &self,
        query: &str,
        candidate: &str,
    ) -> Option<Range<usize>> {
        todo!()
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
    #[inline]
    pub fn with_scoring_scheme(mut self, scheme: FzfScheme) -> Self {
        self.scheme = match scheme {
            FzfScheme::Default => scheme::DEFAULT,
            FzfScheme::Path => scheme::PATH,
            FzfScheme::History => scheme::HISTORY,
        };
        self
    }
}

impl Metric for FzfV2 {
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
            &self.scheme,
            self.case_sensitivity,
            self.with_matched_ranges,
        );

        let distance = FzfDistance::from_score(score);

        Some(Match::new(distance, matched_ranges))
    }
}
