use core::ops::Range;

/// TODO: docs
pub struct Match<D> {
    /// TODO: docs
    distance: D,
}

impl<D> Match<D> {
    /// TODO: docs
    #[inline]
    pub(crate) fn new(distance: D, matched_ranges: Vec<Range<usize>>) -> Self {
        Self { distance }
    }

    /// TODO: docs
    pub fn matched_ranges(&self) -> &[Range<usize>] {
        &[]
    }
}
