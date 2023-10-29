use crate::fzf::FzfQuery;
use crate::*;

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
            Self::Sensitive => utils::case_sensitive_eq,

            Self::Insensitive => utils::case_insensitive_eq,

            Self::Smart => {
                if query.has_uppercase() {
                    utils::case_sensitive_eq
                } else {
                    utils::case_insensitive_eq
                }
            },
        }
    }
}

/// TODO: docs
pub(crate) type CaseMatcher = fn(char, char) -> bool;
