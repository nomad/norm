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
    /// distance value must indicate a closer or more relevant match.
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

    /// This method always returns the same value as [`Self::distance`], but in
    /// the case of a match it also fills the provided buffer with the **byte**
    /// ranges of the candidate that matched the query. If the candidate does
    /// not match the query, the buffer is left untouched.
    fn distance_and_ranges(
        &mut self,
        query: Self::Query<'_>,
        candidate: &str,
        ranges_buf: &mut Vec<Range<usize>>,
    ) -> Option<Self::Distance>;
}
