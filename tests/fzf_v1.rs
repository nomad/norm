use norm::fzf::{bonus, penalty, FzfQuery, FzfV1};
use norm::{CaseSensitivity, Metric};

#[test]
fn fzf_v1_empty_query() {
    let fzf = FzfV1::new();
    let empty = FzfQuery::from_str("");
    assert!(fzf.distance(empty, "foo").is_none());
}

#[test]
fn fzf_v1_score_1() {
    let fzf = FzfV1::new().with_case_sensitivity(CaseSensitivity::Insensitive);

    let query = FzfQuery::from_str("oBZ");

    let score = fzf.score(query, "fooBarbaz").unwrap();

    assert_eq!(
        score,
        bonus::MATCH * 3 + bonus::CAMEL_123
            - penalty::GAP_START
            - penalty::GAP_EXTENSION * 3
    );
}
