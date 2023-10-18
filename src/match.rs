use core::ops::Range;

/// TODO: docs
pub struct Match<D: Copy> {
    /// TODO: docs
    distance: D,
}

impl<D: Copy> Match<D> {
    /// TODO: docs
    pub fn distance(&self) -> D {
        self.distance
    }

    /// TODO: docs
    pub fn matched_ranges(&self) -> &[Range<usize>] {
        &[]
    }

    /// TODO: docs
    #[inline]
    pub(crate) fn new(distance: D, matched_ranges: Vec<Range<usize>>) -> Self {
        Self { distance }
    }
}
