#![allow(clippy::single_range_in_vec_init)]

mod fzf_common;

use common::SortedRanges;
use fzf_common as common;
use norm::fzf::{bonus, FzfParser, FzfV2};
use norm::{CaseSensitivity, Metric};

#[test]
fn fzf_v2_empty_query() {
    common::empty_query::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_exact_1() {
    common::upstream_exact_1::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_exact_2() {
    common::upstream_exact_2::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_exact_3() {
    common::upstream_exact_3::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_exact_4() {
    common::upstream_exact_4::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_exact_5() {
    common::upstream_exact_5::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_exact_6() {
    common::upstream_exact_6::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_exact_7() {
    common::upstream_exact_7::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_fuzzy_1() {
    common::upstream_fuzzy_1::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_2() {
    common::upstream_fuzzy_2::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_3() {
    common::upstream_fuzzy_3::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_4() {
    common::upstream_fuzzy_4::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_5() {
    common::upstream_fuzzy_5::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_6() {
    common::upstream_fuzzy_6::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_7() {
    common::upstream_fuzzy_7::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_8() {
    common::upstream_fuzzy_8::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_9() {
    common::upstream_fuzzy_9::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_10() {
    common::upstream_fuzzy_10::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_11() {
    common::upstream_fuzzy_11::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_12() {
    common::upstream_fuzzy_12::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_13() {
    common::upstream_fuzzy_13::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_14() {
    common::upstream_fuzzy_14::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_15() {
    common::upstream_fuzzy_15::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_16() {
    common::upstream_fuzzy_16::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_17() {
    common::upstream_fuzzy_17::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_18() {
    common::upstream_fuzzy_18::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_19() {
    common::upstream_fuzzy_19::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_fuzzy_20() {
    common::upstream_fuzzy_20::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_prefix_1() {
    common::upstream_prefix_1::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_prefix_2() {
    common::upstream_prefix_2::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_prefix_3() {
    common::upstream_prefix_3::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_prefix_4() {
    common::upstream_prefix_4::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_prefix_5() {
    common::upstream_prefix_5::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_prefix_6() {
    common::upstream_prefix_6::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_prefix_7() {
    common::upstream_prefix_7::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_prefix_8() {
    common::upstream_prefix_8::<FzfV2>()
}

#[test]
fn fzf_v2_score_1() {
    let mut fzf = FzfV2::new()
        .with_case_sensitivity(CaseSensitivity::Sensitive)
        .with_matched_ranges(true);

    let mut parser = FzfParser::new();

    let mach = fzf.distance(parser.parse("jelly"), "jellyfish").unwrap();

    assert_eq!(
        mach.distance().into_score(),
        bonus::MATCH * 5
            + fzf.scheme().bonus_boundary_white
                * bonus::FIRST_QUERY_CHAR_MULTIPLIER
            + fzf.scheme().bonus_boundary_white * 4
    );

    assert_eq!(mach.matched_ranges().sorted(), [0..5]);
}
