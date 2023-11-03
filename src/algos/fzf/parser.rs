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
