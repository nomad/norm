/// TODO: docs
pub(crate) type CharEq = fn(char, char) -> bool;

/// TODO: docs
const ASCII_CASE_MASK: u8 = 0b0010_0000;

/// TODO: docs
#[inline(always)]
fn ascii_letter_flip_case(ascii_letter: u8) -> u8 {
    debug_assert!(ascii_letter.is_ascii_alphabetic());
    ascii_letter ^ ASCII_CASE_MASK
}

#[inline(always)]
fn case_insensitive_eq(lhs: char, rhs: char) -> bool {
    lhs.eq_ignore_ascii_case(&rhs)
}

#[inline(always)]
fn case_insensitive_normalized_eq(lhs: char, rhs: char) -> bool {
    lhs.eq_ignore_ascii_case(&normalize_candidate_char(lhs, rhs))
}

#[inline(always)]
fn case_sensitive_eq(lhs: char, rhs: char) -> bool {
    lhs == rhs
}

#[inline(always)]
fn case_sensitive_normalized_eq(lhs: char, rhs: char) -> bool {
    lhs == normalize_candidate_char(lhs, rhs)
}

/// TODO: docs
#[inline(always)]
pub fn char_eq(
    is_case_sensitive: bool,
    normalize_candidate: bool,
) -> fn(char, char) -> bool {
    match (is_case_sensitive, normalize_candidate) {
        (false, false) => case_insensitive_eq,
        (true, false) => case_sensitive_eq,
        (false, true) => case_insensitive_normalized_eq,
        (true, true) => case_sensitive_normalized_eq,
    }
}

/// TODO: docs
#[inline(always)]
pub fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// TODO: docs
#[inline(always)]
pub fn find_first(
    needle: char,
    haystack: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    char_eq: CharEq,
) -> Option<(usize, char)> {
    if is_candidate_ascii {
        if needle.is_ascii() {
            find_first_ascii(needle as u8, haystack, is_case_sensitive)
        } else {
            None
        }
    } else {
        find_first_unicode(needle, haystack, char_eq)
    }
}

/// TODO: docs
#[inline(always)]
fn find_first_ascii(
    needle: u8,
    haystack: &str,
    is_case_sensitive: bool,
) -> Option<(usize, char)> {
    debug_assert!(needle.is_ascii());
    debug_assert!(haystack.is_ascii());

    let haystack = haystack.as_bytes();

    let idx = if is_case_sensitive || !needle.is_ascii_alphabetic() {
        memchr::memchr(needle, haystack)
    } else {
        memchr::memchr2(needle, ascii_letter_flip_case(needle), haystack)
    }?;

    Some((idx, haystack[idx] as char))
}

/// TODO: docs
#[inline(always)]
fn find_first_unicode(
    needle: char,
    haystack: &str,
    char_eq: CharEq,
) -> Option<(usize, char)> {
    haystack
        .char_indices()
        .find_map(|(offset, ch)| char_eq(needle, ch).then_some((offset, ch)))
}

/// TODO: docs
#[inline(always)]
pub fn find_last(
    needle: char,
    haystack: &str,
    is_candidate_ascii: bool,
    is_case_sensitive: bool,
    char_eq: CharEq,
) -> Option<(usize, char)> {
    if is_candidate_ascii {
        if needle.is_ascii() {
            find_last_ascii(needle as u8, haystack, is_case_sensitive)
        } else {
            None
        }
    } else {
        find_last_unicode(needle, haystack, char_eq)
    }
}

/// TODO: docs
#[inline(always)]
fn find_last_ascii(
    needle: u8,
    haystack: &str,
    is_case_sensitive: bool,
) -> Option<(usize, char)> {
    debug_assert!(needle.is_ascii());
    debug_assert!(haystack.is_ascii());

    let haystack = haystack.as_bytes();

    let idx = if is_case_sensitive || !needle.is_ascii_alphabetic() {
        memchr::memchr_iter(needle, haystack).next_back()
    } else {
        memchr::memchr2_iter(needle, ascii_letter_flip_case(needle), haystack)
            .next_back()
    }?;

    Some((idx, haystack[idx] as char))
}

/// TODO: docs
#[inline(always)]
fn find_last_unicode(
    needle: char,
    haystack: &str,
    char_eq: CharEq,
) -> Option<(usize, char)> {
    haystack
        .char_indices()
        .find_map(|(offset, ch)| char_eq(needle, ch).then_some((offset, ch)))
}

/// TODO: docs
#[inline(always)]
pub fn leading_spaces(s: &str) -> usize {
    s.bytes().take_while(|&b| b == b' ').count()
}

/// TODO: docs
#[inline(always)]
fn normalize_candidate_char(query_char: char, candidate_char: char) -> char {
    candidate_char
}

/// TODO: docs
#[inline(always)]
pub fn strip_leading_spaces(s: &str) -> &str {
    &s[leading_spaces(s)..]
}

/// TODO: docs
#[inline(always)]
pub fn trailing_spaces(s: &str) -> usize {
    s.bytes().rev().take_while(|&b| b == b' ').count()
}
