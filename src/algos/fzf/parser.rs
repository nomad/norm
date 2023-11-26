use core::mem::transmute;

use super::query::{Condition, FzfQuery, Pattern};
use crate::utils;

/// The parser used to parse strings into [`FzfQuery`]s.
///
/// Queries can be parsed according to fzf's [extended-search mode][esm] via
/// [`parse`][FzfParser::parse]. If this is not desired, use
/// [`parse_not_extended`][FzfParser::parse_not_extended] instead.
///
/// [esm]: https://github.com/junegunn/fzf#search-syntax
#[derive(Clone)]
pub struct FzfParser {
    /// TODO: docs
    chars: Vec<char>,

    /// TODO: docs
    patterns: Vec<Pattern<'static>>,

    /// TODO: docs
    conditions: Vec<Condition<'static>>,
}

impl Default for FzfParser {
    #[inline]
    fn default() -> Self {
        Self {
            chars: vec![char::default(); 64],
            patterns: vec![Pattern::default(); 64],
            conditions: vec![Condition::default(); 64],
        }
    }
}

impl core::fmt::Debug for FzfParser {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FzfParser").finish_non_exhaustive()
    }
}

impl FzfParser {
    /// Creates a new `FzfParser`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses the given query string according to fzf's
    /// [extended-search mode][esm].
    ///
    /// In extended-search mode certain characters change how the query is
    /// matched in candidates.
    ///
    /// In particular:
    ///
    /// | Pattern | Matches                                      |
    /// | ------- | -------------------------------------------- |
    /// | `foo`   | candidates that fuzzy-match `"foo"`          |
    /// | `'foo`  | candidates that include `"foo"`              |
    /// | `^foo`  | candidates that start with `"foo"`           |
    /// | `foo$`  | candidates that end with `"foo"`             |
    /// | `!foo`  | candidates that **don't** include `"foo"`    |
    /// | `!^foo` | candidates that **don't** start with `"foo"` |
    /// | `!foo$` | candidates that **don't**  end with `"foo"`  |
    ///
    /// It's also possible to query for multiple patterns by separating them
    /// with spaces or with the pipe character `"|"`. A space acts as a logical
    /// AND operator, while a pipe character acts as a logical OR operator.
    ///
    /// For example, the query `"^main .c$ | .rs$"` would only match candidates
    /// that start with `"main"` and end with either `".c"` or `".rs"`.
    /// Spaces can be escaped with a backslash if they're part of a pattern,
    /// e.g. `"foo\ baz"` will match `"foo bar baz"` but not `"baz foo"`.
    ///
    /// Note that like in fzf, but unlike in logical expressions, the pipe
    /// character (OR) has a higher precedence than the space character (AND),
    /// so that `"foo bar | baz"` gets parsed as `"foo && (bar || baz)"`, and
    /// **not** as `"(foo && bar) || baz"`;
    ///
    /// If you want to treat all the characters in the query as fuzzy-matching,
    /// use [`parse_not_extended`][FzfParser::parse_not_extended] instead.
    ///
    /// [esm]: https://github.com/junegunn/fzf#search-syntax
    #[inline]
    pub fn parse<'a>(&'a mut self, query: &str) -> FzfQuery<'a> {
        let max_chars = query.len();

        if self.chars.len() < max_chars {
            self.chars.resize(max_chars, char::default());
        }

        // The theoretical maximum number of conditions that could be included
        // in the query.
        //
        // The actual number of conditions (which we'll only know after
        // parsing) matches this maximum on space-separated queries of
        // multiple ascii characters, e.g. `a b c d`.
        let max_conditions = query.len() / 2 + 1;

        if self.conditions.len() < max_conditions {
            self.conditions.resize(max_conditions, Condition::default());
        }

        if self.patterns.len() < max_conditions {
            self.patterns.resize(max_conditions, Pattern::default());
        }

        let patterns: &'a mut [Pattern<'static>] =
            self.patterns.as_mut_slice();

        // SAFETY: todo.
        let patterns = unsafe {
            transmute::<&'a mut [Pattern<'static>], &'a mut [Pattern<'a>]>(
                patterns,
            )
        };

        let mut num_conditions = 0;

        for condition in
            Patterns::new(patterns, &mut self.chars, query).map(Condition::new)
        {
            // SAFETY: todo
            let condition = unsafe {
                transmute::<Condition, Condition<'static>>(condition)
            };

            self.conditions[num_conditions] = condition;

            num_conditions += 1;
        }

        FzfQuery::new_extended(&self.conditions[..num_conditions])
    }

    /// Parses the given query string without using fzf's extended-search mode.
    ///
    /// All the characters in the query string are used for fuzzy-matching,
    /// with no special meaning attached to any of them. This should be
    /// equivalent to calling `fzf` with the `--no-extended` flag.
    ///
    /// If you want to apply fzf's extended-search mode to the query, parse it
    /// with [`parse`][FzfParser::parse] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use norm::fzf::{FzfParser, FzfV2};
    /// # use norm::Metric;
    /// let mut fzf = FzfV2::new();
    /// let mut parser = FzfParser::new();
    /// let mut ranges = Vec::new();
    ///
    /// let query = parser.parse_not_extended("^bar | baz$");
    ///
    /// let distance =
    ///     fzf.distance_and_ranges(query, "^foo bar | baz $ foo", &mut ranges);
    ///
    /// // We expect a match since the characters in the query fuzzy-match the
    /// // candidate.
    /// //
    /// // If we parsed the query by calling `parse` there wouldn't have been a
    /// // match since the candidate doesn't start with `"bar"` nor does it end
    /// // with `"baz"`.
    /// assert!(distance.is_some());
    ///
    /// assert_eq!(ranges, [0..1, 5..14, 15..16]);
    /// ```
    #[inline]
    pub fn parse_not_extended<'a>(&'a mut self, query: &str) -> FzfQuery<'a> {
        let mut char_len = 0;

        for ch in query.chars() {
            self.chars[char_len] = ch;
            char_len += 1;
        }

        FzfQuery::new_not_extended(&self.chars[..char_len])
    }
}

