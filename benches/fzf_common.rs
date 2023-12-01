mod common;

use criterion::{
    criterion_group,
    criterion_main,
    measurement::WallTime,
    BenchmarkGroup,
    Criterion,
    Throughput,
};
use norm::{
    fzf::{FzfParser, FzfV1},
    Metric,
};

/// The char length of the queries.
const QUERY_LEN: usize = 16;

fn bench(
    group: &mut BenchmarkGroup<WallTime>,
    query: &str,
    candidate: &str,
    bench_name: &str,
) {
    // Using V1 or V2 doesn't matter because we're not doing fuzzy matching
    // here.
    let mut fzf = FzfV1::new();

    let mut parser = FzfParser::new();

    let query = parser.parse(query);

    group.throughput(Throughput::Elements(1));

    group.bench_function(bench_name, |b| {
        b.iter(|| {
            let _ = fzf.distance(query, candidate);
        })
    });
}

fn group(c: &mut Criterion) -> BenchmarkGroup<WallTime> {
    c.benchmark_group("fzf")
}

fn exact(c: &mut Criterion) {
    let mut group = group(c);

    let candidate = common::MEDIUM_TEXT;

    let query = {
        let midpoint = candidate.len() / 2;
        let start = midpoint - QUERY_LEN / 2;
        let end = start + QUERY_LEN;
        &candidate[start..end]
    };

    let query = format!("'{query}");

    bench(&mut group, &query, candidate, "exact");
}

fn prefix(c: &mut Criterion) {
    let mut group = group(c);

    let candidate = common::MEDIUM_TEXT;

    let query = format!("^{query}", query = &candidate[..QUERY_LEN]);

    bench(&mut group, &query, candidate, "prefix");
}

fn suffix(c: &mut Criterion) {
    let mut group = group(c);

    let candidate = common::MEDIUM_TEXT;

    let query =
        format!("{query}$", query = &candidate[candidate.len() - QUERY_LEN..]);

    bench(&mut group, &query, candidate, "suffix");
}

fn equal(c: &mut Criterion) {
    let mut group = group(c);

    let candidate = &common::MEDIUM_TEXT[..QUERY_LEN];

    let query = format!("^{candidate}$");

    bench(&mut group, &query, candidate, "equal");
}

criterion_group!(benches, exact, prefix, suffix, equal);
criterion_main!(benches);
