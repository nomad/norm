use core::ops::Range;

/// TODO: docs
pub trait SortedRanges {
    fn sorted(&self) -> Vec<Range<usize>>;
}

impl SortedRanges for &[Range<usize>] {
    fn sorted(&self) -> Vec<Range<usize>> {
        let mut sorted = self.to_vec();
        sorted.sort_by_key(|r| r.start);
        sorted
    }
}
