use crate::fzf::FzfQuery;

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
    pub(crate) fn matcher(self, query: FzfQuery) -> CaseMatcher {
        match self {
            Self::Sensitive => case_sensitive_eq,

            Self::Insensitive => case_insensitive_eq,

            Self::Smart => {
                if query.chars().any(char::is_uppercase) {
                    case_sensitive_eq
                } else {
                    case_insensitive_eq
                }
            },
        }
    }
}

/// TODO: docs
pub(crate) type CaseMatcher = fn(char, char) -> bool;

#[inline]
fn case_insensitive_eq(query: char, candidate: char) -> bool {
    query.eq_ignore_ascii_case(&candidate)
}

#[inline]
fn case_sensitive_eq(query: char, candidate: char) -> bool {
    query == candidate
}
