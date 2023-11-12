use super::*;

/// TODO: docs
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CharClass {
    /// TODO: docs
    WhiteSpace,

    /// TODO: docs
    NonWord,

    /// TODO: docs
    Delimiter,

    /// TODO: docs
    Lower,

    /// TODO: docs
    Upper,

    /// TODO: docs
    Letter,

    /// TODO: docs
    Number,
}

/// TODO: docs
#[inline]
pub(super) fn char_class(ch: char, scheme: &Scheme) -> CharClass {
    if ch.is_ascii() {
        ascii_char_class(ch, scheme)
    } else {
        non_ascii_char_class(ch, scheme)
    }
}

/// TODO: docs
#[inline]
fn ascii_char_class(ch: char, scheme: &Scheme) -> CharClass {
    if ch.is_ascii_lowercase() {
        CharClass::Lower
    } else if ch.is_ascii_uppercase() {
        CharClass::Upper
    } else if ch.is_ascii_digit() {
        CharClass::Number
    } else if ch.is_ascii_whitespace() {
        CharClass::WhiteSpace
    } else if (scheme.is_delimiter)(ch) {
        CharClass::Delimiter
    } else {
        CharClass::NonWord
    }
}

/// TODO: docs
#[inline]
fn non_ascii_char_class(ch: char, scheme: &Scheme) -> CharClass {
    if ch.is_lowercase() {
        CharClass::Lower
    } else if ch.is_uppercase() {
        CharClass::Upper
    } else if ch.is_numeric() {
        CharClass::Number
    } else if ch.is_alphabetic() {
        CharClass::Letter
    } else if ch.is_whitespace() {
        CharClass::WhiteSpace
    } else if (scheme.is_delimiter)(ch) {
        CharClass::Delimiter
    } else {
        CharClass::NonWord
    }
}

/// TODO: docs
#[inline]
pub(super) fn bonus(
    prev_class: CharClass,
    next_class: CharClass,
    scheme: &Scheme,
) -> Score {
    use CharClass::*;

    match next_class {
        NonWord => bonus::NON_WORD,

        WhiteSpace => scheme.bonus_boundary_white,

        Upper if prev_class == Lower => bonus::CAMEL_123,

        Number if prev_class != Number => bonus::CAMEL_123,

        _ => {
            if prev_class == WhiteSpace {
                scheme.bonus_boundary_white
            } else if prev_class == Delimiter {
                scheme.bonus_boundary_delimiter
            } else if prev_class == NonWord {
                bonus::BOUNDARY
            } else {
                0
            }
        },
    }
}
