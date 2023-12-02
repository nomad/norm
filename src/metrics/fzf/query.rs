use core::fmt::Write;

/// A parsed fzf query.
///
/// This struct is created by the [`parse`](super::FzfParser::parse) method on
/// [`FzfParser`](super::FzfParser). See its documentation for more.
#[derive(Clone, Copy)]
pub struct FzfQuery<'a> {
    pub(super) search_mode: SearchMode<'a>,
}

/// TODO: docs
#[derive(Clone, Copy)]
pub(super) enum SearchMode<'a> {
    /// TODO: docs
    Extended(&'a [Condition<'a>]),

    /// TODO: docs
    NotExtended(Pattern<'a>),
}

impl core::fmt::Debug for FzfQuery<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self.search_mode {
            SearchMode::Extended(conditions) => conditions
                .iter()
                .map(|condition| format!("{:?}", condition))
                .collect::<Vec<_>>()
                .join(" && "),

            SearchMode::NotExtended(pattern) => pattern.into_string(),
        };

        f.debug_tuple("FzfQuery").field(&s).finish()
    }
}

impl<'a> FzfQuery<'a> {
    /// TODO: docs
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        match self.search_mode {
            SearchMode::Extended(conditions) => conditions.is_empty(),
            SearchMode::NotExtended(pattern) => pattern.is_empty(),
        }
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new_extended(conditions: &'a [Condition<'a>]) -> Self {
        // If there's only one condition with a single pattern, and that
        // pattern is fuzzy, then we can use the non-extended search mode.
        if conditions.len() == 1 {
            let mut patterns = conditions[0].iter();

            let first_pattern = patterns
                .next()
                .expect("conditions always have at least one pattern");

            if patterns.next().is_none()
                && matches!(first_pattern.match_type, MatchType::Fuzzy)
            {
                return Self {
                    search_mode: SearchMode::NotExtended(first_pattern),
                };
            }
        }

        Self { search_mode: SearchMode::Extended(conditions) }
    }

    /// TODO: docs
    #[inline]
    pub(super) fn new_not_extended(chars: &'a [char]) -> Self {
        Self { search_mode: SearchMode::NotExtended(Pattern::raw(chars)) }
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

                for (idx, pattern) in self.iter().enumerate() {
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
    #[cfg(test)]
    pub(super) fn or_patterns(&self) -> &'a [Pattern<'a>] {
        self.or_patterns
    }

    #[inline]
    pub(super) fn iter(
        &self,
    ) -> impl Iterator<Item = Pattern<'a>> + ExactSizeIterator + '_ {
        self.or_patterns.iter().copied()
    }

    #[inline]
    pub(super) fn new(or_patterns: &'a [Pattern<'a>]) -> Self {
        Self { or_patterns }
    }
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub(super) struct Pattern<'a> {
    /// TODO: docs
    text: &'a [char],

    /// Whether any of the characters in [`Self::text`] are uppercase.
    pub(super) has_uppercase: bool,

    /// TODO: docs
    pub(super) match_type: MatchType,

    /// TODO: docs
    pub(super) is_inverse: bool,

    /// TODO: docs
    pub(super) leading_spaces: usize,

    /// TODO: docs
    pub(super) trailing_spaces: usize,
}

