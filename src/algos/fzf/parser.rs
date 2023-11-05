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

impl FzfParser {
    /// TODO: docs
    #[inline]
    pub fn parse<'a>(&'a mut self, query: &str) -> FzfQuery<'a> {
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

        // // SAFETY: todo.
        // let pattern = unsafe {
        //     core::mem::transmute::<Pattern, Pattern<'static>>(pattern)
        // };
        //
        // // SAFETY: todo.
        // let condition = unsafe {
        //     core::mem::transmute::<Condition, Condition<'static>>(condition)
        // };

        let conditions = OrBlocks::new(&mut self.chars, query)
            .map(|or_block| {
                or_block
                    .into_iter()
                    .map(Pattern::parse)
                    .collect::<Vec<_>>()
                    .leak() as _
            })
            .map(Condition::new)
            .collect::<Vec<_>>();

        FzfQuery::new(conditions.leak())
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

const OR_BLOCK_SEPARATOR: &[char] = &['|'];

/// TODO: docs
struct OrBlocks<'buf, 's> {
    /// TODO: docs
    words: Words<'buf, 's>,

    /// TODO: docs
    next: Option<<Words<'buf, 's> as Iterator>::Item>,
}

impl<'buf, 's> OrBlocks<'buf, 's> {
    #[inline]
    fn new(buf: &'buf mut Vec<char>, s: &'s str) -> Self {
        Self { words: Words::new(buf, s), next: None }
    }
}

impl<'buf, 's> Iterator for OrBlocks<'buf, 's> {
    type Item = Vec<<Words<'buf, 's> as Iterator>::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut blocks;

        // Whether we're expecting the next word yielded by `self.words` to be
        // a "|". This is set to true after getting a word, and set to false
        // after a "|".
        let mut looking_for_or;

        if let Some(first_block) = self.next.take() {
            blocks = vec![first_block];
            looking_for_or = true;
        } else {
            blocks = Vec::new();
            looking_for_or = false;
        }

        loop {
            let Some(word) = self.words.next() else {
                break;
            };

            let word_is_condition = word != OR_BLOCK_SEPARATOR;

            if word_is_condition {
                if looking_for_or {
                    self.next = Some(word);
                    break;
                } else {
                    blocks.push(word);
                    looking_for_or = true;
                    continue;
                }
            }

            looking_for_or = false;
        }

        (!blocks.is_empty()).then_some(blocks)
    }
}

impl core::iter::FusedIterator for OrBlocks<'_, '_> {}

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
    buf: &'buf mut Vec<char>,

    /// TODO: docs
    allocated: usize,

    /// TODO: docs
    s: &'sentence str,
}

impl<'buf, 'sentence> Words<'buf, 'sentence> {
    /// TODO: docs
    #[inline]
    fn alloc(&mut self, s: &str) {
        let buf = &mut self.buf[self.allocated..];

        let mut char_len = 0;

        for ch in s.chars() {
            buf[char_len] = ch;
            char_len += 1;
        }

        self.allocated += char_len;
    }

    /// TODO: docs
    #[inline]
    fn new(buf: &'buf mut Vec<char>, s: &'sentence str) -> Self {
        let max_yielded_char_len = s.len();

        if buf.len() < max_yielded_char_len {
            buf.resize(max_yielded_char_len, char::default());
        }

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

impl core::iter::FusedIterator for Words<'_, '_> {}

/// TODO: docs
#[inline(always)]
fn strip_leading_spaces(s: &str) -> &str {
    let leading_spaces = s.bytes().take_while(|&b| b == b' ').count();
    &s[leading_spaces..]
}

/// TODO: docs
#[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
#[doc(hidden)]
pub fn or_blocks(s: &str) -> impl Iterator<Item = Vec<String>> {
    let mut buf = Vec::new();

    OrBlocks::new(&mut buf, s)
        .map(|blocks| {
            blocks.into_iter().map(String::from_iter).collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter()
}

#[cfg(test)]
mod or_blocks_tests {
    use super::*;

    #[test]
    fn or_blocks_empty() {
        let mut blocks = or_blocks("");
        assert!(blocks.next().is_none());
    }

    #[test]
    fn or_blocks_single() {
        let mut blocks = or_blocks("foo");
        assert_eq!(blocks.next().unwrap(), ["foo"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_multiple_ors() {
        let mut blocks = or_blocks("foo | bar | baz");
        assert_eq!(blocks.next().unwrap(), ["foo", "bar", "baz"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_multiple_ands() {
        let mut blocks = or_blocks("foo bar baz");
        assert_eq!(blocks.next().unwrap(), ["foo"]);
        assert_eq!(blocks.next().unwrap(), ["bar"]);
        assert_eq!(blocks.next().unwrap(), ["baz"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_empty_between_ors() {
        let mut blocks = or_blocks("foo | | bar");
        assert_eq!(blocks.next().unwrap(), ["foo", "bar"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_multiple_ors_multiple_ands() {
        let mut blocks = or_blocks("foo | bar baz qux | quux | corge");
        assert_eq!(blocks.next().unwrap(), ["foo", "bar"]);
        assert_eq!(blocks.next().unwrap(), ["baz"]);
        assert_eq!(blocks.next().unwrap(), ["qux", "quux", "corge"]);
        assert_eq!(blocks.next(), None);
    }
}

#[cfg(debug_assertions)]
#[doc(hidden)]
pub fn words(s: &str) -> impl Iterator<Item = String> {
    let mut buf = Vec::new();

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
