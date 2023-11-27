#![allow(clippy::single_range_in_vec_init)]

mod fzf_common;

use fzf_common as common;
use norm::fzf::{FzfParser, FzfV1};
use norm::{CaseSensitivity, Metric};

#[test]
fn fzf_v1_upstream_empty() {
    common::upstream_empty::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_exact_1() {
    common::upstream_exact_1::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_exact_2() {
    common::upstream_exact_2::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_exact_3() {
    common::upstream_exact_3::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_exact_4() {
    common::upstream_exact_4::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_exact_5() {
    common::upstream_exact_5::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_exact_6() {
    common::upstream_exact_6::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_exact_7() {
    common::upstream_exact_7::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_fuzzy_1() {
    common::upstream_fuzzy_1::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_2() {
    common::upstream_fuzzy_2::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_3() {
    common::upstream_fuzzy_3::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_fuzzy_4() {
    common::upstream_fuzzy_4::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_fuzzy_5() {
    common::upstream_fuzzy_5::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_fuzzy_6() {
    common::upstream_fuzzy_6::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_fuzzy_7() {
    common::upstream_fuzzy_7::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_fuzzy_8() {
    common::upstream_fuzzy_8::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_9() {
    common::upstream_fuzzy_9::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_10() {
    common::upstream_fuzzy_10::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_11() {
    common::upstream_fuzzy_11::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_12() {
    common::upstream_fuzzy_12::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_13() {
    common::upstream_fuzzy_13::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_14() {
    common::upstream_fuzzy_14::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_15() {
    common::upstream_fuzzy_15::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_16() {
    common::upstream_fuzzy_16::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_17() {
    common::upstream_fuzzy_17::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_18() {
    common::upstream_fuzzy_18::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_19() {
    common::upstream_fuzzy_19::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_fuzzy_20() {
    common::upstream_fuzzy_20::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_prefix_1() {
    common::upstream_prefix_1::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_prefix_2() {
    common::upstream_prefix_2::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_prefix_3() {
    common::upstream_prefix_3::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_prefix_4() {
    common::upstream_prefix_4::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_prefix_5() {
    common::upstream_prefix_5::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_prefix_6() {
    common::upstream_prefix_6::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_prefix_7() {
    common::upstream_prefix_7::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_prefix_8() {
    common::upstream_prefix_8::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_suffix_1() {
    common::upstream_suffix_1::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_suffix_2() {
    common::upstream_suffix_2::<FzfV1>();
}

#[test]
fn fzf_v1_upstream_suffix_3() {
    common::upstream_suffix_3::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_suffix_4() {
    common::upstream_suffix_4::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_suffix_5() {
    common::upstream_suffix_5::<FzfV1>()
}

#[test]
fn fzf_v1_upstream_suffix_6() {
    common::upstream_suffix_6::<FzfV1>()
}

#[test]
fn fzf_v1_score_1() {
    let mut fzf = FzfV1::new();

    let mut parser = FzfParser::new();

    let mut ranges = Vec::new();

    let _ = fzf
        .set_case_sensitivity(CaseSensitivity::Sensitive)
        .distance_and_ranges(parser.parse("ZZ"), "ӥZZZ", &mut ranges)
        .unwrap();

    assert_eq!(ranges, [2..4]);
}

#[test]
fn fzf_v1_score_2() {
    let mut fzf = FzfV1::new();

    let mut parser = FzfParser::new();

    let query = parser.parse("^\\$ ]]%]]'\0\0\0\0\0\0");

    let mach = fzf
        .set_case_sensitivity(CaseSensitivity::Sensitive)
        .distance(query, "\0");

    assert!(mach.is_none());
}

#[test]
fn fzf_v1_score_3() {
    let mut fzf = FzfV1::new();

    let mut parser = FzfParser::new();

    let query = parser.parse("^\\$");

    let mach = fzf
        .set_case_sensitivity(CaseSensitivity::Sensitive)
        .distance(query, " ");

    assert!(mach.is_none());
}

#[test]
fn fzf_v1_score_4() {
    let mut fzf = FzfV1::new();

    let mut parser = FzfParser::new();

    let mut ranges = Vec::new();

    let query = parser.parse("z\n");

    let candidate = "ZZ\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\u{65e}\nZ\u{65e}";

    let _ = fzf
        .set_case_sensitivity(CaseSensitivity::Insensitive)
        .distance_and_ranges(query, candidate, &mut ranges)
        .unwrap();

    assert_eq!(ranges, [1..2, 21..22]);
}

#[test]
fn fzf_v1_score_5() {
    let mut fzf = FzfV1::new();

    let mut parser = FzfParser::new();

    let mut ranges = Vec::new();

    let _ = fzf
        .set_case_sensitivity(CaseSensitivity::Sensitive)
        .set_normalization(true)
        .distance_and_ranges(
            parser.parse("e !"),
            " !I\\hh+\u{364}",
            &mut ranges,
        )
        .unwrap();

    assert_eq!(ranges, [1..2, 7..9]);
}

#[test]
fn fzf_v1_score_6() {
    let mut fzf = FzfV1::new();

    let mut parser = FzfParser::new();

    let mut ranges = Vec::new();

    let query = parser.parse("^e");

    let _ = fzf
        .set_case_sensitivity(CaseSensitivity::Insensitive)
        .set_normalization(true)
        .distance_and_ranges(query, "\u{364}", &mut ranges);

    assert_eq!(ranges, [0..2]);
}
