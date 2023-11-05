use alloc::borrow::Cow;

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
        let max_char_len = query.len();

        if max_char_len > self.chars.len() {
            self.chars.resize(max_char_len, char::default());
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

        // let mut char_len = 0;
        //
        // for ch in query.chars() {
        //     self.chars[char_len] = ch;
        //     char_len += 1;
        // }

        // // SAFETY: todo.
        // let pattern = unsafe {
        //     core::mem::transmute::<Pattern, Pattern<'static>>(pattern)
        // };
        //
        // // SAFETY: todo.
        // let condition = unsafe {
        //     core::mem::transmute::<Condition, Condition<'static>>(condition)
        // };

        let conditions = parse(query);

        FzfQuery::new(conditions.leak())
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

/// TODO: docs
#[inline]
fn parse(query: &str) -> Vec<Condition<'static>> {
    OrBlocks::new(query)
        .map(|or_block| {
            or_block
                .into_iter()
                .map(|pattern| pattern.chars().collect::<Vec<_>>().leak() as _)
                .map(Pattern::parse)
                .collect::<Vec<_>>()
                .leak() as _
        })
        .map(Condition::new)
        .collect::<Vec<_>>()
}

const OR_BLOCK_SEPARATOR: &str = "|";

/// TODO: docs
struct OrBlocks<'a> {
    /// TODO: docs
    words: Words<'a>,

    /// TODO: docs
    next: Option<<Words<'a> as Iterator>::Item>,
}

impl<'a> OrBlocks<'a> {
    #[inline]
    fn new(s: &'a str) -> Self {
        Self { words: Words::new(s), next: None }
    }
}

impl<'a> Iterator for OrBlocks<'a> {
    type Item = Vec<<Words<'a> as Iterator>::Item>;

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

impl core::iter::FusedIterator for OrBlocks<'_> {}

/// An iterator over the words of a string.
///
/// Here, a "word" is simply a string of consecutive non-ascii-space
/// characters. Escaped spaces are treated as non-space characters.
///
/// # Examples
///
/// ```rust
/// # use norm::algos::fzf::parser::Words;
/// let mut words = Words::new("foo 'bar' \"baz\"");
/// assert_eq!(words.next().as_deref(), Some("foo"));
/// assert_eq!(words.next().as_deref(), Some("'bar'"));
/// assert_eq!(words.next().as_deref(), Some("\"baz\""));
/// assert_eq!(words.next(), None);
/// ```
///
/// ```rust
/// # use norm::algos::fzf::parser::Words;
/// let mut words = Words::new("foo\\ bar baz");
/// assert_eq!(words.next().as_deref(), Some("foo bar"));
/// assert_eq!(words.next().as_deref(), Some("baz"));
/// assert_eq!(words.next(), None);
/// ```
///
/// ```rust
/// # use norm::algos::fzf::parser::Words;
/// let mut words = Words::new("foo \\ bar");
/// assert_eq!(words.next().as_deref(), Some("foo"));
/// assert_eq!(words.next().as_deref(), Some(" "));
/// assert_eq!(words.next().as_deref(), Some("baz"));
/// assert_eq!(words.next(), None);
/// ```
struct Words<'a> {
    s: &'a str,
}

impl<'a> Words<'a> {
    #[inline]
    fn new(s: &'a str) -> Self {
        Self { s: strip_spaces(s) }
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = Cow<'a, str>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.s.is_empty() {
            return None;
        }

        let mut word = Cow::Borrowed("");

        let mut word_end = 0;

        let mut s = self.s;

