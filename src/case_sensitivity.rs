/// TODO: docs
#[derive(Copy, Clone, Debug, Default)]
pub enum CaseSensitivity {
    /// TODO: docs
    Sensitive,

    /// TODO: docs
    Insensitive,

    /// TODO: docs
    #[default]
    Smart,
}

impl CaseSensitivity {
    #[inline]
    pub(crate) fn is_sensitive(self) -> bool {
        matches!(self, Self::Sensitive)
    }

    #[inline]
    pub(crate) fn is_insensitive(self) -> bool {
        matches!(self, Self::Insensitive)
    }

    #[inline]
    pub(crate) fn is_smart(self) -> bool {
        matches!(self, Self::Smart)
    }
}
