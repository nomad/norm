use core::fmt::Write;

/// TODO: docs.
#[derive(Clone, Copy)]
pub struct FzfQuery<'a> {
    conditions: &'a [Condition<'a>],
}

impl core::fmt::Debug for FzfQuery<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = self
            .conditions
            .iter()
            .map(|condition| format!("{:?}", condition))
            .collect::<Vec<_>>()
            .join(" && ");

        f.debug_tuple("FzfQuery").field(&s).finish()
    }
}

impl<'a> FzfQuery<'a> {
    /// TODO: docs
    #[inline(always)]
    pub(super) fn conditions(&self) -> &[Condition<'a>] {
        self.conditions
    }

    /// TODO: docs
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new(conditions: &'a [Condition<'a>]) -> Self {
        Self { conditions }
    }
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
pub(super) struct Condition<'a> {
    /// TODO: docs
    pub(super) or_patterns: &'a [Pattern<'a>],
}

impl core::fmt::Debug for Condition<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.or_patterns {
            [] => Ok(()),

            [pattern] => pattern.into_string().fmt(f),

            _ => {
                f.write_char('(')?;

                let len = self.or_patterns.len();

                for (idx, pattern) in self.or_patterns().enumerate() {
                    let is_last = idx + 1 == len;

                    pattern.into_string().fmt(f)?;

                    if !is_last {
                        f.write_str(" || ")?;
                    }
                }

                f.write_char(')')
            },
        }
    }
}

impl<'a> Condition<'a> {
    #[inline]
    pub(super) fn new(or_patterns: &'a [Pattern<'a>]) -> Self {
        Self { or_patterns }
    }

    #[inline]
    pub(super) fn or_patterns(
        &self,
    ) -> impl Iterator<Item = Pattern<'a>> + '_ {
        self.or_patterns.iter().copied()
    }
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
pub(super) struct Pattern<'a> {
    /// TODO: docs
    text: &'a [char],

    /// Whether any of the characters in [`Self::text`] are uppercase.
    pub(super) has_uppercase: bool,

    /// TODO: docs
    pub(super) match_type: MatchType,
}

impl core::fmt::Debug for Pattern<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Pattern").field(&self.into_string()).finish()
    }
}

impl<'a> Pattern<'a> {
    /// TODO: docs
    #[inline(always)]
    pub(super) fn char_len(&self) -> usize {
        self.text.len()
    }

    /// TODO: docs
    #[inline(always)]
    pub(super) fn char(&self, idx: usize) -> char {
        self.text[idx]
    }

    /// TODO: docs
    #[inline]
    pub(crate) fn chars(
        &self,
    ) -> impl Iterator<Item = char> + DoubleEndedIterator + '_ {
        self.text.iter().copied()
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new(text: &'a [char], ty: MatchType) -> Self {
        Self {
            has_uppercase: text.iter().copied().any(char::is_uppercase),
            text,
            match_type: ty,
        }
    }

    /// TODO: docs
    #[inline]
    fn into_string(self) -> String {
        self.text.iter().collect::<String>()
    }
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
pub(super) enum MatchType {
    /// TODO: docs
    #[default]
    Fuzzy,

    /// TODO: docs
    Exact,

    /// TODO: docs
    PrefixExact,

    /// TODO: docs
    SuffixExact,

    /// TODO: docs
    InverseExact,

    /// TODO: docs
    InversePrefixExact,

    /// TODO: docs
    InverseSuffixExact,
}
