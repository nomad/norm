#[cfg(feature = "fzf-v1")]
mod fzf;
#[cfg(feature = "fzf-v1")]
pub use fzf::FzfV1;
#[cfg(any(feature = "fzf-v1", feature = "fzf-v2"))]
pub use fzf::{FzfDistance, FzfQuery};
