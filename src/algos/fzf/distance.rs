use core::cmp::{Ord, PartialOrd};

pub(super) type Score = i64;

/// TODO: docs
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FzfDistance(Score);

impl PartialOrd for FzfDistance {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FzfDistance {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Putting other first makes the type act like a distance and not like
        // a score.
        other.0.cmp(&self.0)
    }
}

impl Default for FzfDistance {
    #[inline]
    fn default() -> Self {
        Self::from_score(0)
    }
}

impl FzfDistance {
    /// TODO: docs
    #[inline(always)]
    pub(super) fn from_score(score: Score) -> Self {
        Self(score)
    }

    /// TODO: docs
    #[cfg(any(feature = "into-score", feature = "tests"))]
    #[inline(always)]
    pub fn into_score(self) -> Score {
        self.0
    }
}
