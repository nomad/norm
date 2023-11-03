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
fn parse(mut query: &str) -> Vec<Condition<'static>> {
    let mut conditions = Vec::new();

    while !query.is_empty() {
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

        query = &query[pattern_start..];

        let pattern_end;

        let next_pattern_start;

        match memchr::memchr2(b' ', b'\\', query.as_bytes()) {
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
                next_pattern_start = query.len();
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

        query = &query[next_pattern_start..];
    }

    conditions
}

#[cfg(test)]
mod tests {
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
