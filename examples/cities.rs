use std::ops::Range;

use norm::fzf::{FzfParser, FzfV2};
use norm::Metric;

fn main() {
    let mut fzf = FzfV2::new();

    let mut parser = FzfParser::new();

    let query = parser.parse("aa");

    let cities = ["Geneva", "Ulaanbaatar", "New York City", "Adelaide"];

    let mut results = cities
        .iter()
        .copied()
        .filter_map(|city| fzf.distance(query, city).map(|dist| (city, dist)))
        .collect::<Vec<_>>();

    results.sort_by_key(|(_city, dist)| *dist);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "Adelaide");
    assert_eq!(results[1].0, "Ulaanbaatar");

    let mut ranges: Vec<Range<usize>> = Vec::new();

    let _ = fzf.distance_and_ranges(query, results[0].0, &mut ranges);
    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0], 0..1);
    assert_eq!(ranges[1], 4..5);

    ranges.clear();

    let _ = fzf.distance_and_ranges(query, results[1].0, &mut ranges);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], 2..4);
}
