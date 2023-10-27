#![allow(clippy::single_range_in_vec_init)]

mod common;

use common::SortedRanges;
use norm::fzf::{bonus, penalty, FzfParser, FzfV1};
use norm::{CaseSensitivity, Metric};

#[test]
fn fzf_v1_empty_query() {
    let mut fzf = FzfV1::new();
    let mut parser = FzfParser::new();
    assert!(fzf.distance(parser.parse(""), "foo").is_none());
}

#[test]
fn fzf_v1_upstream_1() {
    let mut fzf = FzfV1::new()
        .with_case_sensitivity(CaseSensitivity::Insensitive)
        .with_matched_ranges(true);

    let mut parser = FzfParser::new();

    let mach = fzf.distance(parser.parse("oBZ"), "fooBarbaz").unwrap();

    assert_eq!(
        mach.distance().into_score(),
        bonus::MATCH * 3 + bonus::CAMEL_123
            - penalty::GAP_START
            - penalty::GAP_EXTENSION * 3
    );

    assert_eq!(mach.matched_ranges().sorted(), [2..4, 8..9]);
}
