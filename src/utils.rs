use crate::normalize::{is_normalized, normalize};

/// TODO: docs
pub(crate) type CharEq = fn(char, char) -> bool;

/// TODO: docs
const ASCII_CASE_MASK: u8 = 0b0010_0000;

/// TODO: docs
#[inline(always)]
pub fn ascii_letter_flip_case(ascii_letter: u8) -> u8 {
    debug_assert!(ascii_letter.is_ascii_alphabetic());
    ascii_letter ^ ASCII_CASE_MASK
}

#[inline(always)]
pub fn case_insensitive_eq(lhs: char, rhs: char) -> bool {
    lhs.eq_ignore_ascii_case(&rhs)
}

#[inline(always)]
pub fn case_insensitive_normalized_eq(lhs: char, rhs: char) -> bool {
    lhs.eq_ignore_ascii_case(&normalize_candidate_char(lhs, rhs))
}

#[inline(always)]
pub fn case_sensitive_eq(lhs: char, rhs: char) -> bool {
    lhs == rhs
}

#[inline(always)]
pub fn case_sensitive_normalized_eq(lhs: char, rhs: char) -> bool {
    lhs == normalize_candidate_char(lhs, rhs)
}

#[inline(always)]
pub fn char_eq(is_case_sensitive: bool, normalize_candidate: bool) -> CharEq {
    match (is_case_sensitive, normalize_candidate) {
        (false, false) => case_insensitive_eq,
        (true, false) => case_sensitive_eq,
        (false, true) => case_insensitive_normalized_eq,
        (true, true) => case_sensitive_normalized_eq,
    }
}

/// TODO: docs
#[inline(always)]
fn leading_spaces(s: &str) -> usize {
    s.bytes().take_while(|&b| b == b' ').count()
}

/// TODO: docs
#[inline(always)]
fn normalize_candidate_char(query_char: char, candidate_char: char) -> char {
    if is_normalized(query_char) {
        normalize(candidate_char)
    } else {
        candidate_char
    }
}

/// TODO: docs
#[inline(always)]
pub fn strip_leading_spaces(s: &str) -> &str {
    &s[leading_spaces(s)..]
}
