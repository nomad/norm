use core::ops::Range;

use crate::{Match, Metric};

/// TODO: docs.
#[derive(Debug)]
pub struct FzfQuery<'a> {
    /// TODO: docs.
    raw: &'a str,
}

impl<'a> FzfQuery<'a> {
    /// TODO: docs
    #[inline]
    pub fn from_str(s: &'a str) -> Self {
        Self { raw: s }
    }

    /// TODO: docs
    #[inline]
    fn is_empty(&self) -> bool {
        self.raw().is_empty()
    }

    /// TODO: docs
    #[inline]
    fn raw(&self) -> &'a str {
        self.raw
    }
}

#[cfg(feature = "fzf-v1")]
pub use v1::FzfV1;

#[cfg(feature = "fzf-v1")]
mod v1 {
    use super::*;

    /// TODO: docs
    #[derive(Default)]
    pub struct FzfV1 {
        /// TODO: docs
        is_case_sensitive: bool,
    }

    impl FzfV1 {
        /// TODO: docs
        #[inline]
        pub fn new() -> Self {
            Self::default()
        }

        /// TODO: docs
        #[inline]
        pub fn fuzzy_match(
            &self,
            query: &str,
            candidate: &str,
        ) -> Option<Range<usize>> {
            debug_assert!(!query.is_empty());

            let range_forward =
                forward_pass(query, candidate, self.is_case_sensitive)?;

            let candidate = &candidate[range_forward.clone()];

            let start_backward =
                backward_pass(query, candidate, self.is_case_sensitive);

            Some(range_forward.start + start_backward..range_forward.end)
        }
    }

    impl Metric for FzfV1 {
        type Query<'a> = FzfQuery<'a>;

        type Distance = u64;

        #[inline]
        fn distance(
            &self,
            query: FzfQuery<'_>, // helwo
            candidate: &str,     // Hello World!
        ) -> Option<Match<Self::Distance>> {
            if query.is_empty() {
                return None;
            }

            let range = self.fuzzy_match(query.raw(), candidate)?;

            todo!();
        }
    }

    /// TODO: docs
    #[inline]
    fn forward_pass(
        query: &str,
        candidate: &str,
        is_case_sensitive: bool,
    ) -> Option<Range<usize>> {
        let mut start_offset = None;

        let mut end_offset = None;

        let mut query_chars = query.chars();

        let mut query_char = query_chars.next().expect("query is not empty");

        for (offset, mut candidate_char) in candidate.char_indices() {
            if !is_case_sensitive {
                candidate_char.make_ascii_lowercase();
            }

            if query_char != candidate_char {
                continue;
            }

            if start_offset.is_none() {
                start_offset = Some(offset);
            }

            let Some(next_target_char) = query_chars.next() else {
                end_offset = Some(offset + candidate_char.len_utf8());
                break;
            };

            query_char = next_target_char;
        }

        let (Some(start), Some(end)) = (start_offset, end_offset) else {
            return None;
        };

        Some(start..end)
    }

    /// TODO: docs
    #[inline]
    fn backward_pass(
        query: &str,
        candidate: &str,
        is_case_sensitive: bool,
    ) -> usize {
        // The candidate must start with the first character of the query.
        debug_assert!(candidate.starts_with(query.chars().next().unwrap()));

        // The candidate must end with the last character of the query.
        debug_assert!(candidate.ends_with(query.chars().next_back().unwrap()));

        let mut start_offset = 0;

        let mut query_chars = query.chars().rev();

        let mut query_char = query_chars.next().expect("query is not empty");

        for (offset, mut candidate_char) in candidate.char_indices().rev() {
            if !is_case_sensitive {
                candidate_char.make_ascii_lowercase();
            }

            if query_char != candidate_char {
                continue;
            }

            let Some(next_query_char) = query_chars.next() else {
                start_offset = offset;
                break;
            };

            query_char = next_query_char;
        }

        start_offset
    }
}
