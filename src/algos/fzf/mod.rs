//! TODO: docs

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
