use super::{bonus, CharClass, Score};

/// TODO: docs
#[derive(Debug, Default, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum FzfScheme {
    /// TODO: docs
    #[default]
    Default,

    /// TODO: docs
    Path,

    /// TODO: docs
    History,
}

impl FzfScheme {
    /// TODO: docs
    #[inline]
    pub(super) fn into_inner(self) -> Scheme {
        match self {
            Self::Default => DEFAULT,
            Self::Path => PATH,
            Self::History => HISTORY,
        }
    }
}

/// TODO: docs
#[derive(Clone)]
pub(super) struct Scheme {
    pub bonus_boundary_white: Score,
    pub bonus_boundary_delimiter: Score,
    pub initial_char_class: CharClass,
    pub is_delimiter: fn(char) -> bool,
}

impl Default for Scheme {
    #[inline]
    fn default() -> Self {
        DEFAULT
    }
}

/// TODO: docs
pub const DEFAULT: Scheme = Scheme {
    bonus_boundary_white: bonus::BOUNDARY + 2,
    bonus_boundary_delimiter: bonus::BOUNDARY + 1,
    initial_char_class: CharClass::WhiteSpace,
    is_delimiter: is_delimiter_default,
};

#[inline]
fn is_delimiter_default(ch: char) -> bool {
    matches!(ch, '/' | ',' | ':' | ';' | '|')
}

/// TODO: docs
pub const PATH: Scheme = Scheme {
    bonus_boundary_white: bonus::BOUNDARY,
    bonus_boundary_delimiter: bonus::BOUNDARY + 1,
    initial_char_class: CharClass::Delimiter,
    is_delimiter: is_delimiter_path,
};

#[inline]
fn is_delimiter_path(ch: char) -> bool {
    // Using `std::path::MAIN_SEPARATOR` would force us to depend on `std`
    // instead of `core + alloc`, so we use a custom implementation.
    #[cfg(windows)]
    let os_path_separator = '\\';
    #[cfg(not(windows))]
    let os_path_separator = '/';

    ch == '/' || ch == os_path_separator
}

/// TODO: docs
pub const HISTORY: Scheme = Scheme {
    bonus_boundary_white: bonus::BOUNDARY,
    bonus_boundary_delimiter: bonus::BOUNDARY,
    initial_char_class: DEFAULT.initial_char_class,
    is_delimiter: DEFAULT.is_delimiter,
};
