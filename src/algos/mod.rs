#[cfg(any(feature = "fzf-v1", feature = "fzf-v2"))]
mod fzf;

#[cfg(feature = "tests")]
pub use fzf::bonus as fzf_bonus;
#[cfg(feature = "tests")]
pub use fzf::penalty as fzf_penalty;
#[cfg(feature = "fzf-v1")]
pub use fzf::FzfV1;
#[cfg(any(feature = "fzf-v1", feature = "fzf-v2"))]
pub use fzf::{FzfDistance, FzfQuery};
