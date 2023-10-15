use crate::{Match, Metric};

/// TODO: docs
#[derive(Default)]
pub struct Fzf;

impl Fzf {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}

impl Metric for Fzf {
    type Distance = u64;

    #[inline]
    fn distance(
        &self,
        target: &str,
        candidate: &str,
    ) -> Option<Match<Self::Distance>> {
        todo!();
    }
}
