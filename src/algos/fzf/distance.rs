use super::{Distance, Score};

/// TODO: docs
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct FzfDistance(Distance);

impl FzfDistance {
    /// TODO: docs
    #[inline]
    pub(super) fn from_score(score: Score) -> Self {
        // The higher the score the lower the distance.
        Self(Distance::MAX - score)
    }

    /// TODO: docs
    #[cfg(feature = "tests")]
    pub fn into_score(self) -> Score {
        // The higher the score the lower the distance.
        Distance::MAX - self.0
    }
}
