pub(super) type Score = i64;

/// The fzf distance type.
///
/// This struct is returned by [`FzfV1`](super::FzfV1) and
/// [`FzfV2`](super::FzfV2)'s [`Metric`](crate::Metric) implementations.
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
        // This makes the type act like a distance and not like a score.
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
    /// Creates a new [`FzfDistance`] from a score.
    #[inline(always)]
    pub(super) fn from_score(score: Score) -> Self {
        Self(score)
    }

    /// Returns a score representation of the distance.
    ///
    /// This is not part of the public API and should not be relied upon.
    ///
    /// It's only used internally for testing and debugging purposes.
    #[cfg(any(feature = "__into-score", feature = "__tests"))]
    #[inline(always)]
    pub fn into_score(self) -> Score {
        self.0
    }
}
