use core::cmp::Ordering;
use core::ops::Range;

/// TODO: docs
pub(crate) struct MatchedRanges<'a> {
    ranges: &'a mut Vec<Range<usize>>,
    initial_len: usize,
}

impl<'a> From<&'a mut Vec<Range<usize>>> for MatchedRanges<'a> {
    #[inline(always)]
    fn from(ranges: &'a mut Vec<Range<usize>>) -> Self {
        let initial_len = ranges.len();
        Self { ranges, initial_len }
    }
}

impl core::fmt::Debug for MatchedRanges<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;

        let (initial, this) = self.ranges.split_at(self.initial_len);

        if this.is_empty() {
            return initial.fmt(f);
        }

        f.write_char('[')?;

        for (idx, initial) in initial.iter().enumerate() {
            write!(f, "{initial:?}")?;
            if idx + 1 < initial.len() {
                f.write_str(", ")?;
            }
        }

        f.write_str(" | ")?;

        for (idx, this) in this.iter().enumerate() {
            write!(f, "{this:?}")?;
            if idx + 1 < this.len() {
                f.write_str(", ")?;
            }
        }

        f.write_char(']')?;

        Ok(())
    }
}

impl<'a> MatchedRanges<'a> {
    /// TODO: docs
    #[inline(always)]
    fn binary_search_by<'r, F>(&'r self, fun: F) -> Result<usize, usize>
    where
        F: FnMut(&'r Range<usize>) -> Ordering,
    {
        self.ranges[self.initial_len..].binary_search_by(fun)
    }

    /// TODO: docs
    #[inline(always)]
    fn get_mut(&mut self, idx: usize) -> Option<&mut Range<usize>> {
        self.ranges.get_mut(self.initial_len + idx)
    }

    /// TODO: docs
    #[inline(always)]
    pub(crate) fn insert(&mut self, new_range: Range<usize>) {
        let insert_idx = match self
            .binary_search_by(|range| range.start.cmp(&new_range.start))
        {
            Err(idx) => idx,

            // The range at `idx` and the new range have the same start.
            Ok(idx) => {
                let (range, next_range) = {
                    let (left, right) = self.split_at_mut(idx + 1);
                    (&mut left[idx], right.get_mut(0))
                };

                if range.end >= new_range.end {
                    // The new range is completely contained within this
                    // existing range.
                    return;
                }

                if let Some(next_range) = next_range {
                    if new_range.end >= next_range.start {
                        // The new range fills the gap between this range and
                        // the next one.
                        range.end = next_range.end;
                        self.remove(idx + 1);
                        return;
                    }
                }

                range.end = new_range.end;

                return;
            },
        };

        if insert_idx == 0 {
            let Some(first_range) = self.get_mut(0) else {
                // This is the first range.
                self.push(new_range);
                return;
            };

            if new_range.end >= first_range.start {
                first_range.start = new_range.start;
            } else {
                self.insert_at(0, new_range);
            }

            return;
        }

        if insert_idx == self.len() {
            let last_range = self.last_mut().unwrap();

            if new_range.start <= last_range.end {
                last_range.end = last_range.end.max(new_range.end);
            } else {
                self.push(new_range);
            }

            return;
        }

        let (prev_range, next_range) = {
            let (left, right) = self.split_at_mut(insert_idx);
            (&mut left[insert_idx - 1], &mut right[0])
        };

        match (
            new_range.start <= prev_range.end,
            new_range.end >= next_range.start,
        ) {
            // The new range fills the gap between two existing ranges, so
            // we merge them.
            //
            // ------   ------    =>    ---------------
            //     xxxxxxx
            (true, true) => {
                prev_range.end = next_range.end;
                self.remove(insert_idx);
            },

            // The new range starts within an existing range but ends before
            // the next one starts, so we extend the end of the existing range.
            //
            // ------    ------    =>    --------  ------
            //     xxxx
            (true, false) if new_range.end > prev_range.end => {
                prev_range.end = new_range.end;
            },

            // The new range ends within an existing range but starts after
            // the previous one ends, so we extend the start of the existing
            // range.
            //
            // ------    ------    =>    ------  --------
            //         xxxx
            (false, true) => {
                next_range.start = new_range.start;
            },

            // The new range is strictly within an existing gap, so we just
            // insert it.
            // ------         ------    =>   ------  -----  ------
            //         xxxxx
            (false, false) => {
                self.insert_at(insert_idx, new_range);
            },

            _ => {},
        }
    }

    /// TODO: docs
    #[inline(always)]
    fn insert_at(&mut self, idx: usize, range: Range<usize>) {
        self.ranges.insert(self.initial_len + idx, range);
    }

    /// TODO: docs
    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut Range<usize>> {
        self.ranges.last_mut()
    }

    /// TODO: docs
    #[inline(always)]
    fn len(&self) -> usize {
        self.ranges.len() - self.initial_len
    }

    /// TODO: docs
    #[inline(always)]
    fn push(&mut self, range: Range<usize>) {
        self.ranges.push(range);
    }

    /// TODO: docs
    #[inline(always)]
    fn remove(&mut self, idx: usize) -> Range<usize> {
        self.ranges.remove(self.initial_len + idx)
    }

    /// TODO: docs
    #[inline(always)]
    fn split_at_mut(
        &mut self,
        idx: usize,
    ) -> (&mut [Range<usize>], &mut [Range<usize>]) {
        let len = self.initial_len;
        let (left, right) = self.ranges.split_at_mut(len + idx);
        let left = &mut left[len..];
        (left, right)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::single_range_in_vec_init)]

    use super::*;

    impl<'a> MatchedRanges<'a> {
        fn as_slice(&self) -> &[Range<usize>] {
            &self.ranges[..]
        }
    }

    fn ranges() -> MatchedRanges<'static> {
        let vec = Box::leak(Box::default());
        MatchedRanges::from(vec)
    }

    #[test]
    fn matched_ranges_insert_same_start_increasing_end() {
        let mut ranges = ranges();
        ranges.insert(0..1);
        ranges.insert(0..2);
        ranges.insert(0..3);
        assert_eq!(ranges.as_slice(), [0..3]);
        ranges.insert(0..2);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_consecutive_1() {
        let mut ranges = ranges();
        ranges.insert(0..1);
        ranges.insert(1..2);
        ranges.insert(2..3);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_consecutive_2() {
        let mut ranges = ranges();
        ranges.insert(2..3);
        ranges.insert(1..2);
        ranges.insert(0..1);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_fill_gap() {
        let mut ranges = ranges();
        ranges.insert(0..1);
        ranges.insert(2..3);
        assert_eq!(ranges.as_slice(), [0..1, 2..3]);
        ranges.insert(1..2);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_extend_end() {
        let mut ranges = ranges();
        ranges.insert(0..2);
        ranges.insert(4..6);
        ranges.insert(1..3);
        assert_eq!(ranges.as_slice(), [0..3, 4..6]);
    }

    #[test]
    fn matched_ranges_insert_extend_start() {
        let mut ranges = ranges();
        ranges.insert(0..2);
        ranges.insert(4..6);
        ranges.insert(3..5);
        assert_eq!(ranges.as_slice(), [0..2, 3..6]);
    }

    #[test]
    fn matched_ranges_insert_in_gap() {
        let mut ranges = ranges();
        ranges.insert(0..4);
        ranges.insert(6..8);
        ranges.insert(10..14);
        assert_eq!(ranges.as_slice(), [0..4, 6..8, 10..14]);
    }

    #[test]
    fn matched_ranges_insert_smaller_1() {
        let mut ranges = ranges();
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [3..8]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [3..8]);
    }

    #[test]
    fn matched_ranges_insert_smaller_2() {
        let mut ranges = ranges();
        ranges.insert(1..2);
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [1..2, 3..8]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [1..2, 3..8]);
    }

    #[test]
    fn matched_ranges_insert_smaller_3() {
        let mut ranges = ranges();
        ranges.insert(10..11);
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [3..8, 10..11]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [3..8, 10..11]);
    }

    #[test]
    fn matched_ranges_insert_smaller_4() {
        let mut ranges = ranges();
        ranges.insert(1..2);
        ranges.insert(10..11);
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [1..2, 3..8, 10..11]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [1..2, 3..8, 10..11]);
    }
}
