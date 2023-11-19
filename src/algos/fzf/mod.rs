//! Metrics implementing [fzf]'s fuzzy search algorithm.
//!
//! This module contains two metrics -- [`FzfV1`] and [`FzfV2`] -- both of
//! which were ported directly from the original fzf project.
//!
//! ## `FzfV1` and `FzfV2`
//!
//! `FzfV1` is a simple metric that looks for the first "fuzzy" match of a
//! query within a candidate. Its time complexity is `O(len(candidate))` for
//! both matches and non-matches.
//!
//! `FzfV2` can often provide more accurate matches by finding the best
//! possible position of a query within a candidate. This comes at the cost of
//! a `O(len(query) * len(candidate))` time complexity for the distance
//! calculation in the case of a match. Non-matches are still filtered out in
//! `O(len(candidate))` time.
//!
//! For more information on the individual algorithms, refer to the
//! documentation of each metric.
//!
//! ## Extended-search mode
//!
//! Both metrics fully support fzf's [extended-search mode][extended-search] by
//! parsing the query with [`FzfParser::parse`].
//!
//! In extended-search mode, spaces in the query are treated as logical AND
//! operators, while the pipe character `|` is treated as a logical OR. For
//! example, the query `"foo bar | baz"` would only match candidates that
//! contain `"foo"` and either `"bar"` or `"baz"`. It's also possible to query
//! for candidates that either begin or end with a certain string, to negate a
//! query, and more.
//!
//! To know more about extended-search mode's syntax you can
//! look directly at [fzf's docs][extended-search] on it, or at the
//! documentation of the [`FzfParser`].
//!
//! ## Conformance to fzf
//!
//! The behavior of both [`FzfV1`] and [`FzfV2`] is intended to always match
//! that of the latest release of fzf. Any discrepancy between our
//! implementation and fzf's should be considered a bug.
//!
//! [fzf]: https://github.com/junegunn/fzf
//! [extended-search]: https://github.com/junegunn/fzf#search-syntax

mod common;
mod distance;
mod parser;
mod query;
mod scheme;
mod scoring;
mod slab;
#[cfg(feature = "fzf-v1")]
mod v1;
#[cfg(feature = "fzf-v1")]
mod v2;

use common::*;
pub use distance::FzfDistance;
use distance::*;
pub use parser::*;
pub use query::FzfQuery;
pub use scheme::FzfScheme;
#[doc(hidden)]
pub use scheme::Scheme;
use scoring::*;
#[cfg(feature = "fzf-v1")]
pub use v1::FzfV1;
#[cfg(feature = "fzf-v1")]
pub use v2::FzfV2;

#[doc(hidden)]
pub mod bonus {
    //! TODO: docs

    use super::*;

    /// TODO: docs
    pub const MATCH: Score = 16;

    /// TODO: docs
    pub const BOUNDARY: Score = MATCH / 2;

    /// TODO: docs
    pub const NON_WORD: Score = MATCH / 2;

    /// TODO: docs
    pub const CAMEL_123: Score = BOUNDARY - penalty::GAP_EXTENSION;

    /// TODO: docs
    pub const CONSECUTIVE: Score = penalty::GAP_START + penalty::GAP_EXTENSION;

    /// TODO: docs
    pub const FIRST_QUERY_CHAR_MULTIPLIER: Score = 2;
}

#[doc(hidden)]
pub mod penalty {
    //! TODO: docs

    use super::*;

    /// TODO: docs
    pub const GAP_START: Score = 3;

    /// TODO: docs
    pub const GAP_EXTENSION: Score = 1;
}
