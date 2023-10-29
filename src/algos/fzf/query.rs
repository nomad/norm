/// TODO: docs.
#[derive(Clone, Copy)]
pub struct FzfQuery<'a> {
    /// TODO: docs.
    chars: &'a [char],

    /// TODO: docs.
    is_ascii: bool,

    /// TODO: docs.
    has_uppercase: bool,
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
    pub(crate) fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.chars.iter().map(|&c| c as u8)
    }

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
    pub(crate) fn is_ascii(&self) -> bool {
        self.is_ascii
    }

    /// TODO: docs
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// TODO: docs
    #[inline]
    pub(crate) fn has_uppercase(&self) -> bool {
        self.has_uppercase
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new(chars: &'a [char]) -> Self {
        let is_ascii = chars.iter().all(char::is_ascii);
        let has_uppercase = chars.iter().copied().any(char::is_uppercase);
        Self { chars, is_ascii, has_uppercase }
    }
}
