pub(super) type Score = i64;

pub(super) type Distance = Score;

/// TODO: docs
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct FzfDistance(Distance);

impl Default for FzfDistance {
    #[inline]
    fn default() -> Self {
        Self::from_score(0)
    }
}

impl FzfDistance {
    /// TODO: docs
    #[inline]
    pub(super) fn from_score(score: Score) -> Self {
        // The higher the score the lower the distance.
        Self(-score)
    }

    /// TODO: docs
    #[cfg(feature = "tests")]
    pub fn into_score(self) -> Score {
        -self.0
    }
}
