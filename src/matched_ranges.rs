use core::ops::Range;

use crate::tiny_vec::TinyVec;

/// TODO: docs
#[derive(Default)]
pub(crate) struct MatchedRanges {
    ranges: TinyVec<8, Range<usize>>,
}

impl core::fmt::Debug for MatchedRanges {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.ranges.fmt(f)
    }
}

impl MatchedRanges {
    /// TODO: docs
    #[inline(always)]
    pub(crate) fn as_slice(&self) -> &[Range<usize>] {
        self.ranges.as_slice()
    }

    /// TODO: docs
    #[inline(always)]
    pub(crate) fn insert(&mut self, new_range: Range<usize>) {
        let insert_idx = match self
            .ranges
            .binary_search_by(|range| range.start.cmp(&new_range.start))
        {
            Err(idx) => idx,

            // The range at `idx` and the new range have the same start.
            Ok(idx) => {
                let (range, next_range) = {
                    let (left, right) = self.ranges.split_at_mut(idx + 1);
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
                        self.ranges.remove(idx + 1);
                        return;
                    }
                }

                range.end = new_range.end;

                return;
            },
        };

        if insert_idx == 0 {
            let Some(first_range) = self.ranges.get_mut(0) else {
                // This is the first range.
                self.ranges.push(new_range);
                return;
            };

            if new_range.end >= first_range.start {
                first_range.start = new_range.start;
            } else {
                self.ranges.insert(0, new_range);
            }

            return;
        }

        if insert_idx == self.ranges.len() {
            let last_range = &mut self.ranges[insert_idx - 1];

            if new_range.start <= last_range.end {
                last_range.end = last_range.end.max(new_range.end);
            } else {
                self.ranges.push(new_range);
            }

            return;
        }

        let (prev_range, next_range) = {
            let (left, right) = self.ranges.split_at_mut(insert_idx);
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
                self.ranges.remove(insert_idx);
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
                self.ranges.insert(insert_idx, new_range);
            },

            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::single_range_in_vec_init)]

    use super::*;

    #[test]
    fn matched_ranges_insert_same_start_increasing_end() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(0..1);
        ranges.insert(0..2);
        ranges.insert(0..3);
        assert_eq!(ranges.as_slice(), [0..3]);
        ranges.insert(0..2);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_consecutive_1() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(0..1);
        ranges.insert(1..2);
        ranges.insert(2..3);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_consecutive_2() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(2..3);
        ranges.insert(1..2);
        ranges.insert(0..1);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_fill_gap() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(0..1);
        ranges.insert(2..3);
        assert_eq!(ranges.as_slice(), [0..1, 2..3]);
        ranges.insert(1..2);
        assert_eq!(ranges.as_slice(), [0..3]);
    }

    #[test]
    fn matched_ranges_insert_extend_end() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(0..2);
        ranges.insert(4..6);
        ranges.insert(1..3);
        assert_eq!(ranges.as_slice(), [0..3, 4..6]);
    }

    #[test]
    fn matched_ranges_insert_extend_start() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(0..2);
        ranges.insert(4..6);
        ranges.insert(3..5);
        assert_eq!(ranges.as_slice(), [0..2, 3..6]);
    }

    #[test]
    fn matched_ranges_insert_in_gap() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(0..4);
        ranges.insert(6..8);
        ranges.insert(10..14);
        assert_eq!(ranges.as_slice(), [0..4, 6..8, 10..14]);
    }

    #[test]
    fn matched_ranges_insert_smaller_1() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [3..8]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [3..8]);
    }

    #[test]
    fn matched_ranges_insert_smaller_2() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(1..2);
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [1..2, 3..8]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [1..2, 3..8]);
    }

    #[test]
    fn matched_ranges_insert_smaller_3() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(10..11);
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [3..8, 10..11]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [3..8, 10..11]);
    }

    #[test]
    fn matched_ranges_insert_smaller_4() {
        let mut ranges = MatchedRanges::default();
        ranges.insert(1..2);
        ranges.insert(10..11);
        ranges.insert(3..8);
        ranges.insert(4..7);
        assert_eq!(ranges.as_slice(), [1..2, 3..8, 10..11]);
        ranges.insert(5..6);
        assert_eq!(ranges.as_slice(), [1..2, 3..8, 10..11]);
    }
}
