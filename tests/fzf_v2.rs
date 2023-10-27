use norm::fzf::{bonus, penalty, FzfParser, FzfV2};
use norm::{CaseSensitivity, Metric};

#[test]
fn fzf_v2_empty_query() {
    let mut fzf = FzfV2::new();
    let mut parser = FzfParser::new();
    assert!(fzf.distance(parser.parse(""), "foo").is_none());
}

#[test]
fn fzf_v2_score_1() {
    let mut fzf = FzfV2::new()
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

    // assert_eq!(mach.matched_ranges(), [2..4, 8..9]);
}
