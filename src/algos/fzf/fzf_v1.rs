use core::ops::Range;

use super::{query::*, *};
use crate::*;

/// TODO: docs
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
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
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

impl Metric for FzfV1 {
    type Query<'a> = FzfQuery<'a>;

    type Distance = FzfDistance;

    #[inline(always)]
    fn distance(
        &mut self,
        query: FzfQuery<'_>,
        candidate: &str,
    ) -> Option<Self::Distance> {
        let ranges = &mut MatchedRanges::default();
        <Self as Fzf>::distance::<false>(self, query, candidate, ranges)
    }

    #[inline]
    fn distance_and_ranges(
        &mut self,
        query: FzfQuery<'_>,
        candidate: &str,
        ranges: &mut MatchedRanges,
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
    fn scheme(&self) -> &Scheme {
        &self.scheme
    }

    #[inline(always)]
    fn fuzzy<const RANGES: bool>(
        &mut self,
        pattern: Pattern,
        _candidate: Candidate,
        _ranges: &mut MatchedRanges,
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

        let _opts = CandidateOpts::new(is_sensitive, self.normalization);

        todo!();

        // let range_forward = forward_pass(pattern, candidate, opts)?;
        //
        // let start_backward =
        //     backward_pass(pattern, &candidate[range_forward.clone()], opts);
        //
        // let range = range_forward.start + start_backward..range_forward.end;
        //
        // let score =
        //     calculate_score(pattern, candidate, range, opts, scheme, ranges_buf);
        //
        // Some(score)
    }
}

/// TODO: docs
#[inline]
fn forward_pass(
    pattern: Pattern,
    mut candidate: &str,
    opts: impl Opts,
) -> Option<Range<usize>> {
    let mut pattern_chars = pattern.chars();

    let mut pattern_char = pattern_chars.next()?;

    let (start_offset, matched_char_byte_len) =
        opts.find_first(pattern_char, candidate)?;

    let mut end_offset = start_offset + matched_char_byte_len;

    if let Some(next) = pattern_chars.next() {
        pattern_char = next;
    } else {
        return Some(start_offset..end_offset);
    }

    // SAFETY: todo.
    candidate = unsafe { candidate.get_unchecked(end_offset..) };

    loop {
        let (byte_offset, matched_char_byte_len) =
            opts.find_first(pattern_char, candidate)?;

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
    opts: impl Opts,
) -> usize {
    // The candidate must start with the first character of the query.
    debug_assert!(opts.char_eq(
        pattern.chars().next().unwrap(),
        candidate.chars().next().unwrap(),
    ));

    // The candidate must end with the last character of the query.
    debug_assert!(opts.char_eq(
        pattern.chars().next_back().unwrap(),
        candidate.chars().next_back().unwrap(),
    ));

    let mut pattern_chars = pattern.chars().rev();

    let mut pattern_char = pattern_chars.next().expect("pattern is not empty");

    loop {
        let (byte_offset, _) =
            opts.find_last(pattern_char, candidate).unwrap();

        if let Some(next) = pattern_chars.next() {
            pattern_char = next;
        } else {
            return byte_offset;
        }

        // SAFETY: todo.
        candidate = unsafe { candidate.get_unchecked(..byte_offset) };
    }
}
