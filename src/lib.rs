//! TODO: docs

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

extern crate alloc;

mod algos;
mod case_sensitivity;
mod r#match;
mod matched_ranges;
mod metric;
mod normalize;
mod utils;

pub use algos::*;
pub use case_sensitivity::CaseSensitivity;
use matched_ranges::MatchedRanges;
pub use metric::Metric;
pub use r#match::Match;
use utils::CharEq;
