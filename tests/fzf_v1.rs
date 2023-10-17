use norm::{FzfQuery, FzfV1, Metric};

#[test]
fn fzf_v1_empty_query() {
    let fzf = FzfV1::new();
    let empty = FzfQuery::from_str("");
    assert!(fzf.distance(empty, "foo").is_none());
}
