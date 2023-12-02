//! Metrics implementing fzf's fuzzy search algorithm.
//!
//! This module contains two metrics -- [`FzfV1`] and [`FzfV2`] -- which were
//! ported from [fzf], a popular command-line fuzzy-finder.
//! The behavior of both metrics is intended to always match that of the latest
//! release of fzf. Any discrepancy between our implementation and fzf's should
//! be considered a bug.
//!
//! ## Extended-search mode
//!
//! fzf's [extended-search mode][esm] is fully-supported by parsing the query
//! with [`FzfParser::parse`].
//!
//! In extended-search mode, spaces in the query are treated as logical AND
//! operators, while the pipe character `|` is treated as a logical OR. For
//! example, the query `"foo bar | baz"` would only match candidates that
//! contain `"foo"` and either `"bar"` or `"baz"`. It's also possible to query
//! for candidates that either begin or end with a certain string, to negate a
//! query, and more.
//!
//! To know more about extended-search mode's syntax you can look directly at
//! [fzf's docs][esm] on it, or at the documentation of the [`FzfParser`].
//!
//! [fzf]: https://github.com/junegunn/fzf
//! [esm]: https://github.com/junegunn/fzf#search-syntax

mod candidate;
mod distance;
mod fzf;
#[cfg(feature = "fzf-v1")]
mod fzf_v1;
#[cfg(feature = "fzf-v2")]
mod fzf_v2;
mod parser;
mod query;
mod scheme;
mod scoring;
mod slab;

use candidate::*;
pub use distance::FzfDistance;
use distance::*;
use fzf::*;
#[cfg(feature = "fzf-v1")]
pub use fzf_v1::FzfV1;
#[cfg(feature = "fzf-v2")]
pub use fzf_v2::FzfV2;
pub use parser::*;
pub use query::FzfQuery;
pub use scheme::FzfScheme;
#[doc(hidden)]
pub use scheme::Scheme;
use scoring::*;
use slab::*;

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
