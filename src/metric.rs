use core::fmt::Debug;

use crate::Match;

/// TODO: docs.
pub trait Metric {
    /// TODO: docs.
    type Distance: Debug + Ord;

    /// TODO: docs.
    fn distance(
        &self,
        target: &str,
        candidate: &str,
    ) -> Option<Match<Self::Distance>>;
}
