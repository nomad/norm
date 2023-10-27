/// TODO: docs.
#[derive(Clone, Copy, Debug)]
pub struct FzfQuery<'a> {
    /// TODO: docs.
    raw: &'a str,
}

impl<'a> FzfQuery<'a> {
    /// TODO: docs
    #[inline]
    pub fn new(s: &'a str) -> Self {
        Self { raw: s }
    }

    /// TODO: docs
    #[inline]
    pub(super) fn raw(&self) -> &'a str {
        self.raw
    }
}