const OR_BLOCK_SEPARATOR: &[char] = &['|'];

/// TODO: docs
struct Patterns<'buf, 's> {
    /// TODO: docs
    buf: &'buf mut [Pattern<'buf>],

    /// TODO: docs
    allocated: usize,

    /// TODO: docs
    words: Words<'buf, 's>,

    /// TODO: docs
    next: Option<Pattern<'buf>>,
}

impl<'buf, 's> Patterns<'buf, 's> {
    #[inline]
    fn alloc(&mut self, pattern: Pattern<'buf>) {
        self.buf[self.allocated] = pattern;
        self.allocated += 1;
    }

    #[inline]
    fn new(
        patterns_buf: &'buf mut [Pattern<'buf>],
        char_buf: &'buf mut [char],
        s: &'s str,
    ) -> Self {
        Self {
            buf: patterns_buf,
            allocated: 0,
            words: Words::new(char_buf, s),
            next: None,
        }
    }
}

impl<'buf, 's> Iterator for Patterns<'buf, 's> {
    type Item = &'buf [Pattern<'buf>];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let prev_allocated = self.allocated;

        // Whether we're expecting the next word yielded by `self.words` to be
        // a "|". This is set to true after getting a word, and set to false
        // after a "|".
        let mut looking_for_or;

        if let Some(first_pattern) = self.next.take() {
            self.alloc(first_pattern);
            looking_for_or = true;
        } else {
            looking_for_or = false;
        }

        loop {
            let Some(word) = self.words.next() else {
                break;
            };

            let word_is_condition = word != OR_BLOCK_SEPARATOR;

            if word_is_condition {
                let word = Pattern::parse(word);

                if looking_for_or {
                    self.next = Some(word);
                    break;
                } else {
                    self.alloc(word);
                    looking_for_or = true;
                    continue;
                }
            }

            looking_for_or = false;
        }

        if self.allocated == prev_allocated {
            return None;
        }

        let patterns = &self.buf[prev_allocated..self.allocated];

        // SAFETY: todo
        let patterns =
            unsafe { transmute::<&[Pattern], &'buf [Pattern]>(patterns) };

        Some(patterns)
    }
}

