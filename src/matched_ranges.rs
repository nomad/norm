use core::ops::Range;

/// TODO: docs
#[derive(Default)]
pub(crate) struct MatchedRanges {
    ranges: Vec<Range<usize>>,
}

impl MatchedRanges {
    /// TODO: docs
    #[inline(always)]
    pub(crate) fn as_slice(&self) -> &[Range<usize>] {
        self.ranges.as_slice()
    }

    /// TODO: docs
    #[inline(always)]
    pub(crate) fn last_mut(&mut self) -> Option<&mut Range<usize>> {
        self.ranges.last_mut()
    }

    /// TODO: docs
    #[inline(always)]
    pub(crate) fn push(&mut self, range: Range<usize>) {
        self.ranges.push(range);
    }
}
