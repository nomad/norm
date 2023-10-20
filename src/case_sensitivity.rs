/// TODO: docs
#[derive(Copy, Clone, Debug, Default)]
pub enum CaseSensitivity {
    /// TODO: docs
    Sensitive,

    /// TODO: docs
    Insensitive,

    /// TODO: docs
    #[default]
    Smart,
}

impl CaseSensitivity {
    /// TODO: docs
    #[inline]
    pub(crate) fn matcher(self, query: &str) -> CaseMatcher {
        let matcher = match self {
            Self::Sensitive => case_sensitive_eq,

            Self::Insensitive => case_insensitive_eq,

            Self::Smart => {
                if query.chars().any(|c| c.is_uppercase()) {
                    case_sensitive_eq
                } else {
                    case_insensitive_eq
                }
            },
        };

        CaseMatcher { matcher }
    }
}

/// TODO: docs
pub(crate) struct CaseMatcher {
    matcher: fn(char, char) -> bool,
}

impl CaseMatcher {
    /// TODO: docs
    pub(crate) fn eq(&self, query_char: char, candidate_char: char) -> bool {
        (self.matcher)(query_char, candidate_char)
    }
}

#[inline]
fn case_insensitive_eq(query: char, candidate: char) -> bool {
    query.eq_ignore_ascii_case(&candidate)
}

#[inline]
fn case_sensitive_eq(query: char, candidate: char) -> bool {
    query == candidate
}