/// An iterator over the words of a string.
///
/// Here, a "word" is simply a string of consecutive non-ascii-space
/// characters. Escaped spaces are treated as non-space characters.
///
/// # Examples
///
/// ```rust
/// # use norm::fzf::words;
/// let mut words = words("foo 'bar' \"baz\"");
/// assert_eq!(words.next().as_deref(), Some("foo"));
/// assert_eq!(words.next().as_deref(), Some("'bar'"));
/// assert_eq!(words.next().as_deref(), Some("\"baz\""));
/// assert_eq!(words.next(), None);
/// ```
///
/// ```rust
/// # use norm::fzf::words;
/// let mut words = words("foo\\ bar baz");
/// assert_eq!(words.next().as_deref(), Some("foo bar"));
/// assert_eq!(words.next().as_deref(), Some("baz"));
/// assert_eq!(words.next(), None);
/// ```
///
/// ```rust
/// # use norm::fzf::words;
/// let mut words = words("foo \\ bar");
/// assert_eq!(words.next().as_deref(), Some("foo"));
/// assert_eq!(words.next().as_deref(), Some(" bar"));
/// assert_eq!(words.next(), None);
/// ```
#[doc(hidden)]
pub struct Words<'buf, 'sentence> {
    /// TODO: docs
    buf: &'buf mut [char],

    /// TODO: docs
    allocated: usize,

    /// TODO: docs
    s: &'sentence str,
}

impl<'buf, 'sentence> Words<'buf, 'sentence> {
    /// TODO: docs
    #[inline]
    fn alloc(&mut self, s: &str) {
        for ch in s.chars() {
            self.buf[self.allocated] = ch;
            self.allocated += 1;
        }
    }

    /// TODO: docs
    #[inline]
    fn new(buf: &'buf mut [char], s: &'sentence str) -> Self {
        Self { buf, s: utils::strip_leading_spaces(s), allocated: 0 }
    }
}

impl<'buf> Iterator for Words<'buf, '_> {
    type Item = &'buf [char];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.s.is_empty() {
            return None;
        }

        let prev_allocated = self.allocated;

        let mut word_byte_end = 0;

        let mut s = self.s;

        loop {
            match memchr::memchr(b' ', s.as_bytes()) {
                Some(0) => break,

                Some(offset) if s.as_bytes()[offset - 1] == b'\\' => {
                    // Push everything up to (but not including) the escape.
                    self.alloc(&s[..offset - 1]);

                    // ..skip the escape..

                    // ..and push the space.
                    self.alloc(" ");

                    s = &s[offset + 1..];

                    word_byte_end += offset + 1;
                },

                Some(offset) => {
                    let s = &s[..offset];
                    self.alloc(s);
                    word_byte_end += s.len();
                    break;
                },

                None => {
                    self.alloc(s);
                    word_byte_end += s.len();
                    break;
                },
            }
        }

        self.s = utils::strip_leading_spaces(&self.s[word_byte_end..]);

        let word = &self.buf[prev_allocated..self.allocated];

        // SAFETY: todo
        let word = unsafe { transmute::<&[char], &'buf [char]>(word) };

        Some(word)
    }
}

/// TODO: docs
#[cfg(feature = "tests")]
#[doc(hidden)]
pub fn parse(s: &str) -> FzfQuery<'static> {
    let parser = Box::leak(Box::new(FzfParser::new()));
    parser.parse(s)
}

#[cfg(test)]
mod parse_tests {
    use super::super::query::*;
    use super::*;

    #[test]
    fn parse_query_empty() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn parse_query_single_fuzzy() {
        let query = parse("foo");

        let SearchMode::NotExtended(pattern) = query.search_mode else {
            panic!();
        };

        assert_eq!(pattern.into_string(), "foo");

        assert_eq!(pattern.match_type, MatchType::Fuzzy);
    }

    #[test]
    fn parse_query_upstream_extended() {
        let query = parse(
            "aaa 'bbb ^ccc ddd$ !eee !'fff !^ggg !hhh$ | ^iii$ ^xxx | 'yyy | \
             zzz$ | !ZZZ |",
        );

        let SearchMode::Extended(conditions) = query.search_mode else {
            panic!();
        };

        assert_eq!(conditions.len(), 9);

        let pattern = conditions[0].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::Fuzzy);
        assert!(!pattern.is_inverse);

