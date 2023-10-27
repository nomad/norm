#![allow(clippy::single_range_in_vec_init)]

mod fzf_common;

use fzf_common as common;
use norm::fzf::FzfV1;

#[test]
fn fzf_v1_empty_query() {
    common::empty_query::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_1() {
    common::upstream_1::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_2() {
    common::upstream_2::<FzfV1>();
}
