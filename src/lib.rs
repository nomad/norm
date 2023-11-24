//! This crate provides a collection of different distance metrics on strings.
//!
//! This problem is sometimes referred to as "string similarity search", or
//! more colloquially "fuzzy matching". Given a query string and a number of
//! possible candidate strings, the goal is to:
//!
//! a) filter out the candidates that are too dissimilar from the query;
//!
//! b) rank the remaining candidates by their similarity to the query.
//!
//! Here both of these tasks are accomplished by implementing the [`Metric`]
//! trait. This trait is at the basis of norm's design, and it is implemented
//! by all of our metrics. Reading its documentation is a good place to start.
//!
//! # Performance
//!
//! Performance is a top priority for this crate. Our goal is to have the
//! fastest implementation of every metric algorithm we provide, across all
//! languages. [Here][bench] you can find a number of benchmarks comparing
//! norm's metrics to each other, as well as to other popular libraries.
//!
//! # Examples
//!
//! ```rust
//! use norm::fzf::{FzfParser, FzfV2};
//! use norm::Metric;
//!
//! let mut fzf = FzfV2::new();
//!
//! let mut parser = FzfParser::new();
//!
//! let query = parser.parse("aa");
//!
//! let cities = ["Geneva", "Ulaanbaatar", "New York City", "Adelaide"];
//!
//! let mut results = cities
//!     .iter()
//!     .copied()
//!     .filter_map(|city| fzf.distance(query, city).map(|dist| (city, dist)))
//!     .collect::<Vec<_>>();
//!
//! results.sort_by_key(|(_city, dist)| *dist);
//!
//! assert_eq!(results.len(), 2);
//! assert_eq!(results[0].0, "Adelaide");
//! assert_eq!(results[1].0, "Ulaanbaatar");
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

extern crate alloc;

mod algos;
mod candidate;
mod case_sensitivity;
mod matched_ranges;
mod metric;
mod normalize;
mod tiny_vec;
mod utils;

pub use algos::*;
use candidate::{Candidate, CandidateMatches};
pub use case_sensitivity::CaseSensitivity;
pub use matched_ranges::MatchedRanges;
pub use metric::Metric;
