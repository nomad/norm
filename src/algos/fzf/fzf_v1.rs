use core::ops::Range;

use super::{query::*, *};
use crate::*;

/// A metric that implements fzf's v1 algorithm.
///
/// The [`Metric`] implementation of this struct produces the same results that
/// `fzf` would produce when run with the `--algo=v1` flag.
///
/// The algorithm used in the [`distance`](Metric::distance) calculation simply
/// looks for the first fuzzy match of the query in the candidate. If a match
/// is found, it traverses backwards from the end of the match to see if
/// there's a shorter substring that also matches the query.
///
/// By always stopping at the first alignment this metric is able to provide a
/// `O(len(candidate))` time complexity for both matches and non-matches, but
/// it usually produces less accurate results than [`FzfV2`].
///
/// # Example
///
/// ```rust
/// # use norm::fzf::{FzfV1, FzfParser};
/// # use norm::Metric;
/// let mut v1 = FzfV1::new();
/// let mut parser = FzfParser::new();
/// let mut ranges = Vec::new();
///
/// let query = parser.parse("abc");
///
/// let candidate = "a_b_abcd";
///
/// /*
/// We're looking for "abc" in "a_b_abcd". A first scan will find "a_b_abc":
///
/// a_b_abcd
/// *******
///
/// Now it will stop looking for other (potentially better) alignments of the
/// query, and will instead start going backwards from the end of the match,
/// i.e. from "c".
///
/// In this case, it will find "abc", which is shorter than "a_b_abc":
///
/// a_b_abcd
///     ***
///
/// so the matched range will be "abc".
/// */
///
/// let _distance =
///     v1.distance_and_ranges(query, candidate, &mut ranges).unwrap();
///
/// assert_eq!(ranges, [4..7]);
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v1")))]
#[derive(Clone, Default)]
pub struct FzfV1 {
    /// TODO: docs
    candidate_slab: CandidateSlab,

    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    normalization: bool,

    /// TODO: docs
    scheme: Scheme,
}

impl core::fmt::Debug for FzfV1 {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FzfV1")
            .field("case_sensitivity", &self.case_sensitivity)
            .field("normalization", &self.normalization)
            .field("scheme", &FzfScheme::from_inner(&self.scheme).unwrap())
            .finish_non_exhaustive()
    }
}

impl FzfV1 {
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

    /// Sets the case sensitivity to use when comparing the characters of the
    /// query and the candidate. The default is [`CaseSensitivity::Smart`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use norm::fzf::{FzfV1, FzfParser};
    /// # use norm::{Metric, CaseSensitivity};
    /// let mut fzf = FzfV1::new();
    /// let mut parser = FzfParser::new();
    ///
    /// // FzfV1 uses smart case sensitivity by default.
    /// assert!(fzf.distance(parser.parse("abc"), "ABC").is_some());
    ///
    /// fzf.set_case_sensitivity(CaseSensitivity::Sensitive);
    ///
    /// // Now it's case sensitive, so the query won't match the candidate.
    /// assert!(fzf.distance(parser.parse("abc"), "ABC").is_none());
    /// ```
    #[inline(always)]
    pub fn set_case_sensitivity(
        &mut self,
        case_sensitivity: CaseSensitivity,
    ) -> &mut Self {
        self.case_sensitivity = case_sensitivity;
        self
    }

    /// TODO: docs
    #[inline(always)]
    pub fn set_normalization(&mut self, normalization: bool) -> &mut Self {
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

impl Metric for FzfV1 {
    type Query<'a> = FzfQuery<'a>;

    type Distance = FzfDistance;

    #[inline(always)]
    fn distance(
        &mut self,
        query: FzfQuery<'_>,
        candidate: &str,
    ) -> Option<Self::Distance> {
        let ranges = &mut Vec::new();
        <Self as Fzf>::distance::<false>(self, query, candidate, ranges)
    }

    #[inline]
    fn distance_and_ranges(
        &mut self,
        query: FzfQuery<'_>,
        candidate: &str,
        ranges: &mut Vec<Range<usize>>,
    ) -> Option<Self::Distance> {
        <Self as Fzf>::distance::<true>(self, query, candidate, ranges)
    }
}

impl Fzf for FzfV1 {
    #[inline(always)]
    fn alloc_chars<'a>(&mut self, s: &str) -> &'a [char] {
        unsafe { core::mem::transmute(self.candidate_slab.alloc(s)) }
    }

    #[inline(always)]
    fn char_eq(&self, pattern: Pattern) -> utils::CharEq {
        let is_sensitive = match self.case_sensitivity {
            CaseSensitivity::Sensitive => true,
            CaseSensitivity::Insensitive => false,
            CaseSensitivity::Smart => pattern.has_uppercase,
        };

        utils::char_eq(is_sensitive, self.normalization)
    }

    #[inline(always)]
    fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    #[inline(always)]
    fn fuzzy<const RANGES: bool>(
        &mut self,
        pattern: Pattern,
        candidate: Candidate,
        ranges: &mut MatchedRanges,
    ) -> Option<Score> {
        // TODO: can we remove this?
        if pattern.is_empty() {
            return Some(0);
        }

        let is_sensitive = match self.case_sensitivity {
            CaseSensitivity::Sensitive => true,
            CaseSensitivity::Insensitive => false,
            CaseSensitivity::Smart => pattern.has_uppercase,
        };

        let opts = CandidateOpts::new(is_sensitive, self.normalization);

        let end_forward = forward_pass(pattern, candidate, opts)?;

        let start_backward =
            backward_pass(pattern, candidate, end_forward, opts);

        let score = compute_score::<RANGES>(
            pattern,
            candidate,
            start_backward..end_forward,
            opts.char_eq,
            &self.scheme,
            ranges,
        );

        Some(score)
    }
}

/// TODO: docs
#[inline]
fn forward_pass(
    pattern: Pattern,
    candidate: Candidate,
    opts: CandidateOpts,
) -> Option<usize> {
    let mut pattern_chars = pattern.chars();

    let mut pattern_char = pattern_chars.next()?;

    let mut end_offset = 0;

    loop {
        end_offset = candidate.find_first_from(
            end_offset,
            pattern_char,
            opts.is_case_sensitive,
            opts.char_eq,
        )? + 1;

        if let Some(next) = pattern_chars.next() {
            pattern_char = next;
        } else {
            return Some(end_offset);
        }
    }
}

/// TODO: docs
#[inline]
fn backward_pass(
    pattern: Pattern,
    candidate: Candidate,
    end_offset: usize,
    opts: CandidateOpts,
) -> usize {
    let mut pattern_chars = pattern.chars().rev();

    let mut pattern_char = pattern_chars.next().expect("pattern is not empty");

    let mut start_offset = end_offset;

    loop {
        start_offset = candidate
            .find_last_from(
                start_offset,
                pattern_char,
                opts.is_case_sensitive,
                opts.char_eq,
            )
            .unwrap();

        if let Some(next) = pattern_chars.next() {
            pattern_char = next;
        } else {
            return start_offset;
        }
    }
}
