use super::{bonus, CharClass, Score};

/// A distance scheme to tweak the distance algorithm.
///
/// This struct can be passed to both [`FzfV1`](super::FzfV1) and
/// [`FzfV2`](super::FzfV2) to tweak the distance algorithm based on the type
/// of candidates being searched.
#[derive(Debug, Default, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum FzfScheme {
    /// A generic distance scheme that works well for any type of input.
    #[default]
    Default,

    /// A distance scheme tailored for searching file paths. It assigns
    /// additional bonus points to the character immediately following a path
    /// separator (i.e. `/` on Unix-like systems and `\` on Windows).
    Path,

    /// A distance scheme tailored for searching shell command history which
    /// doesn't assign any additional bonus points.
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

    /// TODO: docs
    #[inline]
    pub(super) fn from_inner(scheme: &Scheme) -> Option<Self> {
        if scheme.bonus_boundary_white == DEFAULT.bonus_boundary_white {
            Some(Self::Default)
        } else if scheme.bonus_boundary_white == PATH.bonus_boundary_white {
            if scheme.initial_char_class == CharClass::Delimiter {
                Some(Self::Path)
            } else {
                Some(Self::History)
            }
        } else {
            None
        }
    }
}

/// TODO: docs
#[doc(hidden)]
#[derive(Clone)]
pub struct Scheme {
    pub bonus_boundary_white: Score,
    pub bonus_boundary_delimiter: Score,
    pub(super) initial_char_class: CharClass,
    pub(super) is_delimiter: fn(char) -> bool,
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