impl core::fmt::Debug for Pattern<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.into_string().fmt(f)
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
    pub(super) fn into_string(self) -> String {
        self.text.iter().collect::<String>()
    }

    /// TODO: docs
    #[inline(always)]
    pub(super) fn leading_spaces(&self) -> usize {
        self.leading_spaces
    }

    /// TODO: docs
    #[inline]
    fn raw(text: &'a [char]) -> Self {
        let leading_spaces = text.iter().take_while(|&&c| c == ' ').count();

        let trailing_spaces =
            text.iter().rev().take_while(|&&c| c == ' ').count();

        Self {
            leading_spaces,
            trailing_spaces,
            has_uppercase: text.iter().copied().any(char::is_uppercase),
            text,
            match_type: MatchType::Fuzzy,
            is_inverse: false,
        }
    }

    /// TODO: docs
    #[inline]
    pub(super) fn parse(mut text: &'a [char]) -> Option<Self> {
        debug_assert!(!text.is_empty());

        let mut is_inverse = false;

        let mut match_type = MatchType::Fuzzy;

        if starts_with(text, '!') {
            is_inverse = true;
            match_type = MatchType::Exact;
            text = &text[1..];
        }

        if ends_with(text, '$') && text.len() > 1 {
            match_type = MatchType::SuffixExact;
            text = &text[..text.len() - 1];
        }

        if starts_with(text, '\'') {
            match_type =
                if !is_inverse { MatchType::Exact } else { MatchType::Fuzzy };

            text = &text[1..];
        } else if starts_with(text, '^') {
            match_type = if match_type == MatchType::SuffixExact {
                MatchType::EqualExact
            } else {
                MatchType::PrefixExact
            };

            text = &text[1..];
        }

        if text.is_empty() {
            return None;
        }

        let has_uppercase = text.iter().copied().any(char::is_uppercase);

        let leading_spaces = text.iter().take_while(|&&c| c == ' ').count();

        let trailing_spaces =
            text.iter().rev().take_while(|&&c| c == ' ').count();

        let this = Self {
            is_inverse,
            match_type,
            text,
            has_uppercase,
            leading_spaces,
            trailing_spaces,
        };

        Some(this)
    }

    /// TODO: docs
    #[inline(always)]
    pub(super) fn trailing_spaces(&self) -> usize {
        self.trailing_spaces
    }
}

#[inline(always)]
fn ends_with(haystack: &[char], needle: char) -> bool {
    haystack.last().copied() == Some(needle)
}

#[inline(always)]
fn starts_with(haystack: &[char], needle: char) -> bool {
    haystack.first().copied() == Some(needle)
}

/// TODO: docs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    EqualExact,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_parse_specials_1() {
        assert!(Pattern::parse(&['\'']).is_none());
        assert!(Pattern::parse(&['^']).is_none());
        assert!(Pattern::parse(&['!']).is_none());

        let pattern = Pattern::parse(&['$']).unwrap();
        assert_eq!(pattern.into_string(), "$");
        assert_eq!(pattern.match_type, MatchType::Fuzzy);
    }

    #[test]
    fn pattern_parse_specials_2() {
        assert!(Pattern::parse(&['!', '\'']).is_none());
        assert!(Pattern::parse(&['!', '^']).is_none());
        assert!(Pattern::parse(&['\'', '$']).is_none());
        assert!(Pattern::parse(&['^', '$']).is_none());

        let pattern = Pattern::parse(&['\'', '^']).unwrap();
        assert_eq!(pattern.into_string(), "^");
        assert_eq!(pattern.match_type, MatchType::Exact);

        let pattern = Pattern::parse(&['!', '$']).unwrap();
        assert_eq!(pattern.into_string(), "$");
        assert_eq!(pattern.match_type, MatchType::Exact);
        assert!(pattern.is_inverse);

        let pattern = Pattern::parse(&['!', '!']).unwrap();
        assert_eq!(pattern.into_string(), "!");
        assert_eq!(pattern.match_type, MatchType::Exact);
        assert!(pattern.is_inverse);

        let pattern = Pattern::parse(&['$', '$']).unwrap();
        assert_eq!(pattern.into_string(), "$");
        assert_eq!(pattern.match_type, MatchType::SuffixExact);
    }

    #[test]
    fn pattern_parse_specials_3() {
        assert!(Pattern::parse(&['!', '^', '$']).is_none());

        let pattern = Pattern::parse(&['\'', '^', '$']).unwrap();
        assert_eq!(pattern.into_string(), "^");
        assert_eq!(pattern.match_type, MatchType::Exact);

        let pattern = Pattern::parse(&['\'', '!', '$']).unwrap();
        assert_eq!(pattern.into_string(), "!");
        assert_eq!(pattern.match_type, MatchType::Exact);
    }

    #[test]
    fn pattern_parse_specials_4() {
        let pattern = Pattern::parse(&['\'', '^', '$', '$']).unwrap();
        assert_eq!(pattern.into_string(), "^$");
        assert_eq!(pattern.match_type, MatchType::Exact);
    }
}