        loop {
            match memchr::memchr(b' ', s.as_bytes()) {
                Some(0) => break,

                Some(offset) if s.as_bytes()[offset - 1] == b'\\' => {
                    // The string starts with an escaped space. We don't have
                    // to allocate yet.
                    if offset == 1 && word.is_empty() {
                        word = Cow::Borrowed(" ");
                        word_end = 2;
                        s = &s[word_end..];
                        continue;
                    }

                    // This word includes an escaped space, so we have to
                    // allocate because we'll skip the escape.
                    let word = word.to_mut();

                    // Push everything up to (but not including) the escape.
                    word.push_str(&s[..offset - 1]);

                    // ..skip the escape..

                    // ..and push the space.
                    word.push(' ');

                    s = &s[offset + 1..];

                    word_end += offset + 1;
                },

                Some(offset) => {
                    let s = &s[..offset];
                    if word.is_empty() {
                        word = Cow::Borrowed(s);
                    } else {
                        word.to_mut().push_str(s);
                    }
                    word_end += s.len();
                    break;
                },

                None => {
                    if word.is_empty() {
                        word = Cow::Borrowed(s);
                    } else {
                        word.to_mut().push_str(s);
                    }
                    word_end += s.len();
                    break;
                },
            }
        }

        self.s = strip_spaces(&self.s[word_end..]);

        Some(word)
    }
}

impl core::iter::FusedIterator for Words<'_> {}

#[inline(always)]
fn strip_spaces(s: &str) -> &str {
    let leading_spaces = s.bytes().take_while(|&b| b == b' ').count();
    &s[leading_spaces..]
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
        let conditions = parse("foo");

        assert_eq!(conditions.len(), 1);

        let condition = conditions.into_iter().next().unwrap();

        let mut patterns = condition.or_patterns();

        assert_eq!(patterns.len(), 1);

        let pattern = patterns.next().unwrap();

        assert_eq!(pattern.into_string(), "foo");

        assert_eq!(pattern.match_type, MatchType::Fuzzy);
    }
}

#[cfg(test)]
mod or_blocks_tests {
    use super::*;

    #[test]
    fn or_blocks_empty() {
        let mut blocks = OrBlocks::new("");
        assert!(blocks.next().is_none());
    }

    #[test]
    fn or_blocks_single() {
        let mut blocks = OrBlocks::new("foo");
        assert_eq!(blocks.next().unwrap(), ["foo"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_multiple_ors() {
        let mut blocks = OrBlocks::new("foo | bar | baz");
        assert_eq!(blocks.next().unwrap(), ["foo", "bar", "baz"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_multiple_ands() {
        let mut blocks = OrBlocks::new("foo bar baz");
        assert_eq!(blocks.next().unwrap(), ["foo"]);
        assert_eq!(blocks.next().unwrap(), ["bar"]);
        assert_eq!(blocks.next().unwrap(), ["baz"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_empty_between_ors() {
        let mut blocks = OrBlocks::new("foo | | bar");
        assert_eq!(blocks.next().unwrap(), ["foo", "bar"]);
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn or_blocks_multiple_ors_multiple_ands() {
        let mut blocks = OrBlocks::new("foo | bar baz qux | quux | corge");
        assert_eq!(blocks.next().unwrap(), ["foo", "bar"]);
        assert_eq!(blocks.next().unwrap(), ["baz"]);
        assert_eq!(blocks.next().unwrap(), ["qux", "quux", "corge"]);
        assert_eq!(blocks.next(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_empty() {
        let mut words = Words::new("");
        assert!(words.next().is_none());
    }

    #[test]
    fn words_single() {
        let mut words = Words::new("foo");
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_escaped_escape_escaped_space() {
        let mut words = Words::new("\\\\ ");
        assert_eq!(words.next().as_deref(), Some("\\ "));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple() {
        let mut words = Words::new("foo bar");
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next().as_deref(), Some("bar"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple_leading_trailing_spaces() {
        let mut words = Words::new("   foo bar   ");
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next().as_deref(), Some("bar"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple_escaped_spaces() {
        let mut words = Words::new("foo\\ bar\\ baz");
        assert_eq!(words.next().as_deref(), Some("foo bar baz"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple_standalone_escaped_spaces() {
        let mut words = Words::new(" \\  foo \\ bar \\  ");
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next().as_deref(), Some(" bar"));
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_single_escaped_spaces() {
        let mut words = Words::new("\\ ");
        assert_eq!(words.next(), Some(Cow::Borrowed(" ")));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_consecutive_escaped_spaces() {
        let mut words = Words::new(" \\ \\ \\  ");
        assert_eq!(words.next().as_deref(), Some("   "));
        assert_eq!(words.next(), None);
    }
}
