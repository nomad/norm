use crate::utils::*;

/// TODO: docs
#[derive(Copy, Clone)]
pub(crate) enum Candidate<'a> {
    Ascii(&'a [u8]),
    Unicode(&'a [char]),
}

impl<'a> Candidate<'a> {
    /// TODO: docs
    #[inline(always)]
    pub fn char(&self, char_idx: usize) -> char {
        match self {
            Candidate::Ascii(candidate) => candidate[char_idx] as _,
            Candidate::Unicode(candidate) => candidate[char_idx],
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn char_len(&self) -> usize {
        match self {
            Candidate::Ascii(slice) => slice.len(),
            Candidate::Unicode(slice) => slice.len(),
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn find_first_from(
        &self,
        char_offset: usize,
        ch: char,
        is_case_sensitive: bool,
        char_eq: CharEq,
    ) -> Option<usize> {
        match self {
            Candidate::Ascii(slice) => {
                if !ch.is_ascii() {
                    return None;
                }

                let slice = &slice[char_offset..];

                find_first_ascii(ch as _, slice, is_case_sensitive)
                    .map(|offset| offset + char_offset)
            },

            Candidate::Unicode(slice) => {
                let slice = &slice[char_offset..];

                find_first_unicode(ch, slice, char_eq)
                    .map(|idx| idx + char_offset)
            },
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn find_last(
        &self,
        ch: char,
        is_case_sensitive: bool,
        char_eq: CharEq,
    ) -> Option<usize> {
        match self {
            Candidate::Ascii(slice) => {
                if ch.is_ascii() {
                    find_last_ascii(ch as _, slice, is_case_sensitive)
                } else {
                    None
                }
            },

            Candidate::Unicode(slice) => find_last_unicode(ch, slice, char_eq),
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn matches(
        &self,
        ch: char,
        is_case_sensitive: bool,
        char_eq: CharEq,
    ) -> CandidateMatches<'a> {
        match self {
            Candidate::Ascii(slice) => {
                CandidateMatches::from_ascii(ch, slice, is_case_sensitive, 0)
            },

            Candidate::Unicode(slice) => {
                CandidateMatches::from_unicode(ch, slice, char_eq, 0)
            },
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn matches_from(
        &self,
        char_offset: usize,
        ch: char,
        is_case_sensitive: bool,
        char_eq: CharEq,
    ) -> CandidateMatches<'a> {
        match self {
            Candidate::Ascii(slice) => {
                let slice = &slice[char_offset..];
                CandidateMatches::from_ascii(
                    ch,
                    slice,
                    is_case_sensitive,
                    char_offset,
                )
            },

            Candidate::Unicode(slice) => {
                let slice = &slice[char_offset..];
                CandidateMatches::from_unicode(ch, slice, char_eq, char_offset)
            },
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn slice(self, char_start: usize, char_end: usize) -> Self {
        match self {
            Candidate::Ascii(slice) => {
                Candidate::Ascii(&slice[char_start..char_end])
            },

            Candidate::Unicode(slice) => {
                Candidate::Unicode(&slice[char_start..char_end])
            },
        }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn to_byte_offset(self, char_offset: usize) -> usize {
        match self {
            Candidate::Ascii(_) => char_offset,
            Candidate::Unicode(slice) => {
                slice[..char_offset].iter().map(|&ch| ch.len_utf8()).sum()
            },
        }
    }
}

#[inline(always)]
fn find_first_ascii(
    needle: u8,
    haystack: &[u8],
    is_case_sensitive: bool,
) -> Option<usize> {
    if is_case_sensitive || !needle.is_ascii_alphabetic() {
        memchr::memchr(needle, haystack)
    } else {
        memchr::memchr2(needle, ascii_letter_flip_case(needle), haystack)
    }
}

#[inline(always)]
fn find_last_ascii(
    needle: u8,
    haystack: &[u8],
    is_case_sensitive: bool,
) -> Option<usize> {
    if is_case_sensitive || !needle.is_ascii_alphabetic() {
        memchr::memrchr(needle, haystack)
    } else {
        memchr::memrchr2(needle, ascii_letter_flip_case(needle), haystack)
    }
}

#[inline(always)]
fn find_first_unicode(
    needle: char,
    haystack: &[char],
    char_eq: CharEq,
) -> Option<usize> {
    haystack
        .iter()
        .enumerate()
        .find_map(|(idx, &ch)| char_eq(needle, ch).then_some(idx))
}

#[inline(always)]
fn find_last_unicode(
    needle: char,
    haystack: &[char],
    char_eq: CharEq,
) -> Option<usize> {
    haystack
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, &ch)| char_eq(needle, ch).then_some(idx))
}

struct UnicodeMatches<'a> {
    needle: char,
    haystack: &'a [char],
    char_eq: CharEq,
    offset: usize,
}

impl<'a> UnicodeMatches<'a> {
    fn new(ch: char, haystack: &'a [char], char_eq: CharEq) -> Self {
        Self { needle: ch, haystack, char_eq, offset: 0 }
    }
}

impl Iterator for UnicodeMatches<'_> {
    type Item = usize;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let idx =
            self.haystack.iter().enumerate().find_map(|(idx, &ch)| {
                (self.char_eq)(self.needle, ch).then_some(idx)
            })?;

        self.haystack = &self.haystack[idx + 1..];

        let offset = self.offset + idx;

        self.offset = offset + 1;

        Some(offset)
    }
}

/// TODO: docs
pub(crate) struct CandidateMatches<'a> {
    iter: CandidateMatchesIter<'a>,
    start_offset: usize,
}

impl<'a> CandidateMatches<'a> {
    #[inline(always)]
    fn from_ascii(
        needle: char,
        haystack: &'a [u8],
        is_case_sensitive: bool,
        start_offset: usize,
    ) -> Self {
        if !needle.is_ascii() {
            return Self::from_unicode(needle, &[], char_eq(false, false), 0);
        }

        let needle = needle as u8;

        let iter = if is_case_sensitive || !needle.is_ascii_alphabetic() {
            CandidateMatchesIter::Memchr(memchr::Memchr::new(needle, haystack))
        } else {
            CandidateMatchesIter::Memchr2(memchr::Memchr2::new(
                needle,
                ascii_letter_flip_case(needle),
                haystack,
            ))
        };

        Self { iter, start_offset }
    }

    #[inline(always)]
    fn from_unicode(
        needle: char,
        haystack: &'a [char],
        char_eq: CharEq,
        start_offset: usize,
    ) -> Self {
        let iter = UnicodeMatches::new(needle, haystack, char_eq);
        Self { iter: CandidateMatchesIter::Unicode(iter), start_offset }
    }
}

enum CandidateMatchesIter<'a> {
    Memchr(memchr::Memchr<'a>),
    Memchr2(memchr::Memchr2<'a>),
    Unicode(UnicodeMatches<'a>),
}

impl Iterator for CandidateMatches<'_> {
    type Item = usize;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            CandidateMatchesIter::Memchr(memchr) => memchr.next(),
            CandidateMatchesIter::Memchr2(memchr2) => memchr2.next(),
            CandidateMatchesIter::Unicode(unicode) => unicode.next(),
        }
        .map(|offset| self.start_offset + offset)
    }
}
