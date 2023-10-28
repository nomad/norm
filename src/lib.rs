//! TODO: docs

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

mod algos;
mod case_sensitivity;
mod r#match;
mod metric;

pub use algos::*;
use case_sensitivity::CaseMatcher;
pub use case_sensitivity::CaseSensitivity;
pub use metric::Metric;
pub use r#match::Match;
