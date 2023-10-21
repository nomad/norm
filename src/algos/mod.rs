#[cfg(feature = "fzf-v1")]
mod fzf_v1;

#[cfg(feature = "fzf-v2")]
mod fzf_v2;

#[cfg(any(feature = "fzf-v1", feature = "fzf-v2"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "fzf-v1", feature = "fzf-v2"))))]
pub mod fzf {
    //! TODO: docs

    #[cfg(feature = "fzf-v1")]
    pub use super::fzf_v1::*;
    #[cfg(feature = "fzf-v2")]
    pub use super::fzf_v2::*;
}