        let pattern = conditions[1].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::Exact);
        assert!(!pattern.is_inverse);

        let pattern = conditions[2].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::PrefixExact);
        assert!(!pattern.is_inverse);

        let pattern = conditions[3].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::SuffixExact);
        assert!(!pattern.is_inverse);

        let pattern = conditions[4].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::Exact);
        assert!(pattern.is_inverse);

        let pattern = conditions[5].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::Fuzzy);
        assert!(pattern.is_inverse);

        let pattern = conditions[6].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::PrefixExact);
        assert!(pattern.is_inverse);

        let pattern = conditions[7].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::SuffixExact);
        assert!(pattern.is_inverse);

        let pattern = conditions[7].or_patterns()[1];
        assert_eq!(pattern.match_type, MatchType::EqualExact);
        assert!(!pattern.is_inverse);

        let pattern = conditions[8].or_patterns()[0];
        assert_eq!(pattern.match_type, MatchType::PrefixExact);
        assert!(!pattern.is_inverse);

        let pattern = conditions[8].or_patterns()[1];
        assert_eq!(pattern.match_type, MatchType::Exact);
        assert!(!pattern.is_inverse);

        let pattern = conditions[8].or_patterns()[2];
        assert_eq!(pattern.match_type, MatchType::SuffixExact);
        assert!(!pattern.is_inverse);

        let pattern = conditions[8].or_patterns()[3];
        assert_eq!(pattern.match_type, MatchType::Exact);
        assert!(pattern.is_inverse);
    }
}

#[cfg(test)]
mod patterns_tests {
    use super::*;

    fn patterns(
        s: &str,
    ) -> impl Iterator<Item = &'static [Pattern<'static>]> + '_ {
        let patterns_buf = vec![Pattern::default(); s.len() / 2 + 1].leak();
        let char_buf = vec![char::default(); s.len()].leak();
        Patterns::new(patterns_buf, char_buf, s)
    }

    fn pattern(s: &str) -> Pattern<'static> {
        Pattern::parse(s.chars().collect::<Vec<_>>().leak())
    }

    #[test]
    fn patterns_empty() {
        let mut blocks = patterns("");
        assert!(blocks.next().is_none());
    }

    #[test]
    fn patterns_single() {
        let mut blocks = patterns("foo");
        assert_eq!(blocks.next().unwrap(), [pattern("foo")]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn patterns_multiple_ors() {
        let mut blocks = patterns("foo | bar | baz");
        assert_eq!(
            blocks.next().unwrap(),
            [pattern("foo"), pattern("bar"), pattern("baz")]
        );
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn patterns_multiple_ands() {
        let mut blocks = patterns("foo bar baz");
        assert_eq!(blocks.next().unwrap(), [pattern("foo")]);
        assert_eq!(blocks.next().unwrap(), [pattern("bar")]);
        assert_eq!(blocks.next().unwrap(), [pattern("baz")]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn patterns_empty_between_ors() {
        let mut blocks = patterns("foo | | bar");
        assert_eq!(blocks.next().unwrap(), [pattern("foo"), pattern("bar")]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn patterns_multiple_ors_multiple_ands() {
        let mut blocks = patterns("foo | bar baz qux | quux | corge");
        assert_eq!(blocks.next().unwrap(), [pattern("foo"), pattern("bar")]);
        assert_eq!(blocks.next().unwrap(), [pattern("baz")]);
        assert_eq!(
            blocks.next().unwrap(),
            [pattern("qux"), pattern("quux"), pattern("corge")]
        );
        assert_eq!(blocks.next(), None);
    }
}

#[cfg(feature = "tests")]
#[doc(hidden)]
pub fn words(s: &str) -> impl Iterator<Item = String> {
    let mut buf = Vec::new();

    buf.resize(s.len(), char::default());

    Words::new(&mut buf, s)
        .map(String::from_iter)
        .collect::<Vec<_>>()
        .into_iter()
}

#[cfg(test)]
mod word_tests {
    use super::*;

    #[test]
    fn words_empty() {
        let mut words = words("");
        assert!(words.next().is_none());
    }

    #[test]
    fn words_single() {
        let mut words = words("foo");
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_escaped_escape_escaped_space() {
        let mut words = words("\\\\ ");
        assert_eq!(words.next().as_deref(), Some("\\ "));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple() {
        let mut words = words("foo bar");
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next().as_deref(), Some("bar"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple_leading_trailing_spaces() {
        let mut words = words("   foo bar   ");
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next().as_deref(), Some("bar"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple_escaped_spaces() {
        let mut words = words("foo\\ bar\\ baz");
        assert_eq!(words.next().as_deref(), Some("foo bar baz"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple_standalone_escaped_spaces() {
        let mut words = words(" \\  foo \\ bar \\  ");
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next().as_deref(), Some(" bar"));
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_single_escaped_spaces() {
        let mut words = words("\\ ");
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_consecutive_escaped_spaces() {
        let mut words = words(" \\ \\ \\  ");
        assert_eq!(words.next().as_deref(), Some("   "));
        assert_eq!(words.next(), None);
    }
}
