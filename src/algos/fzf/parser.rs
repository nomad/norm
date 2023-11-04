use alloc::borrow::Cow;

use super::query::{Condition, FzfQuery, MatchType, Pattern};

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

        let mut char_len = 0;

        for ch in query.chars() {
            self.chars[char_len] = ch;
            char_len += 1;
        }

        let pattern = Pattern::new(&self.chars[..char_len], MatchType::Fuzzy);

        // SAFETY: todo.
        let pattern = unsafe {
            core::mem::transmute::<Pattern, Pattern<'static>>(pattern)
        };

        self.patterns[0] = pattern;

        let patterns = &self.patterns[..1];

        let condition = Condition::new(patterns);

        // SAFETY: todo.
        let condition = unsafe {
            core::mem::transmute::<Condition, Condition<'static>>(condition)
        };

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

/// TODO: docs
#[inline]
fn parse(query: &str) -> Vec<Condition<'static>> {
    let mut conditions = Vec::new();

    for word in Words::new(query) {
        let mut word = word.as_ref();

        let pattern_start;

        let mut match_type;

        match query.as_bytes()[0] {
            b'\'' => {
                pattern_start = 1;
                match_type = MatchType::Exact;
            },

            b'^' => {
                pattern_start = 1;
                match_type = MatchType::PrefixExact;
            },

            b'!' => {
                match query.as_bytes().get(1) {
                    Some(b'\'') => {
                        pattern_start = 2;
                        match_type = MatchType::InverseFuzzy;
                    },

                    Some(b'^') => {
                        pattern_start = 2;
                        match_type = MatchType::InversePrefixExact;
                    },

                    _ => {
                        pattern_start = 1;
                        match_type = MatchType::InverseExact;
                    },
                };
            },

            _ => {
                pattern_start = 0;
                match_type = MatchType::Fuzzy;
            },
        };

        word = &word[pattern_start..];

        let pattern_end;

        match memchr::memchr2(b' ', b'\\', word.as_bytes()) {
            Some(idx) if query.as_bytes()[idx] == b' ' => {
                todo!();
            },

            Some(idx) if query.as_bytes()[idx] == b'\\' => {
                todo!();
            },

            Some(_) => {
                unreachable!();
            },

            None => {
                pattern_end = query.len();
            },
        };

        let pattern_text = &query[..pattern_end];

        let pattern = {
            let pattern_text = pattern_text.chars().collect::<Vec<_>>().leak();
            Pattern::new(pattern_text, match_type)
        };

        {
            let or_patterns = vec![pattern].leak();
            let condition = Condition::new(or_patterns);
            conditions.push(condition);
        }
    }

    conditions
}

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
/// let mut words = Words::new("foo\ bar baz");
/// assert_eq!(words.next().as_deref(), Some("foo bar"));
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

        let mut word_end = 0;

        let mut bytes = self.s.as_bytes();

        loop {
            match memchr::memchr(b' ', bytes) {
                Some(offset) => {
                    if let Some(b'\\') = bytes.get(offset.wrapping_sub(1)) {
                        word_end += offset + 1;
                        bytes = &bytes[offset + 1..];
                    } else {
                        word_end += offset;
                        break;
                    }
                },

                None => {
                    word_end += bytes.len();
                    break;
                },
            }
        }

        let (word, rest) = self.s.split_at(word_end);

        self.s = strip_spaces(rest);

        Some(Cow::Borrowed(word))
    }
}

#[inline(always)]
fn strip_spaces(s: &str) -> &str {
    let leading_spaces = s.bytes().take_while(|&b| b == b' ').count();
    &s[leading_spaces..]
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
    fn words_multiple_words_space_escaped() {
        let mut words = Words::new("foo\\ bar\\ baz");
        assert_eq!(words.next().as_deref(), Some("foo bar baz"));
        assert_eq!(words.next(), None);
    }

    #[test]
    fn words_multiple_escaped_spaces() {
        let mut words = Words::new(" \\  foo \\ bar \\  ");
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next().as_deref(), Some("foo"));
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next().as_deref(), Some("bar"));
        assert_eq!(words.next().as_deref(), Some(" "));
        assert_eq!(words.next(), None);
    }

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
