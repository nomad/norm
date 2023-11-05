use core::mem::transmute;

use super::query::{Condition, FzfQuery, Pattern};

/// TODO: docs
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
    /// TODO: docs
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

        FzfQuery::new(&self.conditions[..num_conditions])
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
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
        Self { buf, s: strip_leading_spaces(s), allocated: 0 }
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

        self.s = strip_leading_spaces(&self.s[word_byte_end..]);

        let word = &self.buf[prev_allocated..self.allocated];

        // SAFETY: todo
        let word = unsafe { transmute::<&[char], &'buf [char]>(word) };

        Some(word)
    }
}

/// TODO: docs
#[inline(always)]
fn strip_leading_spaces(s: &str) -> &str {
    let leading_spaces = s.bytes().take_while(|&b| b == b' ').count();
    &s[leading_spaces..]
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
    use super::super::query::MatchType;
    use super::*;

    #[test]
    fn parse_query_empty() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn parse_query_single_fuzzy() {
        let query = parse("foo");

        let conditions = query.conditions();

        assert_eq!(conditions.len(), 1);

        let condition = conditions.iter().next().unwrap();

        let mut patterns = condition.or_patterns();

        assert_eq!(patterns.len(), 1);

        let pattern = patterns.next().unwrap();

        assert_eq!(pattern.into_string(), "foo");

        assert_eq!(pattern.match_type, MatchType::Fuzzy);
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
