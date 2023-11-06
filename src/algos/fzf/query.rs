use core::fmt::Write;

use super::*;
use crate::*;

/// TODO: docs
type FuzzyAlgo<T> = fn(
    Pattern,
    &str,
    &Scheme,
    CharEq,
    bool,
    T,
) -> Option<(Score, MatchedRanges)>;

/// TODO: docs.
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
    pub(super) fn new(conditions: &'a [Condition<'a>]) -> Self {
        // If there's only one condition with a single pattern, and that
        // pattern is fuzzy, then we can use the non-extended search mode.
        if conditions.len() == 1 {
            let mut patterns = conditions[0].or_patterns();

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
    ) -> impl Iterator<Item = Pattern<'a>> + ExactSizeIterator + '_ {
        self.or_patterns.iter().copied()
    }
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub(super) struct Pattern<'a> {
    /// TODO: docs
    text: &'a [char],

    /// TODO: docs
    pub(super) byte_len: usize,

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
    pub(super) fn parse(mut text: &'a [char]) -> Self {
        debug_assert!(!text.is_empty());

        let first_char = text[0];

        let last_char = text[text.len() - 1];

        let mut is_inverse = false;

        let match_type;

        match first_char {
            '\'' => {
                text = &text[1..];
                match_type = MatchType::Exact;
            },

            '^' if last_char == '$' => {
                text = &text[1..text.len() - 1];
                match_type = MatchType::EqualExact;
            },

            '^' => {
                text = &text[1..];
                match_type = MatchType::PrefixExact;
            },

            '!' if text.get(1).copied() == Some('\'') => {
                text = &text[2..];
                match_type = MatchType::Fuzzy;
                is_inverse = true;
            },

            '!' if text.get(1).copied() == Some('^') => {
                text = &text[2..];
                match_type = MatchType::PrefixExact;
                is_inverse = true;
            },

            '!' if last_char == '$' => {
                text = &text[1..text.len() - 1];
                match_type = MatchType::SuffixExact;
                is_inverse = true;
            },

            '!' => {
                text = &text[1..];
                match_type = MatchType::Exact;
                is_inverse = true;
            },

            _ if last_char == '$' => {
                text = &text[..text.len() - 1];
                match_type = MatchType::SuffixExact;
            },

            _ => {
                match_type = MatchType::Fuzzy;
            },
        }

        let leading_spaces = text.iter().take_while(|&&c| c == ' ').count();

        let trailing_spaces =
            text.iter().rev().take_while(|&&c| c == ' ').count();

        Self {
            leading_spaces,
            trailing_spaces,
            byte_len: text.iter().copied().map(char::len_utf8).sum(),
            has_uppercase: text.iter().copied().any(char::is_uppercase),
            text,
            match_type,
            is_inverse,
        }
    }

    /// TODO: docs
    #[inline]
    pub(super) fn score<Extras>(
        self,
        candidate: &str,
        scheme: &Scheme,
        char_eq: CharEq,
        with_matched_ranges: bool,
        extras: Extras,
        fuzzy_algo: FuzzyAlgo<Extras>,
    ) -> Option<(Score, MatchedRanges)> {
        let result = match self.match_type {
            MatchType::Fuzzy => fuzzy_algo(
                self,
                candidate,
                scheme,
                char_eq,
                with_matched_ranges,
                extras,
            ),

            MatchType::Exact => exact_match(
                self,
                candidate,
                scheme,
                char_eq,
                with_matched_ranges,
            ),

            MatchType::PrefixExact => prefix_match(
                self,
                candidate,
                scheme,
                char_eq,
                with_matched_ranges,
            ),

            MatchType::SuffixExact => suffix_match(
                self,
                candidate,
                scheme,
                char_eq,
                with_matched_ranges,
            ),

            MatchType::EqualExact => equal_match(
                self,
                candidate,
                scheme,
                char_eq,
                with_matched_ranges,
            ),
        };

        match (result.is_some(), self.is_inverse) {
            (true, false) => result,

            (false, true) => Some((0, MatchedRanges::default())),

            _ => None,
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub(super) fn trailing_spaces(&self) -> usize {
        self.trailing_spaces
    }
}

/// TODO: docs
#[derive(Default, Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
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
