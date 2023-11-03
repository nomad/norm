use core::fmt::Write;

use super::FzfQuery;

/// TODO: docs
#[derive(Default, Clone)]
pub struct FzfParser {
    /// TODO: docs
    chars: Vec<char>,

    /// TODO: docs
    patterns: Vec<Pattern<'static>>,

    /// TODO: docs
    conditions: Vec<Condition<'static>>,
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
pub(super) struct Condition<'a> {
    /// TODO: docs
    pub(super) or_patterns: &'a [Pattern<'a>],
}

impl core::fmt::Debug for Condition<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.or_patterns[..] {
            [] => Ok(()),

            [only] => only.to_string().fmt(f),

            _ => {
                f.write_char('(')?;

                let len = self.or_patterns.len();

                for (idx, pattern) in self.or_patterns().enumerate() {
                    let is_last = idx + 1 == len;

                    pattern.to_string().fmt(f)?;

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
    fn new(or_patterns: &'a [Pattern<'a>]) -> Self {
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
    pub(super) ty: PatternType,
}

impl core::fmt::Debug for Pattern<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Pattern").field(&self.to_string()).finish()
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
    pub(super) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new(text: &'a [char], ty: PatternType) -> Self {
        Self {
            has_uppercase: text.iter().copied().any(char::is_uppercase),
            text,
            ty,
        }
    }

    /// TODO: docs
    #[inline]
    fn to_string(&self) -> String {
        self.text.iter().collect::<String>()
    }
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
pub(super) enum PatternType {
    /// TODO: docs
    #[default]
    FuzzyMatch,

    /// TODO: docs
    ExactMatch,

    /// TODO: docs
    PrefixExactMatch,

    /// TODO: docs
    SuffixExactMatch,

    /// TODO: docs
    InverseExactMatch,

    /// TODO: docs
    InversePrefixExactMatch,

    /// TODO: docs
    InverseSuffixExactMatch,
}

impl FzfParser {
    /// TODO: docs
    #[inline]
    pub fn parse<'a>(&'a mut self, query: &str) -> FzfQuery<'a> {
        // When parsing, the `|` operator has precedence over the `space`
        // operator.
        //
        // `|` acts like multiplication, and `space` acts like addition.

        if query.len() > self.chars.len() {
            self.chars.resize(query.len(), char::default());
        }

        if query.is_empty() {
            return FzfQuery::new(&[]);
        }

        let mut char_len = 0;

        for ch in query.chars() {
            self.chars[char_len] = ch;
            char_len += 1;
        }

        let pattern =
            Pattern::new(&self.chars[..char_len], PatternType::FuzzyMatch);

        // SAFETY: todo.
        let pattern = unsafe {
            core::mem::transmute::<Pattern, Pattern<'static>>(pattern)
        };

        if self.patterns.is_empty() {
            self.patterns.resize(1, Pattern::default());
        }

        self.patterns[0] = pattern;

        let patterns = &self.patterns[..1];

        let condition = Condition::new(patterns);

        // SAFETY: todo.
        let condition = unsafe {
            core::mem::transmute::<Condition, Condition<'static>>(condition)
        };

        if self.conditions.is_empty() {
            self.conditions.resize(1, Condition::default());
        }

        self.conditions[0] = condition;

        let conditions = &self.conditions[..1];

        FzfQuery::new(conditions)
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}
