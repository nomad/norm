/// TODO: docs
const ASCII_CASE_MASK: u8 = 0b0010_0000;

/// TODO: docs
#[inline(always)]
fn ascii_letter_flip_case(ascii_letter: u8) -> u8 {
    debug_assert!(ascii_letter.is_ascii_alphabetic());
    ascii_letter ^ ASCII_CASE_MASK
}

#[inline(always)]
pub fn case_insensitive_eq(lhs: char, rhs: char) -> bool {
    lhs.eq_ignore_ascii_case(&rhs)
}

#[inline(always)]
pub fn case_sensitive_eq(lhs: char, rhs: char) -> bool {
    lhs == rhs
}

/// TODO: docs
#[inline(always)]
pub fn find_first(
    needle: char,
    haystack: &str,
    is_search_case_sensitive: bool,
) -> Option<usize> {
    if needle.is_ascii() {
        find_first_ascii(needle as u8, haystack, is_search_case_sensitive)
    } else {
        find_first_unicode(needle, haystack)
    }
}

/// TODO: docs
#[inline(always)]
fn find_first_ascii(
    needle: u8,
    haystack: &str,
    is_search_case_sensitive: bool,
) -> Option<usize> {
    debug_assert!(needle.is_ascii());

    // We can convert the haystack to a byte slice because all multibyte characters
    // start with a non-ASCII byte, so we don't have to worry about accidentally
    // returning a false positive.
    let haystack = haystack.as_bytes();

    if is_search_case_sensitive || !needle.is_ascii_alphabetic() {
        memchr::memchr(needle, haystack)
    } else {
        memchr::memchr2(needle, ascii_letter_flip_case(needle), haystack)
    }
}

/// TODO: docs
#[inline(always)]
fn find_first_unicode(needle: char, haystack: &str) -> Option<usize> {
    debug_assert!(!needle.is_ascii());

    haystack
        .char_indices()
        .find_map(|(offset, ch)| (needle == ch).then_some(offset))
}

/// TODO: docs
#[inline(always)]
pub fn find_last(
    needle: char,
    haystack: &str,
    is_search_case_sensitive: bool,
) -> Option<usize> {
    if needle.is_ascii() {
        find_last_ascii(needle as u8, haystack, is_search_case_sensitive)
    } else {
        find_last_unicode(needle, haystack)
    }
}

/// TODO: docs
#[inline(always)]
fn find_last_ascii(
    needle: u8,
    haystack: &str,
    is_search_case_sensitive: bool,
) -> Option<usize> {
    debug_assert!(needle.is_ascii());

    // We can convert the haystack to a byte slice because all multibyte characters
    // start with a non-ASCII byte, so we don't have to worry about accidentally
    // returning a false positive.
    let haystack = haystack.as_bytes();

    if is_search_case_sensitive || !needle.is_ascii_alphabetic() {
        memchr::memchr_iter(needle, haystack).next_back()
    } else {
        memchr::memchr2_iter(needle, ascii_letter_flip_case(needle), haystack)
            .next_back()
    }
}

/// TODO: docs
#[inline(always)]
fn find_last_unicode(needle: char, haystack: &str) -> Option<usize> {
    debug_assert!(!needle.is_ascii());

    haystack
        .char_indices()
        .find_map(|(offset, ch)| (needle == ch).then_some(offset))
}
