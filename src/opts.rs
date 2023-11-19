use crate::utils::*;

/// TODO: docs
pub(crate) trait Opts: Copy {
    /// TODO: docs
    fn char_eq(&self, query_ch: char, candidate_ch: char) -> bool;

    /// TODO: docs
    fn find_first(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)>;

    /// TODO: docs
    fn find_last(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)>;

    /// TODO: docs
    fn to_char_offset(&self, candidate: &str, byte_offset: usize) -> usize;
}

#[derive(Clone, Copy)]
pub(crate) struct AsciiCandidateOpts {
    is_case_sensitive: bool,
}

impl AsciiCandidateOpts {
    #[inline(always)]
    pub fn new(is_case_sensitive: bool) -> Self {
        Self { is_case_sensitive }
    }
}

impl Opts for AsciiCandidateOpts {
    #[inline(always)]
    fn char_eq(&self, query_ch: char, candidate_ch: char) -> bool {
        if self.is_case_sensitive {
            query_ch == candidate_ch
        } else {
            query_ch.eq_ignore_ascii_case(&candidate_ch)
        }
    }

    #[inline(always)]
    fn to_char_offset(&self, _: &str, byte_offset: usize) -> usize {
        byte_offset
    }

    #[inline(always)]
    fn find_first(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)> {
        if !query_ch.is_ascii() {
            return None;
        };

        let query_byte = query_ch as u8;

        let offset =
            if self.is_case_sensitive || !query_byte.is_ascii_alphabetic() {
                memchr::memchr(query_byte, candidate.as_bytes())
            } else {
                memchr::memchr2(
                    query_byte,
                    ascii_letter_flip_case(query_byte),
                    candidate.as_bytes(),
                )
            }?;

        Some((offset, 1))
    }

    #[inline(always)]
    fn find_last(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)> {
        if !query_ch.is_ascii() {
            return None;
        };

        let query_byte = query_ch as u8;

        let offset = if self.is_case_sensitive
            || !query_byte.is_ascii_alphabetic()
        {
            memchr::memchr_iter(query_byte, candidate.as_bytes()).next_back()
        } else {
            memchr::memchr2_iter(
                query_byte,
                ascii_letter_flip_case(query_byte),
                candidate.as_bytes(),
            )
            .next_back()
        }?;

        Some((offset, 1))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UnicodeCandidateOpts(CharEq);

impl UnicodeCandidateOpts {
    #[inline(always)]
    pub fn new(is_case_sensitive: bool, normalize_candidate: bool) -> Self {
        let fun = match (is_case_sensitive, normalize_candidate) {
            (false, false) => case_insensitive_eq,
            (true, false) => case_sensitive_eq,
            (false, true) => case_insensitive_normalized_eq,
            (true, true) => case_sensitive_normalized_eq,
        };

        Self(fun)
    }
}

impl Opts for UnicodeCandidateOpts {
    #[inline(always)]
    fn char_eq(&self, query_ch: char, candidate_ch: char) -> bool {
        self.0(query_ch, candidate_ch)
    }

    #[inline(always)]
    fn to_char_offset(&self, candidate: &str, byte_offset: usize) -> usize {
        char_len(&candidate[..byte_offset])
    }

    #[inline(always)]
    fn find_first(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)> {
        candidate.char_indices().find_map(|(offset, candidate_ch)| {
            self.char_eq(query_ch, candidate_ch)
                .then_some((offset, candidate_ch.len_utf8()))
        })
    }

    #[inline(always)]
    fn find_last(
        &self,
        query_ch: char,
        candidate: &str,
    ) -> Option<(usize, usize)> {
        candidate.char_indices().rev().find_map(|(offset, candidate_ch)| {
            self.char_eq(query_ch, candidate_ch)
                .then_some((offset, candidate_ch.len_utf8()))
        })
    }
}
