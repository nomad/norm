/// TODO: docs.
#[derive(Clone, Copy)]
pub struct FzfQuery<'a> {
    /// TODO: docs.
    chars: &'a [char],
}

impl core::fmt::Debug for FzfQuery<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self.chars.iter().collect::<String>();
        f.debug_tuple("FzfQuery").field(&s).finish()
    }
}

impl<'a> FzfQuery<'a> {
    /// TODO: docs
    #[inline]
    pub(super) fn char_len(&self) -> usize {
        self.chars.len()
    }

    /// TODO: docs
    #[inline]
    pub(crate) fn chars(
        &self,
    ) -> impl Iterator<Item = char> + DoubleEndedIterator + '_ {
        self.chars.iter().copied()
    }

    /// TODO: docs
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new(chars: &'a [char]) -> Self {
        Self { chars }
    }
}