use core::fmt::Debug;

use crate::Match;

/// TODO: docs.
pub trait Metric {
    /// TODO: docs.
    type Query<'a>: Debug;

    /// TODO: docs.
    type Distance: Debug + Ord;

    /// TODO: docs.
    fn distance(
        &self,
        query: Self::Query<'_>,
        candidate: &str,
    ) -> Option<Match<Self::Distance>>;
}
