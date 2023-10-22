use crate::Match;

/// TODO: docs.
pub trait Metric {
    /// TODO: docs.
    type Query<'a>;

    /// TODO: docs.
    type Distance: Copy + Ord;

    /// TODO: docs.
    fn distance(
        &mut self,
        query: Self::Query<'_>,
        candidate: &str,
    ) -> Option<Match<Self::Distance>>;
}
