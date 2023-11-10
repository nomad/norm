use core::ops::Range;

use crate::MatchedRanges;

/// TODO: docs
#[derive(Default)]
pub struct Match<D: Copy> {
    /// TODO: docs
    distance: D,

    /// TODO: docs
    matched_ranges: MatchedRanges,
}

impl<D: Copy> Match<D> {
    /// TODO: docs
    #[inline(always)]
    pub fn distance(&self) -> D {
        self.distance
    }

    /// TODO: docs
    #[inline(always)]
    pub fn matched_ranges(&self) -> &[Range<usize>] {
        self.matched_ranges.as_slice()
    }

    /// TODO: docs
    #[inline(always)]
    pub(crate) fn new(distance: D, matched_ranges: MatchedRanges) -> Self {
        Self { distance, matched_ranges }
    }
}
