use core::ops::Range;

/// A trait representing a distance metric on strings.
///
/// This trait encapsulates the logic for comparing a query to a candidate
/// string. It allows to filter out non-matches, to sort the remaining
/// candidates based on the quality of the match, and to show which sub-strings
/// of a candidate matched the query.
pub trait Metric {
    /// The type of query to be found in the candidate.
    ///
    /// This is generic over an associated lifetime `'a` to allow for zero-copy
    /// parsing of the query. Metrics that don't parse queries can simply use
    /// a `&'a str` here.
    type Query<'a>;

    /// The type that expresses how closely a candidate matches the query.
    ///
    /// In order to behave like a distance, its [`Ord`] implementation must be
    /// such that given two candidates `a` and `b`, it holds that
    ///
    /// ```
    /// # use core::cmp::Ordering;
    /// # let a_distance = 0;
    /// # let b_distance = 1;
    /// # let _ =
    /// a_distance.cmp(&b_distance) == Ordering::Less
    /// # ;
    /// ```
    ///
    /// if and only if `a` is a better match than `b`. In other words, a lower
    /// distance value must indicate a more relevant match.
    type Distance: Ord;

    /// This method calculates the "distance" between an instance of the
    /// metric's [`Query`][Self::Query] type and a candidate string.
    ///
    /// A return value of `Some(distance)` means that the metric considers the
    /// candidate to be a match for the query, with the `distance` being the
    /// measure of how good the match is: the better the match, the lower the
    /// distance.
    ///
    /// A return value of `None` means that the candidate does not match the
    /// query and should be filtered out of the search results.
    fn distance(
        &mut self,
        query: Self::Query<'_>,
        candidate: &str,
    ) -> Option<Self::Distance>;

    /// This method has the same semantics and return value as
    /// [`Self::distance`], but in the case of a match it also appends the
    /// **byte** ranges of the candidate that matched the query to the provided
    /// buffer.
    ///
    /// The appended ranges are guaranteed to be non-overlapping, but the order
    /// in which they are appended is not specified by this trait's contract.
    /// Any [`Metric`] implementation is free to choose its order as long as
    /// the ranges don't overlap.
    ///
    /// If the candidate doesn't match the query, the buffer is left untouched.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use norm::fzf::{FzfV1, FzfParser};
    /// # use norm::Metric;
    /// let mut fzf = FzfV1::new();
    /// let mut parser = FzfParser::new();
    /// let mut ranges = Vec::new();
    ///
    /// let query = parser.parse("foo");
    ///
    /// let distance = fzf.distance_and_ranges(query, "bar", &mut ranges);
    ///
    /// assert!(distance.is_none());
    ///
    /// // The candidate wasn't a match, so `ranges` is still empty.
    /// assert!(ranges.is_empty());
    /// ```
    ///
    /// ```rust
    /// # use norm::fzf::{FzfV1, FzfParser};
    /// # use norm::Metric;
    /// let mut fzf = FzfV1::new();
    /// let mut parser = FzfParser::new();
    /// let mut ranges = Vec::new();
    ///
    /// let query = parser.parse("foo");
    ///
    /// let _ = fzf.distance_and_ranges(query, "seafood", &mut ranges);
    ///
    /// // There was a match, so the vector should now contain the byte range
    /// // of "foo" in "seafood".
    /// assert_eq!(ranges, [3..6]);
    ///
    /// let _ = fzf.distance_and_ranges(query, "fancy igloo", &mut ranges);
    ///
    /// // You can call `distance_and_ranges` multiple times with the same
    /// // vector, and it will keep appending to it.
    /// //
    /// // In this case, it appended the byte ranges of "f" and "oo" in
    /// // "fancy igloo".
    /// assert_eq!(ranges, [3..6, 0..1, 9..11]);
    /// ```
    ///
    /// ```rust
    /// # use norm::fzf::{FzfV1, FzfParser};
    /// # use norm::Metric;
    /// let mut fzf = FzfV1::new();
    /// let mut parser = FzfParser::new();
    /// let mut ranges = Vec::new();
    ///
    /// fzf.set_candidate_normalization(true);
    ///
    /// let query = parser.parse("foo");
    ///
    /// let _ = fzf.distance_and_ranges(query, "ƒöö", &mut ranges);
    ///
    /// // The start and end of each range are always byte offsets, not
    /// // character offsets.
    /// assert_eq!(ranges, [0..6]);
    /// ```
    fn distance_and_ranges(
        &mut self,
        query: Self::Query<'_>,
        candidate: &str,
        ranges_buf: &mut Vec<Range<usize>>,
    ) -> Option<Self::Distance>;
}
