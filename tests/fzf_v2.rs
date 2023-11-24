#![allow(clippy::single_range_in_vec_init)]

mod fzf_common;

use fzf_common as common;
use norm::fzf::{bonus, FzfParser, FzfV2};
use norm::{CaseSensitivity, Metric};

#[test]
fn fzf_v2_upstream_empty() {
    common::upstream_empty::<FzfV2>();
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
fn fzf_v2_upstream_suffix_1() {
    common::upstream_suffix_1::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_suffix_2() {
    common::upstream_suffix_2::<FzfV2>();
}

#[test]
fn fzf_v2_upstream_suffix_3() {
    common::upstream_suffix_3::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_suffix_4() {
    common::upstream_suffix_4::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_suffix_5() {
    common::upstream_suffix_5::<FzfV2>()
}

#[test]
fn fzf_v2_upstream_suffix_6() {
    common::upstream_suffix_6::<FzfV2>()
}

#[test]
fn fzf_v2_score_1() {
    let mut fzf = FzfV2::new();

    let mut parser = FzfParser::new();

    let mut ranges = norm::MatchedRanges::default();

    let distance = fzf
        .with_case_sensitivity(CaseSensitivity::Sensitive)
        .with_matched_ranges(true)
        .distance_and_ranges(parser.parse("jelly"), "jellyfish", &mut ranges)
        .unwrap();

    assert_eq!(
        distance.into_score(),
        bonus::MATCH * 5
            + fzf.scheme().bonus_boundary_white
                * bonus::FIRST_QUERY_CHAR_MULTIPLIER
            + fzf.scheme().bonus_boundary_white * 4
    );

    assert_eq!(ranges.as_slice(), [0..5]);
}

#[test]
fn fzf_v2_score_2() {
    let mut fzf = FzfV2::new();

    let mut parser = FzfParser::new();

    let distance = fzf
        .with_case_sensitivity(CaseSensitivity::Sensitive)
        .with_matched_ranges(true)
        .distance(parser.parse("!$"), "$$2");

    assert!(distance.is_none());
}

#[test]
fn fzf_v2_score_3() {
    let mut fzf = FzfV2::new();

    let mut parser = FzfParser::new();

    let mut ranges = norm::MatchedRanges::default();

    let _ = fzf
        .with_case_sensitivity(CaseSensitivity::Sensitive)
        .with_matched_ranges(true)
        .distance_and_ranges(
            parser.parse("\0\0"),
            "\0#B\0\u{364}\0\0",
            &mut ranges,
        )
        .unwrap();

    assert_eq!(ranges.as_slice(), [6..8]);
}

#[test]
fn fzf_v2_score_4() {
    let mut fzf = FzfV2::new();

    let mut parser = FzfParser::new();

    let mut ranges = norm::MatchedRanges::default();

    let _ = fzf
        .with_case_sensitivity(CaseSensitivity::Sensitive)
        .with_matched_ranges(true)
        .with_normalization(true)
        .distance_and_ranges(
            parser.parse("e !"),
            " !I\\hh+\u{364}",
            &mut ranges,
        )
        .unwrap();

    assert_eq!(ranges.as_slice(), [1..2, 7..9]);
}

#[test]
fn fzf_v2_score_5() {
    let mut fzf = FzfV2::new();

    let mut parser = FzfParser::new();

    let mut ranges = norm::MatchedRanges::default();

    let _ = fzf
        .with_case_sensitivity(CaseSensitivity::Insensitive)
        .with_matched_ranges(true)
        .with_normalization(true)
        .distance_and_ranges(parser.parse("E"), "\u{364}E", &mut ranges)
        .unwrap();

    assert_eq!(ranges.as_slice(), [0..2]);
}

#[test]
fn fzf_v2_score_6() {
    let mut fzf = FzfV2::new();

    let mut parser = FzfParser::new();

    let mut ranges = norm::MatchedRanges::default();

    let query = parser.parse("!2\t\0\0\0WWHHWHWWWWWWWZ !I");

    let distance = fzf
        .with_case_sensitivity(CaseSensitivity::Insensitive)
        .with_matched_ranges(true)
        .with_normalization(true)
        .distance_and_ranges(
            query,
            "\u{6}\0\0 N\u{364}\u{e}\u{365}+",
            &mut ranges,
        );

    assert!(distance.is_none());
}
