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
fn fzf_v2_upstream_1() {
    common::upstream_1::<FzfV2>();
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
