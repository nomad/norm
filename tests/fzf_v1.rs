use norm::fzf::{bonus, penalty, FzfQuery, FzfV1};
use norm::{CaseSensitivity, Metric};

#[test]
fn fzf_v1_empty_query() {
    let fzf = FzfV1::new();
    let empty = FzfQuery::new("");
    assert!(fzf.distance(empty, "foo").is_none());
}

#[test]
fn fzf_v1_score_1() {
    let fzf = FzfV1::new()
        .with_case_sensitivity(CaseSensitivity::Insensitive)
        .with_matched_ranges(true);

    let query = FzfQuery::new("oBZ");

    let mach = fzf.distance(query, "fooBarbaz").unwrap();

    assert_eq!(
        mach.distance().into_score(),
        bonus::MATCH * 3 + bonus::CAMEL_123
            - penalty::GAP_START
            - penalty::GAP_EXTENSION * 3
    );

    assert_eq!(mach.matched_ranges(), [2..4, 8..9]);
}
