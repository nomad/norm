/// TODO: docs
pub struct Match<D> {
    /// TODO: docs
    distance: D,
}

impl<D> Match<D> {
    /// TODO: docs
    #[inline]
    pub(crate) fn new(distance: D) -> Self {
        Self { distance }
    }
}
